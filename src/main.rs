pub mod graph;

use rayon::prelude::*;

use graph::{BitPositions, Graph};

use rand::Rng;
use rand::seq::SliceRandom;
use rustc_hash::{FxHashMap, FxHashSet};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use crate::graph::{compose, permute_mask, sign_subset};

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

fn compute_matrices(filename: &str, forest_size: usize) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
        .collect::<Vec<Graph>>();
    let n_graphs = graphs.len();
    println!("Loaded {} graphs from file {}", n_graphs, filename);
    let start_total = Instant::now();

    let res: Vec<_> = graphs
        .iter()
        .enumerate()
        .take(n_graphs)
        .map(|(_i, g)| {
            let (can, _, perms) = g.canonical_label();

            let (gs, dict) = can.simplify(false);
            let subforests = gs.subforests(forest_size, forest_size);
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

            let mut contracted_graphs = vec![None; can.num_vertices as usize];
            let mut contracted_forests = vec![FxHashSet::default(); can.num_vertices as usize];
            let mut ccols = Vec::new();
            for &mask in &forested_graphs {
                let mut dr = Vec::new();
                let mut sign = 1;
                for i in BitPositions(mask) {
                    let (_, to_canon, perms) = contracted_graphs[i].get_or_insert_with(|| {
                        let (g, m) = can.contract_neighborhood(i as u8);
                        let (g, base, perms) = g.canonical_label();
                        (g, compose(&m, &base), perms)
                    });
                    let m = permute_mask(mask & !(1 << i), to_canon);
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

    let mut row_indices = vec![0; n_graphs];
    let mut rcol_indices = vec![0; n_graphs];
    let (mut sum, mut rsum) = (0, 0);
    for (i, (fgs, rfgs, _, _, _, _)) in res.iter().enumerate() {
        sum += fgs.len();
        rsum += rfgs.len();
        if i + 1 < n_graphs {
            row_indices[i + 1] = sum;
            rcol_indices[i + 1] = rsum;
        }
    }

    let mut contracted_num = 0;
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
            let (_, cg) = contracted_graphs.entry(g.clone()).or_insert_with(|| {
                contracted_num += 1;
                (contracted_num - 1, FxHashSet::default())
            });
            for i in fs {
                cg.insert(i);
            }
        }
    }

    let mut ccol_indices = vec![0; contracted_num];
    for (i, cg) in contracted_graphs.values() {
        ccol_indices[*i] = cg.len();
    }

    let total_time = start_total.elapsed();
    println!("---");
    println!("Number of graphs: {}", n_graphs);
    // println!("Mismatches: {}", mismatches);
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!(
        "Avg per graph: {:.3} ms",
        total_time.as_secs_f64() * 1e3 / n_graphs as f64
    );
    println!("Pairs graph + subforest up to iso: {}", sum);
    println!("du differential rows: {}", rsum);
    println!(
        "dc differential rows: {}",
        ccol_indices.iter().copied().sum::<usize>()
    );
    println!("Number of contracted graphs: {}", contracted_num);
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(graphfile) = args.next() else {
        eprintln!("no .g6 file provided; exiting");
        return;
    };
    let num_forests: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);
    compute_matrices(&graphfile, num_forests);
}
