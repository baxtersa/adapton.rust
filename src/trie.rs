use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::rc::Rc;
use std::cmp::min;

use adapton::collections::{ListIntro, ListElim, MapIntro, MapElim, list_fold};
use adapton::bitstring::*;
use adapton::engine::*;
use macros::*;

/// Probablistically Balanced Trie
/// Rough implementation of probabilistic tries from OOPSLA 2015 paper.
///
/// See also: [Tries in OCaml](http://github.com/plum-umd/adapton.ocaml)
#[derive(Debug,PartialEq,Eq,Clone)]
pub enum Trie<X> {
    Nil(BS),
    Leaf(BS, X),
    Bin(BS, Box<Trie<X>>, Box<Trie<X>>),
    Root(Meta, Box<Trie<X>>),
    Name(Name, Box<Trie<X>>),
    Art(Art<Trie<X>>),
}

pub const PLACEMENT_SEED: u64 = 42;

/// Metadata held by the root node.
#[derive(Debug,PartialEq,Eq,Hash,Clone)]
pub struct Meta {
    pub min_depth: i64,
}

pub trait MetaT {
    fn hash_seeded(&self, u64);
}

impl MetaT for Meta {
    fn hash_seeded(&self, seed: u64) {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        "Adapton.Trie.Meta".hash(&mut hasher);
        self.min_depth.hash(&mut hasher);
    }
}

pub trait TrieIntro<X>: Debug + Hash + PartialEq + Eq + Clone + 'static {
    fn nil(BS) -> Self;
    fn leaf(BS, X) -> Self;
    fn bin(BS, Self, Self) -> Self;
    fn root(Meta, Self) -> Self;

    // requisite "adaptonic" constructors: `name` and `art`:
    fn name(Name, Self) -> Self;
    fn art(Art<Self>) -> Self;

    fn empty(Meta) -> Self;
    fn singleton(Meta, Name, X) -> Self;
    fn extend(Name, Self, X) -> Self;
}

pub trait TrieElim<X>: Debug + Hash + PartialEq + Eq + Clone + 'static {
    fn find(&Self, &X, i64) -> Option<X>;
    fn is_empty(&Self) -> bool;
    fn split_atomic(Self) -> Self;

    fn elim<Res, NilC, LeafC, BinC, RootC, NameC>(Self, NilC, LeafC, BinC, RootC, NameC) -> Res
        where NilC: FnOnce(BS) -> Res,
              LeafC: FnOnce(BS, X) -> Res,
              BinC: FnOnce(BS, Self, Self) -> Res,
              RootC: FnOnce(Meta, Self) -> Res,
              NameC: FnOnce(Name, Self) -> Res;

    fn elim_arg<Arg, Res, NilC, LeafC, BinC, RootC, NameC>(Self,
                                                           Arg,
                                                           NilC,
                                                           LeafC,
                                                           BinC,
                                                           RootC,
                                                           NameC)
                                                           -> Res
        where NilC: FnOnce(BS, Arg) -> Res,
              LeafC: FnOnce(BS, X, Arg) -> Res,
              BinC: FnOnce(BS, Self, Self, Arg) -> Res,
              RootC: FnOnce(Meta, Self, Arg) -> Res,
              NameC: FnOnce(Name, Self, Arg) -> Res;

    fn elim_ref<Res, NilC, LeafC, BinC, RootC, NameC>(&Self,
                                                      NilC,
                                                      LeafC,
                                                      BinC,
                                                      RootC,
                                                      NameC)
                                                      -> Res
        where NilC: FnOnce(&BS) -> Res,
              LeafC: FnOnce(&BS, &X) -> Res,
              BinC: FnOnce(&BS, &Self, &Self) -> Res,
              RootC: FnOnce(&Meta, &Self) -> Res,
              NameC: FnOnce(&Name, &Self) -> Res;
}

