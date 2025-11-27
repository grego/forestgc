use rayon::prelude::*;

use graphc::forested_graph::{ForestedGraph, GraphTable};
use graphc::graph::Graph;

use argh::FromArgs;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Forested graph complex computations.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
struct Args {
    /// use all graphs instead of just 3-edge connected
    #[argh(switch, short = 'a')]
    all: bool,
    /// compute the matrices of the full graph complex instead of just trivalent graphs
    #[argh(switch, short = 'f')]
    full: bool,
    /// compute just the dimensions of the graph complex, for all excesses
    #[argh(switch, short = 'd')]
    dimensions: bool,
    /// the directory where the matrices will be output
    #[argh(option, short = 'm', default = "String::from(\"matrices\")")]
    matrix_dir: String,
    /// the rank of the forested graph complex
    #[argh(positional)]
    rank: u8,
    /// the degree of the forested graph complex
    #[argh(positional)]
    degree: Option<u8>,
}

fn read_graphfile(filename: &str, three_connected: bool) -> Vec<Graph> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let g6s = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();
    g6s.par_iter()
        .map(|g6| Graph::from_g6(g6))
        .filter(|g| !three_connected || g.is_3edge_connected())
        .collect()
}

fn read_graphs(rank: u8, min_vertices: u8, three_connected: bool) -> Vec<Graph> {
    let mut graphs = Vec::new();
    for i in min_vertices..=(2 * rank - 2) {
        let filename = format!("graphs/v{}_e{}.g6", i, i + rank - 1);
        let mut gs = read_graphfile(&filename, three_connected);
        graphs.append(&mut gs);
    }
    graphs
}

fn read_all_graphs(rank: u8, three_connected: bool) -> Vec<Vec<Graph>> {
    let mut graphs = Vec::new();
    for i in 2..=(2 * rank - 2) {
        let filename = format!("graphs/v{}_e{}.g6", i, i + rank - 1);
        let gs = read_graphfile(&filename, three_connected);
        graphs.push(gs);
    }
    graphs
}

fn compute_dimensions(graphs: &[Vec<Graph>]) -> Vec<Vec<usize>> {
    let mut res = vec![vec![1]];
    for (i, gs) in graphs.iter().enumerate() {
        let dims = gs
            .par_iter()
            .map(|g| {
                let mut dims = vec![0; i + 2];
                dims[0] += 1;
                let sfs = ForestedGraph::all(g);
                for m in sfs.subforests() {
                    dims[m.count_ones() as usize] += 1;
                }
                dims
            })
            .reduce(
                || vec![0; i + 2],
                |mut d1, d2| {
                    for i in 0..d1.len() {
                        d1[i] += d2[i];
                    }
                    d1
                },
            );
        res.push(dims);
    }
    res
}

fn euler_characteristics(dims: &[Vec<usize>]) -> Vec<isize> {
    let len = dims.len();
    let mut chars: Vec<_> = dims[len - 1].iter().map(|&d| d as isize).collect();
    let mut l = len;
    while l > 0 {
        let mut s: isize = (-1_isize).pow(l as u32);
        for i in 0..l {
            chars[l - 1] += s * dims[i + len - l][i] as isize;
            s *= -1;
        }
        l -= 1;
    }
    chars
}

