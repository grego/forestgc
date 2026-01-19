// use graphc::forested_graph::ForestedGraph;
use graphc::graph::*;
// use rayon::prelude::*;
// use std::fs::File;
// use std::io::{BufRead, BufReader};

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

static TRIANGLES_VERTICES: u8 = 8;
static TRIANGLES_EDGES: &[(u8, u8)] = &[
    (0, 1),
    (0, 2),
    (0, 4),
    (1, 2),
    (1, 3),
    (2, 3),
    (3, 7),
    (4, 5),
    (4, 6),
    (5, 6),
    (5, 7),
    (6, 7),
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

// contract_multiple_neighborhoods function

#[test]
fn test_contract_multiple_neighborhoods_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into());
    let contrpetersen: Graph = petersen
        .contract_edge((6, 9))
        .contract_edge((3, 4))
        .contract_edge((1, 2))
        .to_bipartite();
    let bipartpetersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    let contrbipartpetersen: Graph =
        bipartpetersen.contract_multiple_neighborhoods(&vec![18, 13, 11]);

    assert!(contrbipartpetersen.is_isomorphic_to(&contrpetersen));
}

#[test]
fn test_contract_multiple_neighborhoods_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into());
    let contrmorita: Graph = morita
        .contract_edge((2, 5))
        .contract_edge((1, 4))
        .contract_edge((0, 3))
        .to_bipartite();
    let bipartmorita: Graph =
        Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    let contrbipartmorita: Graph = bipartmorita.contract_multiple_neighborhoods(&vec![7, 9, 13]);

    assert!(contrbipartmorita.is_isomorphic_to(&contrmorita));
}

#[test]
fn test_contract_multiple_neighborhoods_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into());
    let contrbenzen: Graph = benzen
        .contract_edge((1, 5))
        .contract_edge((0, 4))
        .contract_edge((2, 3))
        .to_bipartite();
    let bipartbenzen: Graph =
        Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    let contrbipartbenzen: Graph = bipartbenzen.contract_multiple_neighborhoods(&vec![8, 9, 10]);

    assert!(contrbipartbenzen.is_isomorphic_to(&contrbenzen));
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

// vertices_valency3 function

#[test]
fn test_vertices_valency3_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert_eq!(
        petersen.vertices_valency3().len(),
        PETERSEN_VERTICES as usize
    );
}

#[test]
fn test_vertices_valency3_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(
        morita.vertices_valency3().len(),
        MORITA_RANK4_VERTICES as usize
    );
}

#[test]
fn test_vertices_valency3_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(
        benzen.vertices_valency3().len(),
        BENZEN_RANK4_VERTICES as usize
    );
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

// is_3edge_connected function

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

// girth function

#[test]
fn test_girth_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert_eq!(petersen.girth(), 5);
}

#[test]
fn test_girth_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(morita.girth(), 3);
}

#[test]
fn test_girth_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(benzen.girth(), 2);
}

//  neighbour_vertices function

#[test]
fn test_neighbour_vertices_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert_eq!(petersen.neighbour_vertices(0), vec![1, 4, 5]);
    assert_eq!(petersen.neighbour_vertices(1), vec![0, 2, 6]);
    assert_eq!(petersen.neighbour_vertices(3), vec![2, 4, 9]);
    assert_eq!(petersen.neighbour_vertices(4), vec![0, 3, 7]);
    assert_eq!(petersen.neighbour_vertices(6), vec![1, 7, 9]);
    assert_eq!(petersen.neighbour_vertices(8), vec![2, 5, 7]);
}

#[test]
fn test_neighbour_vertices_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(morita.neighbour_vertices(0), vec![1, 3, 5]);
    assert_eq!(morita.neighbour_vertices(1), vec![0, 4, 5]);
    assert_eq!(morita.neighbour_vertices(2), vec![3, 4, 5]);
    assert_eq!(morita.neighbour_vertices(3), vec![0, 2, 4]);
    assert_eq!(morita.neighbour_vertices(4), vec![1, 2, 3]);
    assert_eq!(morita.neighbour_vertices(5), vec![0, 1, 2]);
}

#[test]
fn test_neighbour_vertices_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(benzen.neighbour_vertices(0), vec![1, 4]);
    assert_eq!(benzen.neighbour_vertices(1), vec![0, 5]);
    assert_eq!(benzen.neighbour_vertices(2), vec![3, 4]);
    assert_eq!(benzen.neighbour_vertices(3), vec![2, 5]);
    assert_eq!(benzen.neighbour_vertices(4), vec![0, 2]);
    assert_eq!(benzen.neighbour_vertices(5), vec![1, 3]);
}

// triangles function

#[test]
fn test_triangles_petersen() {
    let petersen: Graph = Graph::new(PETERSEN_VERTICES, PETERSEN_EDGES.into()).to_bipartite();
    assert_eq!(petersen.triangles(), vec![]);
}

