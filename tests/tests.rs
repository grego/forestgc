use graphc::forested_graph::ForestedGraph;
use graphc::graph::*;
use rayon::prelude::*;
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

//#[test]
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

fn compute_girth_filtration(path: &str) -> (usize, Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
        .collect::<Vec<Graph>>();
    let graphs_loaded_number = graphs.len();

    let (matr, loops) = graphs
        .par_iter()
        .filter(|g| g.is_3edge_connected())
        .map(|g| {
            let mut mat = vec![vec![0; 20]; 20];
            let mut loops = vec![vec![0; 20]; 20];
            for q in 1..20 {
                let forested_graphs = ForestedGraph::new(g, q, false);
                for f in forested_graphs.subforests().iter() {
                    let ddd: Vec<u8> = BitPositions(*f).map(|a| a as u8).collect();
                    let fff = forested_graphs
                        .graph()
                        .contract_multiple_neighborhoods(&ddd);
                    let p = fff.girth() as usize;
                    mat[p][q] += 1;
                    if p == 1 {
                        loops[fff.vertices_valency(1, 1).len()][q] += 1;
                    }
                }
            }
            (mat, loops)
        })
        .reduce(
            || (vec![vec![0; 20]; 20], vec![vec![0; 20]; 20]),
            |(a, b), (k, l)| {
                let mut mat = vec![vec![0; 20]; 20];
                let mut loops = vec![vec![0; 20]; 20];
                for i in 0..20 {
                    for j in 0..20 {
                        mat[i][j] = a[i][j] + k[i][j];
                        loops[i][j] = b[i][j] + l[i][j];
                    }
                }
                (mat, loops)
            },
        );

    (graphs_loaded_number, matr, loops)
}

//#[test]
fn test_girth_filtration_rank6_non_3_edge_connected() {
    let path = "graphs/v10_e15.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

    let expmatr = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 58, 675, 3637, 10953, 20038, 22915, 16049, 5902, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 313, 1252, 3824, 7942, 9783, 6336, 1694, 136, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 54, 121, 198, 129, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    // An older version for all rank 6 graphs, not just 3-edge connected

    assert_eq!(graphs_loaded_number, 66);
    assert_eq!(matr, expmatr);
}

