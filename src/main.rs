use rayon::prelude::*;

use graphc::forested_graph::ForestedGraph;
use graphc::graph::Graph;

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

    let stem: &str = Path::new(filename)
        .file_stem()
        .and_then(|s| s.try_into().ok())
        .unwrap();
    let filename = format!("matrices/{stem}_f{forest_size}.sms");
    let cfilename = format!("matrices/{stem}_f{forest_size}c.sms");
    let mf = File::create(&filename).unwrap();
    let mut mf = BufWriter::new(mf);
    writeln!(mf, "{}M", " ".repeat(24)).unwrap();

    let start = Instant::now();

    let (fgs, dus): (Vec<_>, Vec<_>) = graphs
        .par_iter()
        .enumerate()
        .map(|(_i, g)| {
            let fc = ForestedGraph::new(g, forest_size, false);
            let du = fc.d_unmark();
            (fc, du)
        })
        .collect();

    let mut columns = 0;
    let mut durows = 0;
    for (fg, du) in fgs.iter().zip(dus.into_iter()) {
        for (i, j, s) in du.matrix_entries(columns, durows) {
            writeln!(mf, "{} {} {}", i + 1, j + 1, s).unwrap();
        }
        columns += fg.subforests().len() as u32;
        durows += du.smaller_forests().len() as u32;
    }
    println!("Pairs graph + subforest up to iso: {}", columns);
    println!("du differential rows: {}", durows);

    let dcs: Vec<_> = fgs.into_par_iter().map(|fg| fg.d_contract()).collect();

    let mut contracted_num = 0;
    let mut contracted_graphs = FxHashMap::default();
    for dc in dcs.iter() {
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

    println!("dc differential rows: {}", csum);
    println!("Number of contracted graphs: {}", contracted_num);

    let mut column = 0;
    for dc in &dcs {
        for entries in dc.columns() {
            for ((i, m), s) in entries {
                let (g, _) = &dc.contracted_graphs()[*i];
                let &(index, _) = &contracted_graphs.get(g.as_str()).unwrap();
                let j = contracted[*index].binary_search(m).unwrap()
                    + crow_indices[*index]
                    + durows as usize;
                writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
            }
            column += 1;
        }
    }
    writeln!(mf, "0 0 0").unwrap();
    drop(mf);
    let mut mf = File::options().write(true).open(&filename).unwrap();
    write!(mf, "{} {}", durows + csum as u32, columns).unwrap();
    println!("{} written", &filename);

    let mf = File::create(&cfilename).unwrap();
    let mut mf = BufWriter::new(mf);
    writeln!(mf, "{} {} M", csum, columns).unwrap();
    let mut column = 0;
    for dc in &dcs {
        for entries in dc.columns() {
            for ((i, m), s) in entries {
                let (g, _) = &dc.contracted_graphs()[*i];
                let &(index, _) = &contracted_graphs.get(g.as_str()).unwrap();
                let j = contracted[*index].binary_search(m).unwrap() + crow_indices[*index];
                writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
            }
            column += 1;
        }
    }
    writeln!(mf, "0 0 0").unwrap();
    println!("{} written", &cfilename);

    let total_time = start.elapsed();
    println!("---");
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!(
        "Avg per graph: {:.3} ms",
        total_time.as_secs_f64() * 1e3 / n_graphs as f64
    );
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
