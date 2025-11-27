use graphc::graph::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

static PETERSEN_VERTICES: u8 = 10;
static PETERSEN_EDGES: &[(u8, u8)] = &[
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 0),
    (5, 8),
    (5, 9),
    (6, 7),
    (6, 9),
    (7, 8),
    (0, 5),
    (1, 6),
    (2, 8),
    (3, 9),
    (4, 7),
];

static MORITA_RANK4_VERTICES: u8 = 6;
static MORITA_RANK4_EDGES: &[(u8, u8)] = &[
    (0, 1),
    (0, 3),
    (0, 5),
    (1, 4),
    (1, 5),
    (2, 3),
    (2, 4),
    (2, 5),
    (3, 4),
];

static BENZEN_RANK4_VERTICES: u8 = 6;
static BENZEN_RANK4_EDGES: &[(u8, u8)] = &[
    (0, 1),
    (0, 1),
    (0, 4),
    (1, 5),
    (2, 3),
    (2, 4),
    (2, 4),
    (3, 5),
    (3, 5),
];

static BENZEN_LOOP_RANK4_VERTICES: u8 = 5;
static BENZEN_LOOP_RANK4_EDGES: &[(u8, u8)] = &[
    (0, 0),
    (0, 3),
    (0, 4),
    (1, 2),
    (1, 3),
    (1, 3),
    (2, 4),
    (2, 4),
];

// subforests function
#[test]
fn test_subforest_petersen() {
    let graph: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into());

    let subforests_number_size1: usize = graph.subforests(1, 1).len();
    let expected_subforests_number_size1: usize = 15;
    assert_eq!(subforests_number_size1, expected_subforests_number_size1);

    let subforests_number_size3: usize = graph.subforests(3, 3).len();
    let expected_subforests_number_size3: usize = 2730 / 6;
    assert_eq!(subforests_number_size3, expected_subforests_number_size3);
}

#[test]
fn test_subforest_morita_rank4() {
    let graph: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into());

    let subforests_number_size1: usize = graph.subforests(1, 1).len();
    let expected_subforests_number_size1: usize = 9;
    assert_eq!(subforests_number_size1, expected_subforests_number_size1);

    let subforests_number_size3: usize = graph.subforests(3, 3).len();
    let expected_subforests_number_size3: usize = 82; // 9*8*7/6-2
    assert_eq!(subforests_number_size3, expected_subforests_number_size3);
}

#[test]
fn test_subforest_benzen_rank4() {
    let graph: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into());

    let subforests_number_size1: usize = graph.subforests(1, 1).len();
    let expected_subforests_number_size1: usize = 9;
    assert_eq!(subforests_number_size1, expected_subforests_number_size1);

    let subforests_number_size2: usize = graph.subforests(2, 2).len();
    let expected_subforests_number_size2: usize = 33; // 9*8/2-3
    assert_eq!(subforests_number_size2, expected_subforests_number_size2);
}

// contract_edge

#[test]
fn test_contract_edge_petersen() {
    let edges = vec![
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 0),
        (5, 8),
        (6, 7),
        (6, 5),
        (7, 8),
        (0, 5),
        (1, 6),
        (2, 8),
        (3, 5),
        (4, 7),
    ];

    let contractedpetersen: Graph = Graph::new(9, edges);
    let g: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into());
    let gg: Graph = g.contract_edge((5, 9));

    assert!(gg.num_vertices > 0);
    assert!(!gg.contains_loop());

    let mut a = contractedpetersen.edges.clone();
    a.sort_unstable();
    let mut b = gg.edges.clone();
    b.sort_unstable();

    assert_eq!(a, b);

    assert!(contractedpetersen.is_isomorphic_to(&gg));
}

#[test]
fn test_contract_edge_morita_rank4() {
    let edges = vec![
        (0, 1),
        (0, 3),
        (0, 1),
        (1, 4),
        (2, 3),
        (2, 4),
        (2, 1),
        (3, 4),
    ];

    let contractedmorita: Graph = Graph::new(5, edges);
    let g: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into());
    let gg: Graph = g.contract_edge((1, 5));

    assert!(gg.num_vertices > 0);
    assert!(!gg.contains_loop());

    let mut a = contractedmorita.edges.clone();
    a.sort_unstable();
    let mut b = gg.edges.clone();
    b.sort_unstable();

    assert_eq!(a, b);

    assert!(contractedmorita.is_isomorphic_to(&gg));
}

