use graphc::forested_graph::ForestedGraph;
use graphc::graph::*;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

fn count_double_edges_and_3edge_connected_rank4() {
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

fn count_double_edges_and_3edge_connected_rank5() {
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

fn count_double_edges_and_3edge_connected_rank6() {
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

fn count_double_edges_and_3edge_connected_rank7() {
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

fn count_double_edges_and_3edge_connected_rank8() {
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

fn count_double_edges_and_3edge_connected_rank8_excess1() {
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

fn count_double_edges_and_3edge_connected_rank9() {
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

fn count_double_edges_and_3edge_connected_rank10() {
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

fn girth_filtration_rank6_non_3_edge_connected() {
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

    println!("Graphs loaded:");
    println!("{graphs_loaded_number}");
    println!("Girth filtration dimensions:");
    println!("{:?}", matr);
    println!("Loops filtration dimentsions:");
    println!("{:?}", loops);
}

fn girth_filtration_rank6() {
    let path = "graphs/v10_e15.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

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

    println!("Graphs loaded:");
    println!("{graphs_loaded_number}");
    println!("Girth filtration dimensions:");
    println!("{:?}", matr);
    println!("Loops filtration dimentsions:");
    println!("{:?}", loops);
}

fn girth_filtration_rank7() {
    let path = "graphs/v12_e18.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

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

    println!("Graphs loaded:");
    println!("{graphs_loaded_number}");
    println!("Girth filtration dimensions:");
    println!("{:?}", matr);
    println!("Loops filtration dimentsions:");
    println!("{:?}", loops);
}

fn girth_filtration_rank8() {
    let path = "graphs/v14_e21.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

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

    let exploops = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 731, 14945, 139769, 794131, 3045833, 8165754, 15035028, 17515251, 10868163,
            2765538, 169701, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 2280, 41759, 365481, 1952507, 6774025, 14684163, 16519447, 6863243, 685367,
            0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 6788, 126614, 1079587, 5257938, 13505875, 10904416, 1719570, 0, 0, 0,
            0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 31930, 583592, 4663317, 12691091, 3219749, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 271803, 5793122, 5431141, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10984862, 0, 0, 0, 0, 0, 0, 0,
        ],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6737887, 0, 0, 0, 0, 0, 0,
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
    ];

    println!("Graphs loaded:");
    println!("{graphs_loaded_number}");
    println!("Girth filtration dimensions:");
    println!("{:?}", matr);
    println!("Loops filtration dimentsions:");
    println!("{:?}", loops);
}

fn girth_filtration_rank9() {
    let path = "graphs/v16_e24.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

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

    println!("Graphs loaded:");
    println!("{graphs_loaded_number}");
    println!("Girth filtration dimensions:");
    println!("{:?}", matr);
    println!("Loops filtration dimentsions:");
    println!("{:?}", loops);
}

fn girth_filtration_rank10() {
    let path = "graphs/v18_e27.g6";

    let (graphs_loaded_number, matr, loops) = compute_girth_filtration(path);

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

    println!("Graphs loaded:");
    println!("{graphs_loaded_number}");
    println!("Girth filtration dimensions:");
    println!("{:?}", matr);
    println!("Loops filtration dimentsions:");
    println!("{:?}", loops);
}

fn main() {
    // let mut args = env::args().skip(1);
    // let Some(filename) = args.next() else {
    //     eprintln!("Usage: smssort <.sms file>");
    //     return;
    // };

    girth_filtration_rank9();
}
