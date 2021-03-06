/// Incremental Graph Representations

use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use adapton::engine::{cell, ns, name_of_str, name_unit, Name};
use adapton::collections::{list_of_tree, tree_fold_seq, tree_of_list, Dir2, List, ListIntro,
                           MapIntro, MapElim, SetIntro, Tree, TreeIntro};
use adapton::collections::trie::{trie_fold_seq, Set, Trie, TrieIntro};

/// Representation of a graph as a list of edges, where edges are
/// a pair of node ids.
#[derive(Debug,PartialEq,Eq,Hash,Clone)]
pub struct Graph<Node> {
    edge_tree: Tree<(Node, Node)>,
}

/// Representation of a graph as finite map from node ids to
/// an outgoing adjacency list of node ids.
#[derive(Debug,PartialEq,Eq,Hash,Clone)]
pub struct AdjacencyGraph<Node>
    where Node: Debug + Clone + Hash + PartialEq + Eq + 'static
{
    adjacency_map: Trie<(Node, Tree<Node>)>,
}

trait NamedGraph<Node>: Debug + Clone + Hash + PartialEq + Eq {
    fn name(nm: Name, g: Self) -> Self;
}

/// Produce a graph.
pub trait GraphIntro<Node>: Debug + Clone + Hash + PartialEq + Eq {
    /// Yields an empty graph, i.e. no vertices or edges.
    fn empty() -> Self;
    /// Adds the edge `(src, dst)` named `nm` to the graph `graph`.
    fn add_edge(graph: Self, nm: Name, src: Node, dst: Node) -> Self;
}

/// Reduce a graph to a value.
pub trait GraphElim<Node>: Debug + Clone + Hash + PartialEq + Eq {
    /// Returns a set of the edges in the graph `graph`.
    fn edges(graph: &Self) -> Set<(Node, Node)>;
    /// Returns a set of the vertices of the graph `graph`.
    fn vertices(graph: &Self) -> Set<Node>;
    /// Returns a graph whose set of edges are the reversed edges of `graph`.
    fn reverse_edges(graph: &Self) -> Self;
}

impl<Node: Debug + Clone + Hash + PartialEq + Eq + 'static> NamedGraph<Node> for Graph<Node> {
    fn name(nm: Name, g: Graph<Node>) -> Graph<Node> {
        let edge_list = ns(name_of_str("list_of_tree"),
                           move || list_of_tree(g.edge_tree, Dir2::Left));
        let el = List::name_art(Some(nm), edge_list);
        let edge_tree = ns(name_of_str("tree_of_list"),
                           || tree_of_list::<_, _, Tree<_>, _>(Dir2::Left, el));
        Graph::<Node> { edge_tree: edge_tree }
    }
}

impl<Node: Debug + Clone + Hash + PartialEq + Eq + 'static> GraphIntro<Node> for Graph<Node> {
    fn empty() -> Graph<Node> {
        Graph::<Node> { edge_tree: Tree::nil() }
    }

    fn add_edge(graph: Graph<Node>, nm: Name, src: Node, dst: Node) -> Graph<Node> {
        let edge_list = ns(name_of_str("list_of_tree"),
                           move || list_of_tree(graph.edge_tree, Dir2::Left));
        let el = List::name_art(Some(nm), edge_list);
        let el = List::cons((src, dst), el);
        let edge_tree = ns(name_of_str("tree_of_list"),
                           || tree_of_list::<_, _, Tree<_>, _>(Dir2::Left, el));
        Graph::<Node> { edge_tree: edge_tree }
    }
}

impl<Node: Debug + Clone + Hash + PartialEq + Eq + 'static> GraphElim<Node> for Graph<Node> {
    fn edges(graph: &Graph<Node>) -> Set<(Node, Node)> {
        tree_fold_seq(graph.edge_tree.clone(),
                      Dir2::Left,
                      SetIntro::empty(),
                      Rc::new(|e, set| SetIntro::add(set, e)),
                      Rc::new(|_, set| set),
                      Rc::new(|nm: Name, _, set| {
                          TrieIntro::name(nm.clone(), TrieIntro::art(cell(nm, set)))
                      }))
    }

    fn vertices(graph: &Graph<Node>) -> Set<Node> {
        let edge_trie = ns(name_of_str("edge_trie_vertices"), || Self::edges(graph));
        trie_fold_seq(edge_trie,
                      SetIntro::empty(),
                      Rc::new(|((src,dst), ()), set| {
                          let add_src = SetIntro::add(set, src);
                          SetIntro::add(add_src, dst)
                      }),
                      Rc::new(move |set| set),
                      Rc::new(move |n: Name, set| {
                          Set::name(n.clone(), Set::art(cell(n, set)))
                      }))
    }

    fn reverse_edges(graph: &Graph<Node>) -> Graph<Node> {
        let es = Self::edges(graph);
        trie_fold_seq(es,
                      Self::empty(),
                      Rc::new(|((src, dst), ()), g| Self::add_edge(g, name_unit(), dst, src)),
                      Rc::new(|g| g),
                      Rc::new(|nm: Name, g| Self::name(nm, g)))
    }
}

impl<Node: Debug + Clone + Hash + PartialEq + Eq + 'static> NamedGraph<Node>
    for AdjacencyGraph<Node> {
    fn name(nm: Name, g: AdjacencyGraph<Node>) -> AdjacencyGraph<Node> {
        let adj = TrieIntro::name(nm.clone(),
                                  TrieIntro::art(cell(nm, g.adjacency_map)));
        AdjacencyGraph { adjacency_map: adj }
    }
}