#[test]
fn test_triangles_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(
        morita.triangles(),
        vec![
            (1 << 6) | (1 << 8) | (1 << 10),
            (1 << 11) | (1 << 12) | (1 << 14)
        ]
    );
}

#[test]
fn test_triangles_benzen_rank4() {
    let benzen: Graph = Graph::new(BENZEN_RANK4_VERTICES, BENZEN_RANK4_EDGES.into()).to_bipartite();
    assert_eq!(benzen.triangles(), vec![]);
}

#[test]
fn test_triangles_triangles() {
    let triangles: Graph = Graph::new(TRIANGLES_VERTICES, TRIANGLES_EDGES.into()).to_bipartite();
    assert_eq!(
        triangles.triangles(),
        vec![
            (1 << 11) | (1 << 12) | (1 << 13),
            (1 << 15) | (1 << 16) | (1 << 17),
            (1 << 17) | (1 << 18) | (1 << 19),
            (1 << 8) | (1 << 9) | (1 << 11)
        ]
    );
}

// has_edge_on_triangle function

#[test]
fn test_has_edge_on_triangle_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert!(!morita.has_edge_on_triangle((1 << 9) | (1 << 13) | (1 << 7))); // no edges on triangles
    assert!(morita.has_edge_on_triangle((1 << 9) | (1 << 12) | (1 << 13))); // no edge on first trinagle, one on second
    assert!(!morita.has_edge_on_triangle((1 << 14) | (1 << 11) | (1 << 13) | (1 << 7))); // no edge on first triangle, two on second
    assert!(morita.has_edge_on_triangle((1 << 6) | (1 << 7) | (1 << 13))); // one edge on first triangle, none on second
    assert!(morita.has_edge_on_triangle((1 << 9) | (1 << 10) | (1 << 12) | (1 << 13))); // one edge on first triangle, one on second
    assert!(morita.has_edge_on_triangle((1 << 12) | (1 << 11) | (1 << 9) | (1 << 8))); // one edge on first triangle, two on second
    assert!(!morita.has_edge_on_triangle((1 << 8) | (1 << 9) | (1 << 10))); // two edges on first triangle, none on second
    assert!(morita.has_edge_on_triangle((1 << 8) | (1 << 9) | (1 << 10) | (1 << 11))); // two edges on first triangle, one on second
    assert!(!morita.has_edge_on_triangle((1 << 6) | (1 << 8) | (1 << 12) | (1 << 14))); // two edges on first triangle, two on second
}

#[test]
fn test_has_edge_on_triangle_triangles() {
    let triangles: Graph = Graph::new(TRIANGLES_VERTICES, TRIANGLES_EDGES.into()).to_bipartite();
    assert!(!triangles.has_edge_on_triangle((1 << 10) | (1 << 14))); // no edges on triangles
    assert!(triangles.has_edge_on_triangle((1 << 8) | (1 << 9) | (1 << 14) | (1 << 19))); // no edges on first triangle, none on second, one on third, two on fourth
    assert!(!triangles.has_edge_on_triangle((1 << 8) | (1 << 9) | (1 << 15) | (1 << 16))); // no edges on first triangle, two on second, none on third, two on fourth
    assert!(triangles.has_edge_on_triangle((1 << 10) | (1 << 15) | (1 << 17))); // no edges on first triangle, two on second, one on third, none on fourth
    assert!(triangles.has_edge_on_triangle((1 << 9) | (1 << 15) | (1 << 17) | (1 << 18))); // no edges on first triangle, two on second, one on third, one on fourth
    assert!(triangles.has_edge_on_triangle((1 << 8) | (1 << 10) | (1 << 12) | (1 << 14))); // one edge on first triangle, none on second, none on third, one on fourth
    assert!(triangles.has_edge_on_triangle((1 << 8) | (1 << 13) | (1 << 17) | (1 << 19))); // one edge on first triangle, none on second, two on third, one on fourth
    assert!(triangles.has_edge_on_triangle((1 << 9) | (1 << 13) | (1 << 15) | (1 << 19))); // one edge on first triangle, one on second, one on third, one on fourth
    assert!(
        triangles.has_edge_on_triangle(
            (1 << 8) | (1 << 9) | (1 << 12) | (1 << 15) | (1 << 17) | (1 << 18)
        )
    ); // one edge on first triangle, one on second, two on third, two on fourth
    assert!(
        triangles.has_edge_on_triangle((1 << 12) | (1 << 15) | (1 << 17) | (1 << 18) | (1 << 19))
    ); // one edge on first triangle, two on second, two on third, none on fourth
    assert!(triangles.has_edge_on_triangle(
        (1 << 9) | (1 << 12) | (1 << 13) | (1 << 15) | (1 << 17) | (1 << 19)
    )); // two edges on first triangle, two on second, one on third, one on fourth
    assert!(!triangles.has_edge_on_triangle(
        (1 << 8) | (1 << 9) | (1 << 11) | (1 << 13) | (1 << 16) | (1 << 17) | (1 << 19)
    )); // two edges on first triangle, two on second, two on third, two on fourth
}