#[test]
fn test_contract_edge_benzen_rank4() {
    let edges: Vec<(u8, u8)> = vec![
        (0, 1),
        (0, 1),
        (0, 3),
        (1, 4),
        (2, 3),
        (2, 3),
        (2, 4),
        (2, 4),
    ];
    let contractedbenzen: Graph = Graph::new(5, edges);
    let g: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into());
    let gg: Graph = g.contract_edge((2, 3));

    assert!(gg.num_vertices > 0);
    assert!(!gg.contains_loop());

    let mut a = contractedbenzen.edges.clone();
    a.sort_unstable();
    let mut b = gg.edges.clone();
    b.sort_unstable();

    assert_eq!(a, b);

    assert!(contractedbenzen.is_isomorphic_to(&gg));
}

// contract_neighborhood function

#[test]
fn test_contract_neighborhood_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into());
    let contrpetersen: Graph = petersen.contract_edge((5, 9));
    let contrpetersenbipart: Graph = contrpetersen.to_bipartite();
    let petersenbipart: Graph = petersen.to_bipartite().contract_neighborhood(10).0;

    for ed in petersen.edges.iter() {
        let gg: Graph = petersen.contract_edge(*ed).to_bipartite();
        for i in 10..24 {
            let g: Graph = petersen.to_bipartite().contract_neighborhood(i).0;
            assert!(g.is_isomorphic_to(&gg));
        }
    }

    assert!(petersenbipart.is_isomorphic_to(&contrpetersenbipart));
}

#[test]
fn test_contract_neighborhood_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into());
    let contrmorita: Graph = morita.contract_edge((2, 3));
    let contrmoritabipart: Graph = contrmorita.to_bipartite();
    let moritabipart: Graph = morita.to_bipartite().contract_neighborhood(10).0;

    assert!(moritabipart.is_isomorphic_to(&contrmoritabipart));
}

#[test]
fn test_contract_neighborhood_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into());
    let contrbenzen: Graph = benzen.contract_edge((2, 3));
    let contrbenzenbipart: Graph = contrbenzen.to_bipartite();
    let benzenbipart: Graph = benzen.to_bipartite().contract_neighborhood(10).0;

    assert_eq!(benzenbipart.edges, contrbenzenbipart.edges);
    assert_eq!(benzenbipart.adj, contrbenzenbipart.adj);

    let benzenloop: Graph = benzen.to_bipartite().contract_neighborhood(6).0;
    let bloop: Graph =
        Graph::new(BENZEN_LOOP_RANK4_VERTICES, BENZEN_LOOP_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(benzenloop.edges, bloop.edges);
    assert_eq!(benzenloop.adj, bloop.adj);

    assert!(benzenbipart.is_isomorphic_to(&contrbenzenbipart));
}

// count_double_edges function

#[test]
fn test_count_double_edges_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert_eq!(petersen.count_double_edges(), 0);
}

#[test]
fn test_count_double_edges_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(morita.count_double_edges(), 0);
}

#[test]
fn test_count_double_edges_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(benzen.count_double_edges(), 3);
}

fn count_double_edges_3edge_connected(path: &str) -> (usize, Vec<Vec<Graph>>, usize) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
        .collect::<Vec<Graph>>();
    let graphs_loaded_number = graphs.len();

    let mut graphs_split: Vec<Vec<Graph>> = vec![
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    for g in graphs.iter() {
        graphs_split[g.count_double_edges() as usize].push(g.clone());
    }

    let mut three_edge_connected_number = 0;
    for g in graphs_split[0].iter() {
        if g.is_3edge_connected() {
            three_edge_connected_number += 1;
        }
    }

    (
        graphs_loaded_number,
        graphs_split,
        three_edge_connected_number,
    )
}

