#![feature(test)]
extern crate adapton;
extern crate test;
use self::test::Bencher;
use adapton::collections::graph::*;
use adapton::engine::*;

mod graph_add_edge {
    use super::*;

    fn doit<G: GraphIntro<usize> + GraphElim<usize>>(g: G, (i, j): (usize, usize)) -> G {
        ns(name_of_str("add_edge"),
           || G::add_edge(g, name_pair(name_of_usize(i), name_of_usize(j)), i, j))
    }

    fn run_bench<G: GraphIntro<usize> + GraphElim<usize>, GraphBuilder>(b: &mut Bencher,
                                                                        builder: GraphBuilder)
        where GraphBuilder: Fn() -> G
    {
        let mut input = builder();

        let max = 100;

        for i in (1..max / 2 - 1).into_iter() {
            let j = max - i;

            b.iter(|| doit(input.clone(), (i, j)));
            input = GraphIntro::add_edge(input, name_pair(name_of_usize(i), name_of_usize(j)), i, j)
        }
    }

    #[bench]
    fn benchmark_naive_graph(b: &mut Bencher) {
        init_naive();
        run_bench(b, Graph::empty);
    }

    #[bench]
    fn benchmark_dcg_graph(b: &mut Bencher) {
        init_dcg();
        run_bench(b, Graph::empty);
    }

    #[bench]
    fn benchmark_naive_adj_graph(b: &mut Bencher) {
        init_naive();
        run_bench(b, AdjacencyGraph::empty);
    }

    #[bench]
    fn benchmark_dcg_adj_graph(b: &mut Bencher) {
        init_dcg();
        run_bench(b, AdjacencyGraph::empty);
    }
}

mod graph_conversion {
    use super::*;

    fn convert_to_adj(g: &Graph<usize>) -> AdjacencyGraph<usize> {
        ns(name_of_str("convert_to_adj"), || adjacency_of_edge_list(g))
    }

    fn run_graph_bench(b: &mut Bencher) {
        let mut input = Graph::empty();

        let max = 100;

        for i in (1..max / 2 - 1).into_iter() {
            let j = max - i;
            input = GraphIntro::add_edge(input, name_pair(name_of_usize(i), name_of_usize(j)), i, j);

            b.iter(|| convert_to_adj(&input));
        }
    }

    #[bench]
    fn benchmark_naive_graph_to_adj(b: &mut Bencher) {
        init_naive();
        run_graph_bench(b);
    }

    #[bench]
    fn benchmark_dcg_graph_to_adj(b: &mut Bencher) {
        init_dcg();
        run_graph_bench(b);
    }
}