fn compute_matrix(graphs: &[Graph], forest_size: u8, matrix_dir: &str, matrix_name: &str) {
    let n_graphs = graphs.len();
    println!("Loaded {n_graphs} graphs");

    let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}.sms");
    let mf = File::create(&filename).unwrap();
    let mut mf = BufWriter::new(mf);
    writeln!(mf, "{}M", " ".repeat(24)).unwrap();

    let start = Instant::now();

    let (fgs, dus): (Vec<_>, Vec<_>) = graphs
        .par_iter()
        .enumerate()
        .map(|(_i, g)| {
            let fc = ForestedGraph::new(g, forest_size as usize, false);
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
    let graph_table = GraphTable::new(
        dcs.iter()
            .flat_map(|dc| dc.contracted_graphs())
            .map(|(g, fs)| (g.as_str(), fs.as_slice())),
    );
    let csum = graph_table.size();
    println!("dc differential rows: {}", csum);
    println!("Number of contracted graphs: {}", graph_table.num_graphs());

    let mut column = 0;
    for dc in &dcs {
        for entries in dc.columns() {
            for ((i, m), s) in entries {
                let (g, _) = &dc.contracted_graphs()[*i];
                let j = graph_table.get_index(g, *m) + durows as usize;
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

    let total_time = start.elapsed();
    println!("---");
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!(
        "Avg per graph: {:.3} ms",
        total_time.as_secs_f64() * 1e3 / n_graphs as f64
    );
    println!();
}

fn compute_matrix_full(graphs: &[Graph], forest_size: u8, matrix_dir: &str, matrix_name: &str) {
    let n_graphs = graphs.len();
    let g6s: Vec<_> = graphs
        .iter()
        .map(|g| g.canonical_label().0.to_g6())
        .collect();
    println!("Loaded {n_graphs} graphs");

    let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}.sms");
    let mf = File::create(&filename).unwrap();
    let mut mf = BufWriter::new(mf);
    writeln!(mf, "{}M", " ".repeat(24)).unwrap();

    let start = Instant::now();

    let (fgs, (dus, dcs)): (Vec<_>, (Vec<_>, Vec<_>)) = graphs
        .par_iter()
        .enumerate()
        .map(|(_i, g)| {
            let fc = ForestedGraph::new(g, forest_size as usize, true);
            let du = fc.d_unmark();
            let dc = fc.d_contract();
            (fc, (du, dc))
        })
        .collect();

    let mut columns = 0;
    for fg in fgs.iter() {
        columns += fg.subforests().len() as u32;
    }
    println!("Pairs graph + subforest up to iso: {}", columns);

    let graph_table = GraphTable::new(
        g6s.iter()
            .map(|s| s.as_str())
            .zip(dus.iter().map(|du| du.smaller_forests()))
            .chain(
                dcs.iter()
                    .flat_map(|dc| dc.contracted_graphs())
                    .map(|(g, fs)| (g.as_str(), fs.as_slice())),
            ),
    );
    let csum = graph_table.size();
    println!("du + dc differential rows: {}", csum);
    println!("Number of target graphs: {}", graph_table.num_graphs());

    let mut column = 0;
    for (g6, du) in g6s.iter().zip(dus.iter()) {
        for entries in du.columns() {
            for (m, s) in entries {
                let j = graph_table.get_index(g6, *m);
                writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
            }
            column += 1;
        }
    }
    let mut column = 0;
    for dc in &dcs {
        for entries in dc.columns() {
            for ((i, m), s) in entries {
                let (g, _) = &dc.contracted_graphs()[*i];
                let j = graph_table.get_index(g, *m);
                writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
            }
            column += 1;
        }
    }
    writeln!(mf, "0 0 0").unwrap();
    drop(mf);
    let mut mf = File::options().write(true).open(&filename).unwrap();
    write!(mf, "{} {}", csum as u32, columns).unwrap();
    println!("{} written", &filename);

    let total_time = start.elapsed();
    println!("---");
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!(
        "Avg per graph: {:.3} ms",
        total_time.as_secs_f64() * 1e3 / n_graphs as f64
    );
    println!();
}

fn print_dimensions(rank: u8, three_connected: bool) {
    let dims = compute_dimensions(&read_all_graphs(rank, three_connected));
    print!("e\\p\t");
    for i in 0..(2 * rank - 2) {
        print! {"{i}\t"};
    }
    println!();
    let mut e = 2 * rank - 3;
    for ds in &dims {
        print!("{e}\t");
        for d in ds {
            print!("{d}\t")
        }
        println!();
        e -= 1;
    }
    let mut sums = vec![0; dims.len()];
    for d in &dims {
        for (i, n) in d.iter().enumerate() {
            sums[i] += n;
        }
    }
    print!("sum\t");
    for d in sums {
        print!("{d}\t")
    }
    println!();

    let ech = euler_characteristics(&dims);
    print!("rank dc\t");
    for ch in ech {
        print!("{ch}\t");
    }
    println!();
}

fn main() {
    let args: Args = argh::from_env();
    let rank = args.rank;

    if args.dimensions {
        print_dimensions(rank, !args.all);
        return;
    }

    fs::create_dir_all(&args.matrix_dir).unwrap();
    let mcf = if args.full {
        compute_matrix_full
    } else {
        compute_matrix
    };
    let min_vertices = if args.full { 2 } else { 2 * rank - 2 };
    let matrix_name = if args.full {
        format!("fr{rank}")
    } else {
        format!("r{rank}")
    };
    let graphs = read_graphs(rank, min_vertices, !args.all);
    if let Some(d) = args.degree {
        mcf(&graphs, d, &args.matrix_dir, &matrix_name);
    } else {
        for d in ((4 * rank) / 5)..(2 * rank - 2) {
            mcf(&graphs, d, &args.matrix_dir, &matrix_name);
        }
    }
}