//#[test]
fn test_count_double_edges_and_3edge_connected_rank7() {
    let path = "graphs/v12_e18.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected = [
        81, 105, 101, 53, 20, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 365);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    assert_eq!(three_edge_connected_number, 57);
}

//#[test]
fn test_count_double_edges_and_3edge_connected_rank8() {
    let path = "graphs/v14_e21.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected = [
        480, 777, 744, 406, 152, 36, 6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 2602);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    assert_eq!(three_edge_connected_number, 341);
}

//#[test]
fn test_count_double_edges_and_3edge_connected_rank8_excess1() {
    let path = "graphs/v13_e20.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected = [
        1881, 4330, 4573, 2761, 1028, 224, 25, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 14823);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    // assert_eq!(three_edge_connected_number, 341);

    let mut num = vec![0, 0, 0, 0];
    let expnum = vec![1483, 856, 0, 0];

    for i in 0..4 {
        for g in graphs_split[i].iter() {
            if g.is_3edge_connected() {
                num[i] += 1;
            }
        }
    }

    assert_eq!(num, expnum);
}

//#[test]
fn test_count_double_edges_and_3edge_connected_rank9() {
    let path = "graphs/v16_e24.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected = [
        3874, 7152, 6904, 3918, 1508, 379, 68, 7, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 23811);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    assert_eq!(three_edge_connected_number, 2828);
}

//#[test]
fn test_count_double_edges_and_3edge_connected_rank10() {
    let path = "graphs/v18_e27.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected = [
        39866, 79284, 77080, 45125, 17829, 4794, 896, 109, 9, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 264993);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    assert_eq!(three_edge_connected_number, 30468);
}

// vertices_valency2 function

#[test]
fn test_vertices_valency2_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert_eq!(petersen.vertices_valency2().len(), 15);
}

#[test]
fn test_vertices_valency2_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(morita.vertices_valency2().len(), 9);
}

#[test]
fn test_vertices_valency2_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(benzen.vertices_valency2().len(), 9);
}

// connected_components function

#[test]
fn test_connected_components_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert_eq!(petersen.connected_components().0, 1);
}

#[test]
fn test_connected_components_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(morita.connected_components().0, 1);
}

#[test]
fn test_connected_components_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(benzen.connected_components().0, 1);
}

#[test]
fn test_connected_components_4components() {
    let edges = vec![(1, 4), (0, 4), (4, 1), (2, 3), (6, 7)];
    let g = Graph::new(8, edges);
    assert_eq!(g.connected_components().0, 4);
}

#[test]
fn test_connected_components_4components_biggraph() {
    let edges = vec![(1, 4), (0, 4), (4, 1), (2, 3), (6, 7)];
    let g = BigGraph::new(8, edges);
    assert_eq!(g.connected_components().0, 4);
}

// test is_3edge_connected

#[test]
fn test_is_3edge_connected_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert!(petersen.is_3edge_connected());
    assert!(petersen.is_k_edge_connected(3, true));
}

#[test]
fn test_is_3edge_connected_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert!(morita.is_3edge_connected());
    assert!(morita.is_k_edge_connected(3, true));
}

#[test]
fn test_is_3edge_connected_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert!(!benzen.is_3edge_connected());
    assert!(!benzen.is_k_edge_connected(3, true));
}

#[test]
fn test_find_ktuples() {
    let arr = &[1, 2, 3, 4, 5, 6];
    let res = &[
        [1, 2, 3],
        [1, 2, 4],
        [1, 2, 5],
        [1, 2, 6],
        [1, 3, 4],
        [1, 3, 5],
        [1, 3, 6],
        [1, 4, 5],
        [1, 4, 6],
        [1, 5, 6],
        [2, 3, 4],
        [2, 3, 5],
        [2, 3, 6],
        [2, 4, 5],
        [2, 4, 6],
        [2, 5, 6],
        [3, 4, 5],
        [3, 4, 6],
        [3, 5, 6],
        [4, 5, 6],
    ];
    assert_eq!(get_ktuples(arr, 3), res);
    assert_eq!(get_ktuples(arr, 6), [arr]);
}