impl<X: Debug + Hash + PartialEq + Eq + Clone + 'static> Trie<X> {
    fn mfn(nm: Name, meta: Meta, trie: Self, bs: BS, elt: X, hash: u64) -> Self {
        match trie {
            Trie::Nil(_) if BS::length(bs) < meta.min_depth => {
                let h_ = hash >> 1;
                let bs0 = BS::prepend(0, bs);
                let bs1 = BS::prepend(1, bs);
                let mt0 = Self::nil(bs0);
                let mt1 = Self::nil(bs1);
                if hash % 2 == 0 {
                    Self::bin(bs, Self::mfn(nm, meta, mt0, bs0, elt, h_), mt1)
                } else {
                    Self::bin(bs, mt0, Self::mfn(nm, meta, mt1, bs1, elt, h_))
                }
            }
            Trie::Nil(_) => Trie::Leaf(bs, elt),
            Trie::Leaf(_, e) => {
                let depth = BS::length(bs);
                if depth >= BS::MAX_LEN || e == elt {
                    Self::leaf(bs, e)
                } else if depth < BS::MAX_LEN {
                    Self::mfn(nm,
                              meta,
                              Self::split_atomic(Self::leaf(bs, e)),
                              bs,
                              elt,
                              hash)
                } else {
                    panic!("Bad value found in nadd:\nLeaf(bs, e)\n{:?}",
                           Self::leaf(bs, e));
                }
            }
            Trie::Bin(bs, left, right) => {
                let h_ = hash >> 1;
                if hash % 2 == 0 {
                    let l = Self::mfn(nm, meta, *left, BS::prepend(0, bs), elt, h_);
                    Self::bin(bs, l, *right)
                } else {
                    let r = Self::mfn(nm, meta, *right, BS::prepend(1, bs), elt, h_);
                    Self::bin(bs, *left, r)
                }
            }
            Trie::Name(_, box Trie::Art(a)) => Self::mfn(nm, meta, force(&a), bs, elt, hash),
            t => panic!("Bad value found in nadd:\n{:?}\n", t),
        }
    }

    fn root_mfn(_: Name, nm: Name, trie: Self, elt: X) -> Self {
        match trie {
            Trie::Name(_, box Trie::Art(a)) => {
                match force(&a) {
                    Trie::Root(meta, t) => {
                        let (nm, nm_) = name_fork(nm);
                        let mut hasher = DefaultHasher::new();
                        elt.hash(&mut hasher);
                        let a = Self::mfn(nm_,
                                          meta.clone(),
                                          *t,
                                          BS {
                                              length: 0,
                                              value: 0,
                                          },
                                          elt,
                                          hasher.finish());
                        Self::root(meta, Self::name(nm, Self::art(put(a))))
                    }
                    t @ Trie::Name(_, box Trie::Art(_)) => Self::root_mfn(nm.clone(), nm, t, elt),
                    t => panic!("Non-root node entry to `Trie.extend': {:?}", t),
                }
            }
            _ => panic!("None-name node at entry to `Trie.extend'"),
        }
    }
}

impl<X: Debug + Hash + PartialEq + Eq + Clone + 'static> TrieIntro<X> for Trie<X> {
    fn nil(bs: BS) -> Self {
        Trie::Nil(bs)
    }
    fn leaf(bs: BS, x: X) -> Self {
        Trie::Leaf(bs, x)
    }
    fn bin(bs: BS, l: Self, r: Self) -> Self {
        Trie::Bin(bs, Box::new(l), Box::new(r))
    }
    fn root(meta: Meta, trie: Self) -> Self {
        Trie::Root(meta, Box::new(trie))
    }
    fn name(nm: Name, trie: Self) -> Self {
        Trie::Name(nm, Box::new(trie))
    }
    fn art(art: Art<Self>) -> Self {
        Trie::Art(art)
    }

    fn empty(meta: Meta) -> Self {
        if meta.min_depth > BS::MAX_LEN {
            println!("Cannot make Adapton.Trie with min_depth > {} (given {})",
                     BS::MAX_LEN,
                     meta.min_depth);
        }
        let min = min(meta.min_depth, BS::MAX_LEN);
        let meta = Meta { min_depth: min };
        let nm = name_of_str("trie_empty");
        let (nm1, nm2) = name_fork(nm);
        let mtbs = BS {
            length: 0,
            value: 0,
        };
        let nil_art = thunk!(nm2.clone() =>> Self::nil, bs:mtbs);
        let root_art = thunk!(nm1.clone() =>> Self::root, meta:meta,
                              trie:Self::name(nm2, Self::art(nil_art)));
        Self::name(nm1.clone(), Self::art(root_art))
    }

    fn singleton(meta: Meta, nm: Name, elt: X) -> Self {
        Self::extend(nm, TrieIntro::empty(meta), elt)
    }

    fn extend(nm: Name, trie: Self, elt: X) -> Self {
        let (nm, nm_) = name_fork(nm);
        // let a = Self::root_mfn(nm.clone(), nm_, trie, elt);
        let root_mfn_art = put(Self::root_mfn(nm.clone(), nm_, trie, elt));
        Self::name(nm, Self::art(root_mfn_art))
    }
}

