pub mod graph;

use rayon::prelude::*;

use graph::{BitPositions, Graph};
use graph::{compose, inverse};

use rand::Rng;
use rand::seq::SliceRandom;
use rustc_hash::{FxHashMap, FxHashSet};
use std::io::{BufRead, BufReader};
// use std::collections::HashSet;
use std::fs::{self, File};
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use crate::graph::{delete_vertex_from_mask, permute_mask, sign_subset};

#[allow(dead_code)]
fn random_permutation<R: Rng>(rng: &mut R, n: u8) -> Vec<u8> {
    let mut perm: Vec<u8> = (0..n).collect();
    perm.shuffle(rng);
    perm
}

/// Compute the canonical form of a subforest, given a list of graph automorphisms.
/// If it has an odd automorphism, return None.
/// Otherwise, return a touple `(forest, sign)` where `forest` is its representing
/// class and `sign` its sign.
#[inline]
fn canonical_subforest(mask: u64, perms: &[Vec<u8>]) -> Option<(u64, i8)> {
    let mut cm = mask;
    let mut sign = 1;
    for perm in perms {
        let m = permute_mask(mask, perm);
        if m == mask && sign_subset(perm, BitPositions(mask)) == -1 {
            return None;
        } else if m < cm {
            cm = m;
            sign = sign_subset(perm, BitPositions(mask));
        }
    }
    Some((cm, sign))
}

/// Add the value to the `(key, value)` list,
/// or push a new one if not already present.
#[inline]
fn add_or_push<T: Eq>(l: &mut Vec<(T, i8)>, k: T, v: i8) {
    for (kk, vv) in l.iter_mut() {
        if *kk == k {
            *vv += v;
            return;
        }
    }
    l.push((k, v));
}

fn test_canonical_label_file(filename: &str, max_ntests: usize) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
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

    let res: Vec<_> = graphs
        .par_iter()
        .enumerate()
        .take(n_tests)
        .map(|(i, g)| {
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
            let perms: Vec<_> = perms
                .iter()
                .skip(1)
                .map(|p| compose(&inverse(&perms[0]), p))
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
            let subforests = gs.subforests(8, 8);
            let mut forested_graphs = FxHashSet::default();
            for subf in subforests {
                let mut mask = 0_u64;
                for i in subf
                    .iter()
                    .filter_map(|e| dict.binary_search_by_key(e, |&(r, _)| r).ok())
                {
                    mask |= 1 << dict[i].1;
                }
                if let Some((csf, _)) = canonical_subforest(mask, &perms) {
                    forested_graphs.insert(csf);
                }
            }

            let mut smaller_forests = FxHashSet::default();
            let mut rcols = Vec::new();
            for &mask in &forested_graphs {
                let mut dr = Vec::new();
                let mut sign = 1;
                for i in BitPositions(mask) {
                    let m = mask & !(1 << i);
                    if let Some((f, s)) = canonical_subforest(m, &perms) {
                        smaller_forests.insert(f);
                        add_or_push(&mut dr, f, s * sign);
                    }
                    sign *= -1;
                }
                rcols.push((mask, dr));
            }

            let mut contracted_graphs = vec![None; can1.num_vertices as usize];
            let mut contracted_forests = vec![FxHashSet::default(); can1.num_vertices as usize];
            let mut ccols = Vec::new();
            for &mask in &forested_graphs {
                let mut dr = Vec::new();
                let mut sign = 1;
                for i in BitPositions(mask) {
                    let (_, to_canon, perms) = contracted_graphs[i].get_or_insert_with(|| {
                        let mut g = can1.clone();
                        g.contract_binary_neighborhood(i as u8);
                        let (can, perms) = g.canonical_labels();
                        let base = perms[0].clone();
                        (
                            can,
                            base,
                            perms
                                .iter()
                                .skip(1)
                                .map(|p| compose(&inverse(&perms[0]), p))
                                .collect::<Vec<_>>(),
                        )
                    });
                    let m = permute_mask(delete_vertex_from_mask(mask, i as u8), to_canon);
                    if let Some((f, s)) = canonical_subforest(m, perms) {
                        contracted_forests[i].insert(f);
                        add_or_push(&mut dr, (i, f), s * sign);
                    }
                    sign *= -1;
                }
                ccols.push((mask, dr));
            }

            // let mut smaller_forests: Vec<_> = smaller_forests.drain().collect();
            // smaller_forests.sort_unstable();
            // let mut mf = File::create(&format!("matrices/m{i}.sms")).unwrap();
            // writeln!(mf, "{} {} M", smaller_forests.len(), forested_graphs.len()).unwrap();
            // for (i, (_, row)) in rows.iter().enumerate() {
            //     for (m, s) in row {
            //         if let Ok(j) = smaller_forests.binary_search(m) {
            //             writeln!(mf, "{} {} {s}", j + 1, i + 1).unwrap();
            //         }
            //     }
            // }
            // writeln!(mf, "0 0 0").unwrap();
            // println!("{} {}", forested_graphs.len(), smaller_forests.len());
            // let _ = fs::write(format!("graphs/g{i}.dot"), can1.to_dot());
            // let _ = fs::write(
            //     format!("graphs/g{i}_multi.dot"),
            //     can1.to_multigraph().to_dot(),
            // );
            sf_num.fetch_add(forested_graphs.len(), std::sync::atomic::Ordering::Relaxed);

            (
                forested_graphs,
                smaller_forests,
                rcols,
                contracted_graphs,
                contracted_forests,
                ccols,
            )
        })
        .collect();

    let mut contracted_graphs = FxHashMap::default();
    for (_, _, _, cgs, cfs, _) in &res {
        let g6s: Vec<_> = cgs
            .par_iter()
            .map(|x| x.as_ref().map(|(g, _, _)| g.to_g6()))
            .collect();
        for (g, fs) in cfs
            .iter()
            .zip(g6s.iter())
            .filter_map(|(fs, g)| g.as_ref().map(|g| (g, fs)))
        {
            let cg: &mut FxHashSet<_> = contracted_graphs.entry(g.clone()).or_default();
            for i in fs {
                cg.insert(i);
            }
        }
    }

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
    test_canonical_label_file("graphs.g6", 1_000_000);
}
