use rayon::prelude::*;

use graphc::forested_graph::ForestedGraph;
use graphc::graph::{BitPositions, Graph};

use rand::Rng;
use rand::seq::SliceRandom;
use rustc_hash::{FxHashMap, FxHashSet};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use std::mem;
use std::path::Path;
use std::time::Instant;

#[allow(dead_code)]
fn random_permutation<R: Rng>(rng: &mut R, n: u8) -> Vec<u8> {
    let mut perm: Vec<u8> = (0..n).collect();
    perm.shuffle(rng);
    perm
}

fn compute_matrices(filename: &str, forest_size: usize) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let g6s = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();

    let graphs = g6s
        .iter()
        .map(|g6| Graph::from_g6(g6))
        .filter(|g| g.is_3edge_connected())
        .collect::<Vec<Graph>>();
    let n_graphs = graphs.len();
    println!("Loaded {} graphs from file {}", n_graphs, filename);
    let start = Instant::now();

    let res: Vec<_> = graphs
        .par_iter()
        .enumerate()
        .take(n_graphs)
        .map(|(_i, g)| {
            let fc = ForestedGraph::new(g, forest_size, false);
            let du = fc.d_unmark();
            let dc = fc.d_contract();
            (fc, du, dc)
        })
        .collect();

    let mut col_indices = vec![0; n_graphs];
    let mut rrow_indices = vec![0; n_graphs];
    let (mut sum, mut rsum) = (0, 0);
    for (i, (fgs, du, _)) in res.iter().enumerate() {
        sum += fgs.subforests().len();
        rsum += du.smaller_forests().len();
        if i + 1 < n_graphs {
            col_indices[i + 1] = sum;
            rrow_indices[i + 1] = rsum;
        }
    }

    let mut contracted_num = 0;
    let mut contracted_graphs = FxHashMap::default();
    for (_, _, dc) in &res {
        for (g, fs) in dc.contracted_graphs() {
            let (_, cg) = contracted_graphs.entry(g.clone()).or_insert_with(|| {
                contracted_num += 1;
                (contracted_num - 1, FxHashSet::default())
            });
            for i in fs {
                cg.insert(i);
            }
        }
    }
    let mut contracted: Vec<Vec<u64>> = vec![Vec::new(); contracted_num];
    for (i, forests) in contracted_graphs.values_mut() {
        let mut forests = mem::take(forests);
        contracted[*i].extend(forests.drain());
        contracted[*i].sort_unstable();
    }

    let mut crow_indices = vec![0; contracted_num];
    let mut csum = 0;
    for i in 0..contracted_num {
        csum += contracted[i].len();
        if i + 1 < contracted_num {
            crow_indices[i + 1] = csum;
        }
    }

    let total_time = start.elapsed();
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
    println!("dc differential rows: {}", csum);
    println!("Number of contracted graphs: {}", contracted_num);

    let stem: &str = Path::new(filename)
        .file_stem()
        .and_then(|s| s.try_into().ok())
        .unwrap();
    let mf = File::create(format!("tmatrices/{stem}_f{forest_size}.sms")).unwrap();
    let mut mf = BufWriter::new(mf);
    writeln!(mf, "{} {} M", rsum + csum, sum).unwrap();
    let mut column = 0;
    for (i, (_, du, _)) in res.iter().enumerate() {
        for entries in du.columns() {
            for (m, s) in entries {
                let j = du.smaller_forests().binary_search(m).unwrap() + rrow_indices[i];
                writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
            }
            column += 1;
        }
    }
    let mut column = 0;
    for (_, _, dc) in res.iter() {
        for entries in dc.columns() {
            for ((i, m), s) in entries {
                let (g, _) = &dc.contracted_graphs()[*i];
                let &(index, _) = &contracted_graphs.get(g.as_str()).unwrap();
                let j = contracted[*index].binary_search(m).unwrap() + crow_indices[*index] + rsum;
                writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
            }
            column += 1;
        }
    }
    writeln!(mf, "0 0 0").unwrap();

    let mf = File::create(format!("tmatrices/{stem}_f{forest_size}.register")).unwrap();
    let mut mf = BufWriter::new(mf);
    for (i, (fg, du, _)) in res.iter().enumerate() {
        let (g, _, _) = graphs[i].canonical_label();
        writeln!(mf, "Graph: {}", g.to_g6()).unwrap();
        let (_gs, reg) = g.simplify(false);
        for (k, &fg) in fg.subforests().iter().enumerate() {
            write!(mf, "Column {}: ", k + 1).unwrap();
            for i in BitPositions(fg) {
                if let Some(((u, v), _)) = reg.iter().find(|(_, j)| i == *j as usize) {
                    write!(mf, "({u} {v}) ").unwrap();
                }
            }
            writeln!(mf).unwrap();
        }
        writeln!(mf).unwrap();
        for (k, &fg) in du.smaller_forests().iter().enumerate() {
            write!(mf, "dr row {}: ", k + 1).unwrap();
            for i in BitPositions(fg) {
                if let Some(((u, v), _)) = reg.iter().find(|(_, j)| i == *j as usize) {
                    write!(mf, "({u} {v}) ").unwrap();
                }
            }
            writeln!(mf).unwrap();
        }
        writeln!(mf).unwrap();
    }

    for g in contracted_graphs.iter() {
        let &(index, _) = &contracted_graphs.get(g.0.as_str()).unwrap();
        let graph = Graph::from_g6(g.0.as_str());
        let (_gs, reg) = graph.simplify(true);
        writeln!(mf, "Contracted graph: {}", g.0).unwrap();
        for (i, m) in contracted[*index].iter().enumerate() {
            let j = i + crow_indices[*index] + rsum;
            write!(mf, "dc row {}: ", j + 1).unwrap();
            for k in BitPositions(*m) {
                if let Some(((u, v), _)) = reg.iter().find(|(_, j)| k == *j as usize) {
                    write!(mf, "({u} {v}) ").unwrap();
                } else {
                    println!("{} {m:b} {k} wrong!", &g.0);
                    println!("{}", graph.to_dot());
                }
            }
            writeln!(mf).unwrap();
        }
        writeln!(mf).unwrap();
    }
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