impl<X: Debug + Hash + PartialEq + Eq + Clone + 'static> Hash for Trie<X> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Trie::Nil(bs) => bs.hash(state),
            Trie::Leaf(bs, ref x) => {
                x.hash(state);
                bs.hash(state)
            }
            Trie::Bin(bs, ref left, ref right) => {
                right.hash(state);
                left.hash(state);
                bs.hash(state)
            }
            Trie::Root(ref md, ref t) => {
                t.hash(state);
                md.hash_seeded(state.finish());
            }
            Trie::Name(ref nm, ref t) => {
                t.hash(state);
                nm.hash(state)
            }
            Trie::Art(ref art_t) => art_t.hash(state),
        }
    }
}

impl<X: Debug + Hash + PartialEq + Eq + Clone + 'static> TrieElim<X> for Trie<X> {
    fn find(trie: &Self, elt: &X, i: i64) -> Option<X> {
        Self::elim_ref(trie,
                       |_| None,
                       |_, x| if *elt == *x { Some(x.clone()) } else { None },
                       |_, left, right| if i % 2 == 0 {
                           Self::find(left, elt, i >> 1)
                       } else {
                           Self::find(right, elt, i >> 1)
                       },
                       |_, t| Self::find(t, elt, i),
                       |_, t| Self::find(t, elt, i))
    }

    fn is_empty(trie: &Self) -> bool {
        Self::elim_ref(trie,
                       |_| true,
                       |_, _| false,
                       |_, _, _| false,
                       |_, t| Self::is_empty(t),
                       |_, t| Self::is_empty(t))
    }

    fn split_atomic(trie: Self) -> Self {
        fn suffix(bs: BS, k: i64) -> bool {
            bs.value & k == bs.value
        }
        match trie {
            t @ Trie::Nil(_) |
            t @ Trie::Bin(_, _, _) => t,
            Trie::Leaf(bs, e) => {
                let bs0 = BS::prepend(0, bs);
                let bs1 = BS::prepend(1, bs);
                let mut hasher = DefaultHasher::new();
                e.hash(&mut hasher);
                if suffix(bs1, hasher.finish() as i64) {
                    Self::bin(bs, Self::nil(bs0), Self::leaf(bs1, e))
                } else {
                    Self::bin(bs, Self::leaf(bs0, e), Self::nil(bs1))
                }
            }
            _ => panic!("Bad split_atomic(t)"),
        }
    }

    fn elim<Res, NilC, LeafC, BinC, RootC, NameC>(trie: Self,
                                                  nil: NilC,
                                                  leaf: LeafC,
                                                  bin: BinC,
                                                  root: RootC,
                                                  name: NameC)
                                                  -> Res
        where NilC: FnOnce(BS) -> Res,
              LeafC: FnOnce(BS, X) -> Res,
              BinC: FnOnce(BS, Self, Self) -> Res,
              RootC: FnOnce(Meta, Self) -> Res,
              NameC: FnOnce(Name, Self) -> Res
    {
        match trie {
            Trie::Nil(bs) => nil(bs),
            Trie::Leaf(bs, x) => leaf(bs, x),
            Trie::Bin(bs, l, r) => bin(bs, *l, *r),
            Trie::Name(nm, t) => name(nm, *t),
            Trie::Root(meta, t) => root(meta, *t),
            Trie::Art(art) => {
                let trie = force(&art);
                Self::elim(trie, nil, leaf, bin, root, name)
            }
        }
    }

    fn elim_arg<Arg, Res, NilC, LeafC, BinC, RootC, NameC>(trie: Self,
                                                           arg: Arg,
                                                           nil: NilC,
                                                           leaf: LeafC,
                                                           bin: BinC,
                                                           root: RootC,
                                                           name: NameC)
                                                           -> Res
        where NilC: FnOnce(BS, Arg) -> Res,
              LeafC: FnOnce(BS, X, Arg) -> Res,
              BinC: FnOnce(BS, Self, Self, Arg) -> Res,
              RootC: FnOnce(Meta, Self, Arg) -> Res,
              NameC: FnOnce(Name, Self, Arg) -> Res
    {
        match trie {
            Trie::Nil(bs) => nil(bs, arg),
            Trie::Leaf(bs, x) => leaf(bs, x, arg),
            Trie::Bin(bs, l, r) => bin(bs, *l, *r, arg),
            Trie::Name(nm, t) => name(nm, *t, arg),
            Trie::Root(meta, t) => root(meta, *t, arg),
            Trie::Art(art) => {
                let trie = force(&art);
                Self::elim_arg(trie, arg, nil, leaf, bin, root, name)
            }
        }
    }

    fn elim_ref<Res, NilC, LeafC, BinC, RootC, NameC>(trie: &Self,
                                                      nil: NilC,
                                                      leaf: LeafC,
                                                      bin: BinC,
                                                      root: RootC,
                                                      name: NameC)
                                                      -> Res
        where NilC: FnOnce(&BS) -> Res,
              LeafC: FnOnce(&BS, &X) -> Res,
              BinC: FnOnce(&BS, &Self, &Self) -> Res,
              RootC: FnOnce(&Meta, &Self) -> Res,
              NameC: FnOnce(&Name, &Self) -> Res
    {
        match *trie {
            Trie::Nil(ref bs) => nil(bs),
            Trie::Leaf(ref bs, ref x) => leaf(bs, x),
            Trie::Bin(ref bs, ref l, ref r) => bin(bs, &*l, &*r),
            Trie::Name(ref nm, ref t) => name(nm, &*t),
            Trie::Root(ref meta, ref t) => root(meta, &*t),
            Trie::Art(ref art) => {
                let trie = force(art);
                Self::elim_ref(&trie, nil, leaf, bin, root, name)
            }
        }
    }
}

