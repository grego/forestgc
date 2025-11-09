use graphc::graph::*;

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

// contract_binary_neighbourhood function

#[test]
fn test_contract_binary_neighborhood_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into());
    let contrpetersen: Graph = petersen.contract_edge((5, 9));
    let contrpetersenbipart: Graph = contrpetersen.to_bipartite();
    let mut petersenbipart: Graph = petersen.to_bipartite();
    petersenbipart.contract_binary_neighborhood(10);

    for ed in PETERSEN_EDGES {
        let gg: Graph = petersen.contract_edge(*ed).to_bipartite();
        for i in 10..24 {
            let mut g: Graph = petersen.to_bipartite();
            g.contract_binary_neighborhood(i);
            // let g1 = g.edges.clone().sort_unstable();
            // let g2 = gg.edges.clone().sort_unstable();
            //  assert!(g1 != g2);
            assert!(g.is_isomorphic_to(&gg));
        }
    }

    // assert_eq!(petersenbipart.edges, contrpetersenbipart.edges);
    // assert_eq!(petersenbipart.adj, contrpetersenbipart.adj);

    assert!(petersenbipart.is_isomorphic_to(&contrpetersenbipart));
}

#[test]
fn test_contract_binary_neighborhood_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into());
    let contrmorita: Graph = morita.contract_edge((2, 3));
    let contrmoritabipart: Graph = contrmorita.to_bipartite();
    let mut moritabipart: Graph = morita.to_bipartite();
    moritabipart.contract_binary_neighborhood(10);

    //  assert_eq!(moritabipart.edges, contrmoritabipart.edges);
    // assert_eq!(moritabipart.adj, contrmoritabipart.adj);

    assert!(moritabipart.is_isomorphic_to(&contrmoritabipart));
}

#[test]
fn test_contract_binary_neighborhood_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into());
    let contrbenzen: Graph = benzen.contract_edge((2, 3));
    let contrbenzenbipart: Graph = contrbenzen.to_bipartite();
    let mut benzenbipart: Graph = benzen.to_bipartite();
    benzenbipart.contract_binary_neighborhood(10);

    assert_eq!(benzenbipart.edges, contrbenzenbipart.edges);
    assert_eq!(benzenbipart.adj, contrbenzenbipart.adj);

    assert!(benzenbipart.is_isomorphic_to(&contrbenzenbipart));
}

#[test]
fn test_delete_vertex_from_mask() {
    let mask = 0b10011;
    assert_eq!(delete_vertex_from_mask(mask, 2), 0b1011);
}
