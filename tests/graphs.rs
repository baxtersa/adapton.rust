extern crate adapton;

use adapton::engine::*;
use adapton::engine::manage::*;
use adapton::collections::{tree_of_list, Dir2, SetElim, Tree};
use adapton::collections::trie::*;
use adapton::collections::graph::*;

mod graphs {
    use super::*;

    #[test]
    fn test_empty_edge_graph() {
        let empty = Graph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::edges(&empty)));
    }

    #[test]
    fn test_empty_vertex_graph() {
        let empty = Graph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::vertices(&empty)));
    }

    #[test]
    fn test_empty_edge_adj_graph() {
        let empty = AdjacencyGraph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::edges(&empty)));
    }

    #[test]
    fn test_empty_vertex_adj_graph() {
        let empty = AdjacencyGraph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::vertices(&empty)));
    }

    #[test]
    fn test_non_empty_edge_graph() {
        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::edges(&g)));
        assert!(SetElim::is_mem(&GraphElim::edges(&g), &(0, 1)));
        assert!(SetElim::is_mem(&GraphElim::edges(&g), &(2, 3)));
        assert!(!SetElim::is_mem(&GraphElim::edges(&g), &(1, 0)));
        assert!(!SetElim::is_mem(&GraphElim::edges(&g), &(3, 2)));
    }

    #[test]
    fn test_non_empty_vertex_graph() {
        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::vertices(&g)));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &1));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &3));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &0));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &2));
    }

    #[test]
    fn test_non_empty_edge_adj_graph() {
        let g: AdjacencyGraph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                                        name_pair(name_of_usize(0),
                                                                  name_of_usize(1)),
                                                        0,
                                                        1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::edges(&g)));
        assert!(SetElim::is_mem(&GraphElim::edges(&g), &(0, 1)));
        assert!(SetElim::is_mem(&GraphElim::edges(&g), &(2, 3)));
        assert!(!SetElim::is_mem(&GraphElim::edges(&g), &(1, 0)));
        assert!(!SetElim::is_mem(&GraphElim::edges(&g), &(3, 2)));
    }

    #[test]
    fn test_non_empty_vertex_adj_graph() {
        let g: AdjacencyGraph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                                        name_pair(name_of_usize(0),
                                                                  name_of_usize(1)),
                                                        0,
                                                        1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::vertices(&g)));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &1));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &3));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &0));
        assert!(SetElim::is_mem(&GraphElim::vertices(&g), &2));
    }
}

mod graph_conversion {
    use super::*;

    #[test]
    fn test_empty_edge_graph_to_adj() {
        let empty = Graph::<usize>::empty();
        let empty_adj = adjacency_of_edge_list(&empty);
        assert!(TrieElim::is_empty(&GraphElim::edges(&empty_adj)));
    }

    #[test]
    fn test_empty_vertex_graph_to_adj() {
        let empty = Graph::<usize>::empty();
        let empty_adj = adjacency_of_edge_list(&empty);
        assert!(TrieElim::is_empty(&GraphElim::vertices(&empty_adj)));
    }

    #[test]
    fn test_non_empty_edge_graph_to_adj() {
        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        let adj_g = adjacency_of_edge_list(&g);
        assert!(!TrieElim::is_empty(&GraphElim::edges(&adj_g)));
    }

    #[test]
    fn test_non_empty_vertex_graph_to_adj() {
        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        let adj_g = adjacency_of_edge_list(&g);
        assert!(!TrieElim::is_empty(&GraphElim::vertices(&adj_g)));
    }
}

mod graphs_dcg {
    use super::*;

    #[test]
    fn test_empty_edge_graph() {
        init_dcg();

        let empty = Graph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::edges(&empty)));
    }

    #[test]
    fn test_empty_vertex_graph() {
        init_dcg();

        let empty = Graph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::vertices(&empty)));
    }

    #[test]
    fn test_empty_edge_adj_graph() {
        init_dcg();

        let empty = AdjacencyGraph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::edges(&empty)));
    }

    #[test]
    fn test_empty_vertex_adj_graph() {
        init_dcg();

        let empty = AdjacencyGraph::<usize>::empty();
        assert!(TrieElim::is_empty(&GraphElim::vertices(&empty)));
    }

    #[test]
    fn test_non_empty_edge_graph() {
        init_dcg();

        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::edges(&g)));
    }

    #[test]
    fn test_non_empty_vertex_graph() {
        init_dcg();

        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::vertices(&g)));
    }

    #[test]
    fn test_non_empty_edge_adj_graph() {
        init_dcg();

        let g: AdjacencyGraph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                                        name_pair(name_of_usize(0),
                                                                  name_of_usize(1)),
                                                        0,
                                                        1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::edges(&g)));
    }

    #[test]
    fn test_non_empty_vertex_adj_graph() {
        init_dcg();

        let g: AdjacencyGraph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                                        name_pair(name_of_usize(0),
                                                                  name_of_usize(1)),
                                                        0,
                                                        1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        assert!(!TrieElim::is_empty(&GraphElim::vertices(&g)));
    }
}

mod graph_conversion_dcg {
    use super::*;

    #[test]
    fn test_empty_edge_graph_to_adj() {
        init_dcg();

        let empty = Graph::<usize>::empty();
        let empty_adj = adjacency_of_edge_list(&empty);
        assert!(TrieElim::is_empty(&GraphElim::edges(&empty_adj)));
    }

    #[test]
    fn test_empty_vertex_graph_to_adj() {
        init_dcg();

        let empty = Graph::<usize>::empty();
        let empty_adj = adjacency_of_edge_list(&empty);
        assert!(TrieElim::is_empty(&GraphElim::vertices(&empty_adj)));
    }

    #[test]
    fn test_non_empty_edge_graph_to_adj() {
        init_dcg();

        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        let adj_g = adjacency_of_edge_list(&g);
        assert!(!TrieElim::is_empty(&GraphElim::edges(&adj_g)));
    }

    #[test]
    fn test_non_empty_vertex_graph_to_adj() {
        init_dcg();

        let g: Graph<_> = GraphIntro::add_edge(GraphIntro::empty(),
                                               name_pair(name_of_usize(0), name_of_usize(1)),
                                               0,
                                               1);
        let g = GraphIntro::add_edge(g, name_pair(name_of_usize(2), name_of_usize(3)), 2, 3);
        let adj_g = adjacency_of_edge_list(&g);
        assert!(!TrieElim::is_empty(&GraphElim::vertices(&adj_g)));
    }
}