impl<Dom:Debug+Hash+PartialEq+Eq+Clone+'static,
     Cod:Debug+Hash+PartialEq+Eq+Clone+'static>
    MapIntro<Dom,Cod>
    for
    Trie<(Dom,Cod)> {
        fn empty () -> Self {
            ns(name_of_str("map_intro_trie_empty"), || {
                let meta = Meta { min_depth: 1 };
                TrieIntro::empty(meta)
            })
        }
        fn update (map:Self, d:Dom, c:Cod) -> Self {
            TrieIntro::extend(name_unit(), map, (d,c))
        }
}

impl<Dom:Debug+Hash+PartialEq+Eq+Clone+'static,
     Cod:Debug+Hash+PartialEq+Eq+Clone+'static>
    MapElim<Dom,Cod>
    for
    Trie<(Dom,Cod)> {
        fn find(map:&Self, d:&Dom) -> Option<Cod> {
            let mut hasher = DefaultHasher::new();
            d.hash(&mut hasher);
            let i = hasher.finish() as i64;
            fn find_hash<
                    Dom:Debug+Hash+PartialEq+Eq+Clone+'static,
                Cod:Debug+Hash+PartialEq+Eq+Clone+'static>
                (map:&Trie<(Dom,Cod)>,d:&Dom,i:i64) -> Option<Cod> {
                    TrieElim::elim_ref(map,
                                       |_| None,
                                       |_, &(ref d2, ref c)| if *d == *d2 {
                                           Some(c.clone())
                                       } else {
                                           None
                                       },
                                       |_, ref left, ref right| if i % 2 == 0 {
                                           find_hash(left, d, i >> 1)
                                       } else {
                                           find_hash(right, d, i >> 1)
                                       },
                                       |_, ref t| find_hash(t, d, i),
                                       |_, ref t| {
                                           find_hash(t, d, i)
                                       })
                };
            find_hash(map, d, i)
        }

        fn remove (_map:Self, _d:&Dom) -> (Self, Option<Cod>) {
            unimplemented!()
        }

        fn fold<Res,F> (map:Self, res:Res, body:Rc<F>) -> Res
            where F:Fn(Dom, Cod, Res) -> Res+'static,
                  Res:Hash+Debug+Eq+Clone+'static

        {
            trie_fold(map, res, Rc::new(move |(d,c),r|(*body)(d,c,r)) )
        }

        fn append(_map:Self, _other:Self) -> Self {
            unimplemented!()
        }
    }

