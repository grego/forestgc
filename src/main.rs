pub mod graph;

use graph::Graph;

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
// use std::collections::HashSet;
use std::time::Instant;

fn random_permutation<R: Rng>(rng: &mut R, n: u8) -> Vec<u8> {
    let mut perm: Vec<u8> = (0..n).collect();
    perm.shuffle(rng);
    perm
}

fn test_canonical_label_file(filename: &str, max_ntests: usize) {
    let graphs = Graph::load_from_file(filename)
        .unwrap()
        .iter()
        .map(|g6| Graph::from_g6(g6))
        .collect::<Vec<Graph>>();
    let mut n_tests = graphs.len();
    println!("Loaded {} graphs from file {}", n_tests, filename);
    if max_ntests > 0 && n_tests > max_ntests {
        n_tests = max_ntests;
    }
    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let n = graphs[0].num_vertices;
    let mut mismatches = 0;
    let start_total = Instant::now();
    let mut with_autos = 0;

    // let mut unique_simple = HashSet::new();

    for (i, g) in graphs.iter().enumerate().take(n_tests) {
        // let g = Graph::from_g6("GAl??G");
        // println!("Graph A: {} ", g.to_g6());
        let perm = random_permutation(&mut rng, n);
        let g2 = g.permute(&perm);
        let autos = g.automorphisms();
        if autos.len() > 1 {
            // print!("{}", autos.len());
            with_autos += 1;
            // if autos.len() > 2 {
            //     println!("{} automorphisms", autos.len());
            // }
        }
        // let g2 = Graph::from_g6("GSWOO?");
        // println!("Graph B: {} ", g2.to_g6());

        // let start = Instant::now();
        let (can1, _) = g.canonical_label();
        // let (can1, _) = g.canonical_label(g.initial_degree_classes());
        // println!("Canonical A: {} ", can1.to_g6());
        let (can2, _) = g2.canonical_label();
        // let (can2, _) = g2.canonical_label(g2.initial_degree_classes());
        // let duration = start.elapsed();
        // println!("Duration: {:?} ms", duration.as_secs_f64() * 1e3);
        // println!("Canonical B: {} ", can2.to_g6());

        // println!("All: \n{}\n{}\n{}\n{}", g.to_g6(), g2.to_g6(), can1.to_g6(), can2.to_g6());

        if can1.edges != can2.edges {
            mismatches += 1;
            println!("❌ Mismatch at test {i}");
        }

        //unique_simple.insert(can1.edges);

        // println!("Test {i}: {:?} ms", duration.as_secs_f64() * 1e3);
    }

    let total_time = start_total.elapsed();
    println!("---");
    println!("Tests run: {}", n_tests);
    println!("Mismatches: {}", mismatches);
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!(
        "Avg per graph: {:.3} ms",
        total_time.as_secs_f64() * 1e3 / n_tests as f64
    );
    println!("Graphs with nontrivial automorphisms: {}", with_autos);
    // println!("Unique after simplification: {}", unique_simple.len());
}

fn main() {
    // test_canonical_label_random(20);
    test_canonical_label_file("graphs.g6", 1_000_000);
    // test_canonical_label_file("data/graphs11_2.g6",1000);
    // test_canonical_label_file("data/graphs9_3.g6",1000);
}
