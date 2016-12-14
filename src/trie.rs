use std::fmt::Debug;
use std::hash::{Hash, Hasher, SipHasher};
use std::collections::hash_map::DefaultHasher;
use std::cmp::min;

use adapton::bitstring::*;
use adapton::engine::{Art, Name, force, name_fork, name_of_str, name_unit, put};

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
    min_depth: i64,
}

pub trait MetaT {
    fn hash_seeded(&self, u64);
}

impl MetaT for Meta {
    fn hash_seeded(&self, seed: u64) {
        let mut hasher = SipHasher::new_with_keys(0, seed);
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
                let h_ = hash << 1;
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
            Trie::Leaf(bs_, e) => {
                let depth = BS::length(bs);
                if depth >= BS::MAX_LEN || e == elt {
                    Self::leaf(bs, elt)
                } else if depth < BS::MAX_LEN {
                    Self::mfn(nm,
                              meta,
                              Self::split_atomic(Self::leaf(bs_, e)),
                              bs,
                              elt,
                              hash)
                } else {
                    panic!("Bad value found in nadd:\nLeaf(bs, e)\n");
                }
            }
            Trie::Bin(bs, left, right) => {
                let h_ = hash << 1;
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
                    _ => panic!("Non-root node entry to `Trie.extend'"),
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
        let nm = name_of_str("empty");
        let (nm1, nm2) = name_fork(nm);
        let mtbs = BS {
            length: 0,
            value: 0,
        };
        Self::name(nm1,
                   Self::art(put(Self::root(meta,
                                            Self::name(nm2, Self::art(put(Self::nil(mtbs))))))))
    }

    fn singleton(meta: Meta, nm: Name, elt: X) -> Self {
        Self::extend(nm, TrieIntro::empty(meta), elt)
    }

    fn extend(nm: Name, trie: Self, elt: X) -> Self {
        let (nm, nm_) = name_fork(nm);
        let a = Self::root_mfn(nm.clone(), nm_, trie, elt);
        Self::name(nm, Self::art(put(a)))
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

#[test]
fn test_is_empty() {
    let meta = Meta { min_depth: 1 };
    let empty = TrieIntro::<usize>::empty(meta.clone());
    let singleton = Trie::singleton(meta.clone(), name_unit(), 7);
    assert!(Trie::<usize>::is_empty(&Trie::nil(BS {
        length: 0,
        value: 0,
    })));
    assert!(Trie::is_empty(&empty));

    assert!(!Trie::is_empty(&Trie::leaf(BS {
                                            length: 0,
                                            value: 0,
                                        },
                                        0)));
    assert!(!Trie::is_empty(&singleton));
}

#[test]
fn test_equal() {
    let meta = Meta { min_depth: 1 };
    let empty: Trie<usize> = TrieIntro::empty(meta.clone());
    let singleton_7 = Trie::singleton(meta.clone(), name_unit(), 7);
    let singleton_7_ = Trie::singleton(meta.clone(), name_unit(), 7);
    let singleton_8 = Trie::singleton(meta.clone(), name_unit(), 8);
    assert_eq!(empty, empty);
    assert_eq!(singleton_7, singleton_7);
    assert_eq!(singleton_7, singleton_7_);
    assert_eq!(singleton_8, singleton_8);

    assert_ne!(empty, singleton_7);
    assert_ne!(empty, singleton_8);
    assert_ne!(singleton_7, singleton_8);
}

pub trait SetIntro<X>: Debug + Hash + PartialEq + Eq + Clone + 'static {
    fn empty() -> Self;
    fn add(Self, e: X) -> Self;
    // fn remove(Self, e: &X) -> Self;
    // fn union(Self, Self) -> Self;
    // fn inter(Self, Self) -> Self;
    // fn diff(Self, Self) -> Self;
}

pub trait SetElim<X>: Debug + Hash + PartialEq + Eq + Clone + 'static {
    fn mem(set: &Self, e: &X) -> bool;
    // fn fold<Res, F>(set: Self, Res, F) -> Res where F: Fn(X, Res) -> Res;
}

impl<X, Set: TrieIntro<X> + TrieElim<X>> SetIntro<X> for Set {
    fn empty() -> Self {
        let meta = Meta { min_depth: 1 };
        Self::empty(meta)
    }

    fn add(set: Self, elt: X) -> Self {
        Self::extend(name_unit(), set, elt)
    }
}

impl<X: Hash, Set: TrieIntro<X> + TrieElim<X>> SetElim<X> for Set {
    fn mem(set: &Self, elt: &X) -> bool {
        let mut hasher = DefaultHasher::new();
        elt.hash(&mut hasher);
        match Set::find(set, elt, hasher.finish() as i64) {
            Some(_) => true,
            None => false,
        }
    }
}

type Set<X> = Trie<X>;

// Set membership is consistent after additions.
#[test]
fn test_set() {
    let e: Set<usize> = SetIntro::empty();
    assert!(!Set::mem(&e, &7));
    assert!(!Set::mem(&e, &1));
    let s = SetIntro::add(e, 7);
    let s = SetIntro::add(s, 1);
    let s = SetIntro::add(s, 8);
    println!("{:?}", s);
    assert!(Set::mem(&s, &1));
    assert!(Set::mem(&s, &7));
    assert!(Set::mem(&s, &8));
    assert!(!Set::mem(&s, &0));
}

// Order in which elements are added to sets doesn't matter.
#[test]
fn test_set_equal() {
    let e: Set<usize> = SetIntro::empty();
    let s = SetIntro::add(e, 7);
    let s = SetIntro::add(s, 1);
    let s = SetIntro::add(s, 8);

    let e: Set<usize> = SetIntro::empty();
    let t = SetIntro::add(e, 8);
    let t = SetIntro::add(t, 7);
    let t = SetIntro::add(t, 1);
    assert_eq!(s, t);
}