pub type Set<X> = Trie<(X, ())>;

pub fn trie_fold
    <X, T:TrieElim<X>, Res:Hash+Debug+Eq+Clone+'static, F: 'static>
    (t: T, res:Res, f: Rc<F>) -> Res
    where F: Fn(X, Res) -> Res {
    T::elim_arg(t,
                (res, f),
                |_, (arg, _)| arg,
                |_, x, (arg, f)| f(x, arg),
                |_, left, right, (arg, f)| trie_fold(right, trie_fold(left, arg, f.clone()), f),
                |_, t, (arg, f)| trie_fold(t, arg, f),
                |nm, t, (arg, f)| memo!(nm =>> trie_fold, t:t, res:arg ;; f:f))
}

pub fn trie_fold_seq<X,
                     T: TrieElim<X>,
                     Res: Hash + Debug + Eq + Clone + 'static,
                     LeafC: 'static,
                     BinC: 'static,
                     NameC: 'static>
    (trie: T,
     res: Res,
     leaf: Rc<LeafC>,
     bin: Rc<BinC>,
     name: Rc<NameC>)
     -> Res
    where LeafC: Fn(X, Res) -> Res,
          BinC: Fn(Res) -> Res,
          NameC: Fn(Name, Res) -> Res
{
    T::elim_arg(trie,
                (res, (leaf, bin, name)),
                |_, (res, _)| res,
                |_, x, (res, (leaf, _, _))| leaf(x, res),
                |_, left, right, (res, (leaf, bin, name))| {
                    let res = trie_fold_seq(left, res, leaf.clone(), bin.clone(), name.clone());
                    let res = (&bin)(res);
                    let res = trie_fold_seq(right, res, leaf, bin, name);
                    res
                },
                |_, t, (res, (leaf, bin, name))| trie_fold_seq(t, res, leaf, bin, name),
                |nm, t, (res, (leaf, bin, name))| {
                    let res = memo!(nm.clone() =>> trie_fold_seq, trie:t, res:res ;;
                                    leaf:leaf, bin:bin, name:name.clone());
                    let res = name(nm, res);
                    res
                })
}

pub fn trie_fold_seq_nm<X,
                        T: TrieElim<X>,
                        Res: Hash + Debug + Eq + Clone + 'static,
                        LeafC: 'static,
                        BinC: 'static,
                        NameC: 'static>
    (trie: T,
     res: Res,
     nm: Option<Name>,
     leaf: Rc<LeafC>,
     bin: Rc<BinC>,
     name: Rc<NameC>)
     -> Res
    where LeafC: Fn(Option<Name>, X, Res) -> Res,
          BinC: Fn(Res) -> Res,
          NameC: Fn(Name, Res) -> Res
{
    T::elim_arg(trie,
                (res, (nm, leaf, bin, name)),
                |_, (res, _)| res,
                |_, x, (res, (nm, leaf, _, _))| leaf(nm, x, res),
                |_, left, right, (res, (nm, leaf, bin, name))| {
        let res = trie_fold_seq_nm(left,
                                   res,
                                   nm.clone(),
                                   leaf.clone(),
                                   bin.clone(),
                                   name.clone());
        let res = (&bin)(res);
        let res = trie_fold_seq_nm(right, res, nm, leaf, bin, name);
        res
    },
                |_, t, (res, (nm, leaf, bin, name))| trie_fold_seq_nm(t, res, nm, leaf, bin, name),
                |n, t, (res, (_, leaf, bin, name))| {
        let res = memo!(n.clone() =>> trie_fold_seq_nm, trie:t, res:res, nm:Some(n.clone()) ;;
                                    leaf:leaf, bin:bin, name:name.clone());
        let res = name(n, res);
        res
    })
}