impl<Node: Debug + Clone + Hash + PartialEq + Eq + 'static>
    GraphIntro<Node> for AdjacencyGraph<Node> {
        fn empty() -> AdjacencyGraph<Node> {
            AdjacencyGraph::<Node> {
                adjacency_map: MapIntro::empty(),
            }
        }

        fn add_edge(graph: AdjacencyGraph<Node>,
                    nm: Name, src: Node, dst: Node) -> AdjacencyGraph<Node> {
            let src_adj_list = MapElim::find(&graph.adjacency_map, &src);
            match src_adj_list {
                None => {
                    // let adj = List::name_art(Some(nm.clone()), List::nil());
                    let adj = List::cons(dst, List::nil());
                    let edge_tree = tree_of_list::<_, _, Tree<_>, _>(Dir2::Left, adj);
                    AdjacencyGraph::<Node> {
                        adjacency_map: TrieIntro::extend(nm, graph.adjacency_map,
                                                         (src, edge_tree)),
                    }
                }
                Some(adj_nodes) => {
                    let adj = list_of_tree(adj_nodes, Dir2::Left);
                    // let adj = List::name_art(Some(nm.clone()), adj);
                    let adj = List::cons(dst, adj);
                    let edge_tree = tree_of_list::<_, _, Tree<_>, _>(Dir2::Left, adj);
                    AdjacencyGraph::<Node> {
                        adjacency_map: TrieIntro::extend(nm, graph.adjacency_map,
                                                         (src, edge_tree)),
                    }
                }
            }
        }
    }

impl<Node: Debug + Copy + Clone + Hash + PartialEq + Eq + 'static>
    GraphElim<Node> for AdjacencyGraph<Node> {
        fn edges(graph: &AdjacencyGraph<Node>) -> Set<(Node, Node)> {
            trie_fold_seq(graph.adjacency_map.clone(), SetIntro::empty(),
                          Rc::new(|(src, dsts), set|
                                  tree_fold_seq(dsts, Dir2::Left, set,
                                                Rc::new(move |dst, set|
                                                        SetIntro::add(set, (src, dst))),
                                                Rc::new(|_, set| set),
                                                Rc::new(|nm: Name, _, set| {
                                                    TrieIntro::name(nm.clone(),
                                                                    TrieIntro::art(cell(nm,
                                                                                        set)))
                                                }))),
                          Rc::new(|set| set),
                          Rc::new(|nm: Name, set|
                                  TrieIntro::name(nm.clone(),
                                                  TrieIntro::art(cell(nm, set)))))
        }

        fn vertices(graph: &AdjacencyGraph<Node>) -> Set<Node> {
            trie_fold_seq(graph.adjacency_map.clone(), SetIntro::empty(),
                          Rc::new(|(src, dsts), set| {
                              let src_set = SetIntro::add(set, src);
                              tree_fold_seq(dsts, Dir2::Left, src_set,
                                            Rc::new(|dst, set| SetIntro::add(set, dst)),
                                            Rc::new(|_, set| set),
                                            Rc::new(|nm: Name, _, set|
                                                    TrieIntro::name(nm.clone(),
                                                                    TrieIntro::art(cell(nm, set)))))
                          }),
                          Rc::new(|set| set),
                          Rc::new(|nm: Name, set| TrieIntro::name(nm.clone(),
                                                                  TrieIntro::art(cell(nm, set)))))
        }

        fn reverse_edges(graph: &AdjacencyGraph<Node>) -> AdjacencyGraph<Node> {
            let es = Self::edges(graph);
            trie_fold_seq(es,
                          Self::empty(),
                          Rc::new(|((src, dst), ()), g| Self::add_edge(g, name_unit(), dst, src)),
                          Rc::new(|g| g),
                          Rc::new(|nm: Name, g| Self::name(nm, g)))
        }
    }

pub fn adjacency_of_edge_list<X: Hash + Clone + Debug + PartialEq + Eq>(el_graph: &Graph<X>)
                                                                        -> AdjacencyGraph<X> {
    let adj_graph = AdjacencyGraph::empty();
    tree_fold_seq(el_graph.edge_tree.clone(),
                  Dir2::Left,
                  adj_graph,
                  Rc::new(|(src, dst), g| AdjacencyGraph::add_edge(g, name_unit(), src, dst)),
                  Rc::new(|_, set| set),
                  Rc::new(|nm: Name, _, g: AdjacencyGraph<_>| AdjacencyGraph::name(nm, g)))
}

pub fn edge_list_of_adjacency
    <X: Hash + Clone + Debug + PartialEq + Eq>(adj_graph: &AdjacencyGraph<X>)
                                               -> Graph<X> {
    let edge_list = Graph::empty();
    trie_fold_seq(adj_graph.adjacency_map.clone(),
                  edge_list,
                  Rc::new(|(src, dsts): (X, Tree<X>), graph: Graph<X>| {
        let edge_tree = tree_fold_seq(dsts,
                                      Dir2::Left,
                                      graph.edge_tree,
                                      Rc::new(move |dst, g| {
            let edge_list = ns(name_of_str("list_of_tree"),
                               move || list_of_tree(g, Dir2::Left));
            let el = List::cons((src.clone(), dst), edge_list);
            ns(name_of_str("tree_of_list"),
               || tree_of_list::<_, _, Tree<_>, _>(Dir2::Left, el))
        }),
                                      Rc::new(|_, g| g),
                                      Rc::new(|nm: Name, _, g| {
            let edge_list = ns(name_of_str("list_of_tree"),
                               move || list_of_tree(g, Dir2::Left));
            let el = List::name_art(Some(nm), edge_list);
            ns(name_of_str("tree_of_list"),
               || tree_of_list::<_, _, Tree<_>, _>(Dir2::Left, el))
        }));
        Graph::<X> { edge_tree: edge_tree }
    }),
                  Rc::new(|g| g),
                  Rc::new(|nm: Name, g: Graph<X>| Graph::name(nm, g)))
}