// has_two_edges_on_triangle function

#[test]
fn test_has_two_edges_on_triangle_morita_rank4() {
    let morita: Graph = Graph::new(MORITA_RANK4_VERTICES, MORITA_RANK4_EDGES.into()).to_bipartite();
    assert!(!morita.has_two_edges_on_triangle((1 << 9) | (1 << 13) | (1 << 7))); // no edges on triangles
    assert!(!morita.has_two_edges_on_triangle((1 << 9) | (1 << 12) | (1 << 13))); // no edge on first trinagle, one on second
    assert!(morita.has_two_edges_on_triangle((1 << 14) | (1 << 11) | (1 << 13) | (1 << 7))); // no edge on first triangle, two on second
    assert!(!morita.has_two_edges_on_triangle((1 << 6) | (1 << 7) | (1 << 13))); // one edge on first triangle, none on second
    assert!(!morita.has_two_edges_on_triangle((1 << 9) | (1 << 10) | (1 << 12) | (1 << 13))); // one edge on first triangle, one on second
    assert!(morita.has_two_edges_on_triangle((1 << 12) | (1 << 11) | (1 << 9) | (1 << 8))); // one edge on first triangle, two on second
    assert!(morita.has_two_edges_on_triangle((1 << 8) | (1 << 9) | (1 << 10))); // two edges on first triangle, none on second
    assert!(morita.has_two_edges_on_triangle((1 << 8) | (1 << 9) | (1 << 10) | (1 << 11))); // two edges on first triangle, one on second
    assert!(morita.has_two_edges_on_triangle((1 << 6) | (1 << 8) | (1 << 12) | (1 << 14))); // two edges on first triangle, two on second
}

#[test]
fn test_has_two_edges_on_triangle_triangles() {
    let triangles: Graph = Graph::new(TRIANGLES_VERTICES, TRIANGLES_EDGES.into()).to_bipartite();
    // assert!(triangles.has_two_edges_on_triangle((1 << 0) | (1 << 0)));
    assert!(!triangles.has_two_edges_on_triangle((1 << 10) | (1 << 14))); // no edges on triangles
    assert!(triangles.has_two_edges_on_triangle((1 << 8) | (1 << 9) | (1 << 14) | (1 << 19))); // no edges on first triangle, none on second, one on third, two on fourth
    assert!(triangles.has_two_edges_on_triangle((1 << 8) | (1 << 9) | (1 << 15) | (1 << 16))); // no edges on first triangle, two on second, none on third, two on fourth
    assert!(triangles.has_two_edges_on_triangle((1 << 10) | (1 << 15) | (1 << 17))); // no edges on first triangle, two on second, one on third, none on fourth
    assert!(triangles.has_two_edges_on_triangle((1 << 9) | (1 << 15) | (1 << 17) | (1 << 18))); // no edges on first triangle, two on second, one on third, one on fourth
    assert!(!triangles.has_two_edges_on_triangle((1 << 8) | (1 << 10) | (1 << 12) | (1 << 14))); // one edge on first triangle, none on second, none on third, one on fourth
    assert!(triangles.has_two_edges_on_triangle((1 << 8) | (1 << 13) | (1 << 17) | (1 << 19))); // one edge on first triangle, none on second, two on third, one on fourth
    assert!(!triangles.has_two_edges_on_triangle((1 << 9) | (1 << 13) | (1 << 15) | (1 << 19))); // one edge on first triangle, one on second, one on third, one on fourth
    assert!(triangles.has_two_edges_on_triangle(
        (1 << 8) | (1 << 9) | (1 << 12) | (1 << 15) | (1 << 17) | (1 << 18)
    )); // one edge on first triangle, one on second, two on third, two on fourth
    assert!(
        triangles
            .has_two_edges_on_triangle((1 << 12) | (1 << 15) | (1 << 17) | (1 << 18) | (1 << 19))
    ); // one edge on first triangle, two on second, two on third, none on fourth
    assert!(triangles.has_two_edges_on_triangle(
        (1 << 9) | (1 << 12) | (1 << 13) | (1 << 15) | (1 << 17) | (1 << 19)
    )); // two edges on first triangle, two on second, one on third, one on fourth
    assert!(triangles.has_two_edges_on_triangle(
        (1 << 8) | (1 << 9) | (1 << 11) | (1 << 13) | (1 << 16) | (1 << 17) | (1 << 19)
    )); // two edges on first triangle, two on second, two on third, two on fourth
}

// find-ktuples function

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

#[test]
fn test_ordered_ktuples() {
    let arr = &[1, 2, 3];
    let res = &[[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]];
    let perms = &[
        [1, 2, 3],
        [1, 3, 2],
        [2, 1, 3],
        [2, 3, 1],
        [3, 1, 2],
        [3, 2, 1],
    ];
    assert_eq!(ordered_ktuples(arr, 2), res);
    assert_eq!(ordered_ktuples(arr, 3), perms);
}