pub fn trie_of_list<X: Hash + Clone + Debug + 'static,
                    T: TrieIntro<X> + 'static,
                    L: ListElim<X> + ListIntro<X> + 'static>
    (list: L)
     -> T {
    list_fold(list,
              T::empty(Meta { min_depth: 1 }),
              Rc::new(|x, trie_acc| T::extend(name_unit(), trie_acc, x)))
}

pub fn list_of_trie<X: Hash + Clone + Debug + 'static,
                    T: TrieElim<X> + 'static,
                    L: ListIntro<X> + 'static>
    (trie: T)
     -> L {
    trie_fold(trie,
              ListIntro::nil(),
              Rc::new(|elt, list| ListIntro::cons(elt, list)))
}

pub fn list_of_trieset<X: Hash + Clone + Debug + 'static,
                       S: TrieElim<(X, ())> + 'static,
                       L: ListIntro<X> + 'static>
    (set: S)
     -> L {
    trie_fold(set,
                  ListIntro::nil(),
                  Rc::new(|(elt, ()), list| ListIntro::cons(elt, list)))
}

pub fn trie_fold_up<X,
                    T: TrieElim<X>,
                    Res: Hash + Debug + Eq + Clone + 'static,
                    NilF: 'static,
                    LeafF: 'static,
                    BinF: 'static,
                    RootF: 'static,
                    NameF: 'static>
    (trie: T,
     nil: Rc<NilF>,
     leaf: Rc<LeafF>,
     bin: Rc<BinF>,
     root: Rc<RootF>,
     name: Rc<NameF>)
     -> Res
    where NilF: Fn(BS) -> Res,
          LeafF: Fn(BS, X) -> Res,
          BinF: Fn(BS, Res, Res) -> Res,
          RootF: Fn(Meta, Res) -> Res,
          NameF: Fn(Name, Res) -> Res
{
    T::elim_arg(trie,
                (nil, leaf, bin, root, name),
                |bs, (nil, _, _, _, _)| nil(bs),
                |bs, x, (_, leaf, _, _, _)| leaf(bs, x),
                |x, l, r, (nil, leaf, bin, root, name)| {
        let resl = trie_fold_up(l,
                                nil.clone(),
                                leaf.clone(),
                                bin.clone(),
                                root.clone(),
                                name.clone());
        let resr = trie_fold_up(r, nil, leaf, bin.clone(), root, name);
        let res = bin(x, resl, resr);
        res
    },
                |meta, t, (nil, leaf, bin, root, name)| {
                    let res = trie_fold_up(t, nil, leaf, bin, root.clone(), name);
                    root(meta, res)
                },
                |n, t, (nil, leaf, bin, root, name)| {
                    let res = memo!(n.clone() =>> trie_fold_up, trie:t ;;
                                    nil:nil, leaf:leaf, bin:bin, root:root, name:name.clone());
                    let res = name(n, res);
                    res
                })
}

/// Produces a trie with the same structure as its input, but without
/// any articulations.  Useful for `println`-style debugging, and for
/// equality comparisons across distinct engine implementations (e.g.,
/// to verify the DCG-based engine).
pub fn eager_trie_of_trie<X: Hash + Clone + 'static,
                          TE: TrieElim<X> + 'static,
                          TI: TrieIntro<X> + 'static>
    (trie: TE)
     -> TI {
    trie_fold_up(trie,
                 Rc::new(|bs| TI::nil(bs)),
                 Rc::new(|bs, x| TI::leaf(bs, x)),
                 Rc::new(|bs, l, r| TI::bin(bs, l, r)),
                 Rc::new(|meta, t| TI::root(meta, t)),
                 Rc::new(|n, t| TI::name(n, t)))
}
