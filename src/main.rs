pub mod graph;

use rayon::prelude::*;

use graph::Graph;
use graph::{compose, inverse};

use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::HashSet;
// use std::collections::HashSet;
use std::fs;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use crate::graph::sign_subset;

#[allow(dead_code)]
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
    // let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    // let mismatches = 0;
    let start_total = Instant::now();
    let with_autos = AtomicUsize::new(0);

    // let mut unique_forests = Mutex::new(HashSet::new());
    // let mut forest_edges = Vec::with_capacity(3439906022);
    let sf_num = AtomicUsize::new(0);

    graphs
        .par_iter()
        .enumerate()
        .take(n_tests)
        .for_each(|(i, g)| {
            // let g = Graph::from_g6("GAl??G");
            // println!("Graph A: {} ", g.to_g6());
            // let perm = random_permutation(&mut rng, n);
            // let g2 = g.permute(&perm);
            // let autos = g.automorphisms();
            // if autos.len() > 1 {
            //     // print!("{}", autos.len());
            //     with_autos.fetch_add(autos.len(), std::sync::atomic::Ordering::Relaxed);
            //     // if autos.len() > 2 {
            //     //     println!("{} automorphisms", autos.len());
            //     // }
            // }
            // let g2 = Graph::from_g6("GSWOO?");
            // println!("Graph B: {} ", g2.to_g6());

            // let start = Instant::now();
            let (can1, perms) = g.canonical_labels();
            let base = perms[0].clone();
            let perms: Vec<_> = perms
                .iter()
                .skip(1)
                .map(|p| compose(&inverse(&base), p))
                .collect();
            // let (can1, _) = g.canonical_label(g.initial_degree_classes());
            // println!("Canonical A: {} ", can1.to_g6());
            // let (can2, _) = g2.canonical_label();
            // let (can2, _) = g2.canonical_label(g2.initial_degree_classes());
            // let duration = start.elapsed();
            // println!("Duration: {:?} ms", duration.as_secs_f64() * 1e3);
            // println!("Canonical B: {} ", can2.to_g6());

            // println!("All: \n{}\n{}\n{}\n{}", g.to_g6(), g2.to_g6(), can1.to_g6(), can2.to_g6());

            // if can1.edges != can2.edges {
            //     mismatches += 1;
            //     println!("❌ Mismatch at test {i}");
            // }

            let (gs, dict) = can1.simplify(true);
            // println!("{}", gs.edges.len());
            // if !unique_forests.contains_key(&gs.edges) {
            // let (mut clt, mut prmt) = (0, 0);
            let subforests = gs.subforests(3, 3);
            let mut unique_perms = HashSet::new();
            // for sf in (0..can1.num_vertices)
            //     .filter(|v| can1.adj[*v as usize].count_ones() == 2)
            //     .map(|v| vec![v])
            for subf in subforests {
                // let mut g = can1.clone();
                // for e in &sf {
                //     if let Ok(v) = dict.binary_search_by_key(e, |&(r, _)| r) {
                //         g.add_unary_vertex(dict[v].1);
                //     }
                // }
                let mut sf: Vec<_> = subf
                    .iter()
                    .filter_map(|e| dict.binary_search_by_key(e, |&(r, _)| r).ok())
                    .map(|i| dict[i].1)
                    .collect();
                sf.sort_unstable();

                let mut mask = 0;
                for &i in &sf {
                    mask |= 1 << i;
                }

                // let t = Instant::now();
                // let (cang, perms) = g.canonical_labels();
                // clt += t.elapsed().as_micros();
                // let t = Instant::now();
                let mut new = true;
                for perm in &perms {
                    let mut m = 0;
                    for i in sf.iter().map(|i| perm[*i as usize]) {
                        m |= 1 << i;
                    }
                    // let iter =
                    //     (0..cang.num_vertices).filter(|i| cang.adj[*i as usize].count_ones() == 1);
                    if m == mask && sign_subset(perm, sf.iter().copied()) == -1 {
                        new = false;
                    }
                    if unique_perms.contains(&m) {
                        new = false;
                    }
                }
                if new {
                    sf_num.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    unique_perms.insert(mask);
                }
                // prmt += t.elapsed().as_micros();
                // unique_forests.lock().unwrap().insert(g.edges);
            }
            // println!(
            //     "{i}/{} graphs, {:.3}ms CL, {:.3}ms PRM",
            //     graphs.len(),
            //     clt as f32 / 1000.0,
            //     prmt as f32 / 1000.0
            // );
            // let mut edge_ranges = Vec::with_capacity(subforests.len());
            // for sf in subforests {
            //     sf_edges_num += sf.len();
            //     let start = forest_edges.len();
            //     forest_edges.extend(sf);
            //     let end = forest_edges.len();
            //     edge_ranges.push(start..end)
            // }
            // unique_forests.insert(gs.edges.clone(), edge_ranges);
            // }
            let _ = fs::write(format!("graphs/g{i}.dot"), can1.to_dot());
            let _ = fs::write(
                format!("graphs/g{i}_multi.dot"),
                can1.to_multigraph().to_dot(),
            );
            // for (j, f) in subforests.iter().enumerate() {
            //     let _ = fs::write(format!("graphs/g{i}_f{j}.dot"), f.to_dot());
            // }

            // println!("Test {i}: {:?} ms", duration.as_secs_f64() * 1e3);
        });

    let total_time = start_total.elapsed();
    println!("---");
    println!("Number of graphs: {}", n_tests);
    // println!("Mismatches: {}", mismatches);
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!(
        "Avg per graph: {:.3} ms",
        total_time.as_secs_f64() * 1e3 / n_tests as f64
    );
    println!(
        "Graphs with nontrivial automorphisms: {}",
        with_autos.into_inner()
    );
    println!("Pairs graph + subforest up to iso: {}", sf_num.into_inner());
}

fn main() {
    fs::create_dir_all("graphs").unwrap();
    // test_canonical_label_random(20);
    test_canonical_label_file("graphs.g6", 1_000_000);
    // test_canonical_label_file("data/graphs11_2.g6",1000);
    // test_canonical_label_file("data/graphs9_3.g6",1000);
}