//#[test]
fn test_girth_filtration_rank6() {
    let path = "graphs/v10_e15.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

    let mut vertaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut vertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..20 {
        for j in 0..20 {
            vertaddmatr[j] += matr[i][j];
            horzaddmatr[i] += matr[i][j];

            vertaddloops[j] += loops[i][j];
            horzaddloops[i] += loops[i][j];
        }
    }

    let expmatr = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 8, 135, 957, 3694, 8555, 11981, 9997, 4271, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 15, 139, 751, 2341, 3793, 2972, 911, 74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 43, 110, 189, 125, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddmatr = [
        0, 64, 258, 1075, 3423, 7492, 11527, 12892, 10071, 4271, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let exphorzaddmatr = [
        0, 39598, 10996, 472, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let exploops = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 8, 135, 911, 3146, 5461, 3386, 410, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 46, 548, 2884, 5430, 1366, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 210, 3165, 2695, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 5526, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 4271, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddloops = [
        0, 0, 8, 135, 957, 3694, 8555, 11981, 9997, 4271, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let exphorzaddloops = [
        0, 13457, 10274, 6070, 5526, 0, 4271, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(graphs_loaded_number, 66);
    assert_eq!(matr, expmatr);
    assert_eq!(loops, exploops);

    assert_eq!(vertaddmatr, expvertaddmatr);
    assert_eq!(horzaddmatr, exphorzaddmatr);
    assert_eq!(vertaddloops, expvertaddloops);
    assert_eq!(horzaddloops, exphorzaddloops);
}

//#[test]
fn test_girth_filtration_rank7() {
    let path = "graphs/v12_e18.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

    let mut vertaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut vertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..20 {
        for j in 0..20 {
            vertaddmatr[j] += matr[i][j];
            horzaddmatr[i] += matr[i][j];

            vertaddloops[j] += loops[i][j];
            horzaddloops[i] += loops[i][j];
        }
    }

    let expmatr = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 73, 1329, 10883, 53102, 173389, 395591, 624127, 655197, 435819, 151001, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        [
            0, 100, 1339, 9482, 40774, 108333, 177022, 174575, 93264, 20604, 985, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 328, 1642, 5252, 9128, 6583, 922, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 54, 62, 24, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddmatr = [
        0, 482, 3116, 16087, 60789, 168018, 351333, 570166, 717391, 675801, 436804, 151001, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];
    let exphorzaddmatr = [
        0, 2500511, 626478, 23855, 144, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let exploops = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 73, 1329, 10603, 48896, 142640, 258515, 248620, 91965, 7793, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ],
        [
            0, 0, 0, 0, 280, 4206, 29771, 121500, 262213, 191832, 27827, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 978, 15576, 106419, 246027, 62526, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 6875, 125373, 113705, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 223968, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 151001, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddloops = [
        0, 0, 73, 1329, 10883, 53102, 173389, 395591, 624127, 655197, 435819, 151001, 0, 0, 0, 0,
        0, 0, 0, 0,
    ];
    let exphorzaddloops = [
        0, 810434, 637629, 431526, 245953, 223968, 0, 151001, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(graphs_loaded_number, 365);
    assert_eq!(matr, expmatr);
    assert_eq!(loops, exploops);

    assert_eq!(vertaddmatr, expvertaddmatr);
    assert_eq!(horzaddmatr, exphorzaddmatr);
    assert_eq!(vertaddloops, expvertaddloops);
    assert_eq!(horzaddloops, exphorzaddloops);
}

//#[test]
fn test_girth_filtration_rank8() {
    let path = "graphs/v14_e21.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

    let mut vertaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut vertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..20 {
        for j in 0..20 {
            vertaddmatr[j] += matr[i][j];
            horzaddmatr[i] += matr[i][j];

            vertaddloops[j] += loops[i][j];
            horzaddloops[i] += loops[i][j];
        }
    }

    let expmatr = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 731, 14945, 142049, 835890, 3418102, 10244875, 22920570, 38040944, 45828605,
            39017410, 22210390, 6737887, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 856, 14821, 128841, 693694, 2486317, 6096636, 10232435, 11529388, 8203582, 3188522,
            522642, 19636, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 3231, 23485, 106045, 291789, 465466, 378029, 109508, 3660, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            0, 593, 1916, 3078, 1776, 272, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddmatr = [
        0, 4686, 40953, 252909, 1129308, 3787945, 9892784, 20586818, 34453618, 46244526, 49017127,
        39540052, 22230026, 6737887, 0, 0, 0, 0, 0, 0,
    ];
    let exphorzaddmatr = [
        0, 189412398, 43117370, 1381213, 7652, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let exploops = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let exphorzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    assert_eq!(graphs_loaded_number, 2602);
    assert_eq!(matr, expmatr);
    assert_eq!(loops, exploops);

    assert_eq!(vertaddmatr, expvertaddmatr);
    assert_eq!(horzaddmatr, exphorzaddmatr);
    assert_eq!(vertaddloops, expvertaddloops);
    assert_eq!(horzaddloops, exphorzaddloops);
}

//#[test]
fn test_girth_filtration_rank9() {
    let path = "graphs/v16_e24.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

    let mut vertaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut vertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..20 {
        for j in 0..20 {
            vertaddmatr[j] += matr[i][j];
            horzaddmatr[i] += matr[i][j];

            vertaddloops[j] += loops[i][j];
            horzaddloops[i] += loops[i][j];
        }
    }

    let expmatr = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 8, 135, 957, 3694, 8555, 11981, 9997, 4271, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 15, 139, 751, 2341, 3793, 2972, 911, 74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 43, 110, 189, 125, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let exphorzaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let exploops = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let exphorzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    assert_eq!(graphs_loaded_number, 23811);
    assert_eq!(matr, expmatr);
    assert_eq!(loops, exploops);

    assert_eq!(vertaddmatr, expvertaddmatr);
    assert_eq!(horzaddmatr, exphorzaddmatr);
    assert_eq!(vertaddloops, expvertaddloops);
    assert_eq!(horzaddloops, exphorzaddloops);
}

//#[test]
fn test_girth_filtration_rank10() {
    let path = "graphs/v18_e27.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

    let mut vertaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut vertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut horzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..20 {
        for j in 0..20 {
            vertaddmatr[j] += matr[i][j];
            horzaddmatr[i] += matr[i][j];

            vertaddloops[j] += loops[i][j];
            horzaddloops[i] += loops[i][j];
        }
    }

    let expmatr = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 8, 135, 957, 3694, 8555, 11981, 9997, 4271, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 15, 139, 751, 2341, 3793, 2972, 911, 74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 43, 110, 189, 125, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let exphorzaddmatr = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let exploops = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let expvertaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let exphorzaddloops = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    assert_eq!(graphs_loaded_number, 264993);
    assert_eq!(matr, expmatr);
    assert_eq!(loops, exploops);

    assert_eq!(vertaddmatr, expvertaddmatr);
    assert_eq!(horzaddmatr, exphorzaddmatr);
    assert_eq!(vertaddloops, expvertaddloops);
    assert_eq!(horzaddloops, exphorzaddloops);
}

fn count_double_edges_3edge_connected(path: &str) -> (usize, Vec<Vec<Graph>>, usize) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
        .collect::<Vec<Graph>>();
    let graphs_loaded_number = graphs.len();

    let mut graphs_split: Vec<Vec<Graph>> = vec![Vec::new(); 20];
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
fn test_count_double_edges_and_3edge_connected_rank4() {
    let path = "graphs/v6_e9.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected =
        [2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 5);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    assert_eq!(three_edge_connected_number, 2);
}

//#[test]
fn test_count_double_edges_and_3edge_connected_rank5() {
    let path = "graphs/v8_e12.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected =
        [5, 4, 4, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 16);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    assert_eq!(three_edge_connected_number, 4);
}

//#[test]
fn test_count_double_edges_and_3edge_connected_rank6() {
    let path = "graphs/v10_e15.g6";

    let (graphs_loaded_number, graphs_split, three_edge_connected_number) =
        count_double_edges_3edge_connected(path);

    let graphs_split_numbers_expected = [
        18, 18, 17, 9, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut split_numbers = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    for i in 0..10 {
        split_numbers[i] = graphs_split[i].len();
    }

    assert_eq!(graphs_loaded_number, 66);
    assert_eq!(split_numbers, graphs_split_numbers_expected);
    assert_eq!(three_edge_connected_number, 14);
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

    //This numbers were obtained before graphs of excess 1 containing loops were added

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
