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

    assert!(benzenbipart.is_isomorphic_to(&contrbenzenbipart));
}

#[test]
fn test_is_multigraph_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert!(!petersen.is_multigraph());
}

#[test]
fn test_is_multigraph_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert!(!morita.is_multigraph());
}

#[test]
fn test_is_multigraph_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert!(benzen.is_multigraph());
}

#[test]
fn test_is_multigraph1() {
    let path = "graphs/v12_e18.g6";

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
        .collect::<Vec<Graph>>();
    let n_graphs = graphs.len();

    assert_eq!(n_graphs, 365);

    let mut i = 0;
    for g in graphs.iter() {
        if g.is_multigraph() {
            i += 1;
        }
    }

    assert_eq!(i, 284);
}

#[test]
fn test_is_multigraph2() {
    let path = "graphs/v14_e21.g6";

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
        .collect::<Vec<Graph>>();
    let n_graphs = graphs.len();

    assert_eq!(n_graphs, 2602);

    let mut i = 0;
    for g in graphs.iter() {
        if g.is_multigraph() {
            i += 1;
        }
    }

    assert_eq!(i, 2122);
}
