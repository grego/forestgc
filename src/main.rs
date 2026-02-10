use rayon::prelude::*;

use graphc::forested_graph::{ForestedGraph, GraphTable};
use graphc::graph::{BitPositions, Graph};

use argh::FromArgs;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::io::{BufWriter, Write};
use std::path::Path;
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
    /// compute the dc differential
    #[argh(switch)]
    dc: bool,
    /// compute the du differential
    #[argh(switch)]
    du: bool,
    /// compute the registry for row and column indices in the generated matrix
    #[argh(switch)]
    registry: bool,
    /// compute the differentials for all excesses
    #[argh(switch)]
    all_excesses: bool,
    /// compute just the dimensions of the graph complex, for all excesses
    #[argh(switch, short = 'd')]
    dimensions: bool,
    /// use the odd sign convention
    #[argh(switch, short = 'o')]
    odd: bool,
    /// compute the complex with one hair
    #[argh(switch, short = 'T')]
    transpose: bool,
    ///compute the complex with graphs with at least this girth after contracting the subforest
    #[argh(option, short = 'g', default = "1")]
    girthmin: u8,
    ///compute the complex with graphs with at most this girth after contracting the subforest
    #[argh(option, short = 'G', default = "255")]
    girthmax: u8,
    /// compute the complex with the provided number of hairs
    #[argh(option, default = "0")]
    hairs: u8,
    /// the directory where the matrices will be output
    #[argh(option, short = 'm', default = "String::from(\"matrices\")")]
    matrix_dir: String,
    /// the file to load the graphs from
    #[argh(option)]
    graphfile: Option<String>,
    /// the rank of the forested graph complex
    #[argh(positional)]
    rank: u8,
    /// the degree of the forested graph complex
    #[argh(positional)]
    degree: Option<u8>,
}

fn read_graphfile(filename: &str, three_connected: bool, hairs: u8) -> Vec<Graph> {
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => panic!("Unable to open {filename}: {e}"),
    };
    let reader = BufReader::new(file);
    let g6s = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();
    g6s.par_iter()
        .map(|g6| Graph::from_g6(g6))
        .filter(|g| !three_connected || g.is_3edge_connected())
        .filter(|g| hairs == 0 || g.number_of_loops() >= hairs)
        .collect()
}

/// Read the grapsh of the specified rank with the specified number of vertices.
fn read_graphs(rank: u8, minv: u8, maxv: u8, three_connected: bool, hairs: u8) -> Vec<Graph> {
    let mut graphs = Vec::new();
    for i in minv..=maxv {
        let filename = format!("graphs/v{}_e{}.g6", i, i + rank - 1);
        let mut gs = read_graphfile(&filename, three_connected, hairs);
        graphs.append(&mut gs);
    }
    graphs
}

/// Read all graphs of the specified rank into the list by the excess of the graph.
fn read_all_graphs(rank: u8, three_connected: bool, hairs: u8) -> Vec<Vec<Graph>> {
    let mut graphs = Vec::new();
    let rank = rank + hairs;
    for i in 2..=(2 * rank - 2 - hairs) {
        let filename = format!("graphs/v{}_e{}.g6", i, i + rank - 1);
        let gs = read_graphfile(&filename, three_connected, hairs);
        graphs.push(gs);
    }
    graphs
}

fn compute_dimensions(
    graphs: &[Vec<Graph>],
    girthmin: u8,
    girthmax: u8,
    odd: bool,
    hairs: u8,
) -> Vec<Vec<usize>> {
    let mut res = vec![vec![1]];
    for (i, gs) in graphs.iter().enumerate() {
        let dims = gs
            .par_iter()
            .map(|g| {
                let mut dims = vec![0; i + 2];
                let sfs = ForestedGraph::all(g, odd, hairs);
                for fs in sfs {
                    let s = fs.filter(|f| {
                        if girthmin == 1 && girthmax == 255 {
                            return true;
                        }
                        let forest: Vec<u8> = BitPositions(f).map(|a| a as u8).collect();
                        let gr = fs.graph().contract_multiple_neighborhoods(&forest);
                        let p = gr.girth();
                        // let q = gr.vertices_valency(1, 1).len();
                        // let q = gr.count_double_edges();
                        p >= girthmin && p <= girthmax //&& q > 0
                    });
                    for m in s.subforests() {
                        dims[m.count_ones() as usize] += 1;
                    }
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

fn compute_matrix(
    graphs: &[Graph],
    (forest_size, hairs): (u8, u8),
    (matrix_dir, matrix_name): (&str, &str),
    edges_on_double: bool,
    args: &Args,
) {
    let (du, dc) = (!args.dc, !args.du);
    let (girthmin, girthmax) = (args.girthmin, args.girthmax);
    let odd = args.odd;
    let registry = args.registry;

    let n_graphs = graphs.len();
    println!("Loaded {n_graphs} graphs");

    let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}.sms");
    let mf = File::create(&filename).unwrap();
    let mut mf = BufWriter::new(mf);
    writeln!(mf, "{}M", " ".repeat(24)).unwrap();

    let start = Instant::now();

    let fgs: Vec<_> = graphs
        .par_iter()
        .flat_map(|g| {
            ForestedGraph::hairy(
                g,
                forest_size as usize,
                forest_size as usize,
                edges_on_double,
                odd,
                hairs,
            )
        })
        .map(|fg| {
            if girthmin == 1 && girthmax == 255 {
                fg
            } else {
                fg.filter(|f| {
                    let forest: Vec<u8> = BitPositions(f).map(|a| a as u8).collect();
                    let gr = fg.graph().contract_multiple_neighborhoods(&forest);
                    let p = gr.girth();
                    // let q = gr.vertices_valency(1, 1).len();
                    // let q = gr.count_double_edges();
                    p >= girthmin && p <= girthmax //&& q > 0
                })
            }
        })
        .collect();

    if registry {
        let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}.cols");
        let mf = File::create(&filename).unwrap();
        let mut mf = BufWriter::new(mf);
        for fg in fgs.iter().filter(|g| !g.subforests().is_empty()) {
            writeln!(mf, "{fg}").unwrap();
        }
    }

    let mut durows = 0;
    let mut columns = 0;
    if du {
        let dus: Vec<_> = fgs
            .par_iter()
            .map(|g| {
                let du = g.d_unmark();
                if girthmin == 1 && girthmax == 255 {
                    du
                } else {
                    du.filter(|f| {
                        let forest: Vec<u8> = BitPositions(f).map(|a| a as u8).collect();
                        let gr = g.graph().contract_multiple_neighborhoods(&forest);
                        let p = gr.girth();
                        // let q = gr.vertices_valency(1, 1).len();
                        // let q = gr.count_double_edges();
                        p >= girthmin && p <= girthmax //&& q > 0
                    })
                }
            })
            .collect();
        for (fg, du) in fgs.iter().zip(dus.iter()) {
            for (i, j, s) in du.matrix_entries(columns, durows) {
                writeln!(mf, "{} {} {}", i + 1, j + 1, s).unwrap();
            }
            columns += fg.subforests().len();
            durows += du.smaller_forests().len();
        }
        println!("Pairs graph + subforest up to iso: {}", columns);
        println!("du differential rows: {}", durows);

        if registry {
            let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}.rows");
            let mf = File::create(&filename).unwrap();
            let mut mf = BufWriter::new(mf);
            for (fg, du) in fgs.iter().zip(dus.into_iter()) {
                if du.smaller_forests().is_empty() {
                    continue;
                }
                writeln!(mf, "{}{du}", fg.graph_string()).unwrap();
            }
        }
    }

    let mut csum = 0;
    if dc {
        let dcs: Vec<_> = fgs.into_par_iter().map(|fg| fg.d_contract()).collect();
        let graph_table = GraphTable::new(
            dcs.iter()
                .flat_map(|dc| dc.contracted_graphs())
                .map(|(g, fs)| (g.clone(), fs.as_slice())),
        );
        csum = graph_table.size();
        println!("dc differential rows: {}", csum);
        println!("Number of contracted graphs: {}", graph_table.num_graphs());

        columns = 0;
        for dc in &dcs {
            for entries in dc.columns() {
                for ((i, m), s) in entries {
                    let (g, _) = &dc.contracted_graphs()[*i];
                    let Some(j) = graph_table.get_index(g, *m) else {
                        continue;
                    };
                    writeln!(mf, "{} {} {}", j + durows as usize + 1, columns + 1, s).unwrap();
                }
                columns += 1;
            }
        }

        if registry {
            let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}.rows");
            let mut mf = if du {
                File::options().write(true).open(&filename).unwrap()
            } else {
                File::create(&filename).unwrap()
            };
            mf.seek(SeekFrom::End(0)).unwrap();
            let mut mf = BufWriter::new(mf);
            writeln!(mf, "{}", graph_table).unwrap();
        }
    }

    writeln!(mf, "0 0 0").unwrap();
    drop(mf);
    let mut mf = File::options().write(true).open(&filename).unwrap();
    write!(mf, "{} {}", durows + csum, columns).unwrap();
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

fn compute_matrix_full(
    graphs: &[Graph],
    forest_size: u8,
    (matrix_dir, matrix_name): (&str, &str),
    graph_table: Option<GraphTable>,
    args: &Args,
) {
    let (girthmin, girthmax) = (args.girthmin, args.girthmax);
    let odd = args.odd;
    let registry = args.registry;
    let transpose = args.transpose;

    let n_graphs = graphs.len();
    let g6s: Vec<_> = graphs
        .par_iter()
        .filter(|g| !odd || !g.contains_loop())
        .map(|g| g.canonical_label().0.to_g6())
        .collect();
    println!("Loaded {n_graphs} graphs");

    let t = if transpose { "T" } else { "" };
    let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}{t}.sms");
    let mf = File::create(&filename).unwrap();
    let mut mf = BufWriter::new(mf);
    writeln!(mf, "{}M", " ".repeat(24)).unwrap();

    let start = Instant::now();

    let (fgs, (dus, dcs)): (Vec<_>, (Vec<_>, Vec<_>)) = graphs
        .par_iter()
        .filter(|g| !odd || !g.contains_loop())
        .map(|g| {
            let fc = ForestedGraph::new(g, forest_size as usize, true, odd);
            let g = fc.filter(|f| fc.girth(f) > 0);
            let du = g.d_unmark();
            let dc = g.d_contract();
            (g, (du, dc))
        })
        .collect();

    let mut columns = 0;
    for fg in fgs.iter() {
        columns += fg.subforests().len();
    }
    println!("Pairs graph + subforest up to iso: {}", columns);

    let sfgs: Vec<_> = g6s
        .iter()
        .map(|s| s.as_str())
        .zip(dus.iter().map(|du| du.smaller_forests()))
        .map(|(g, fg)| {
            let fr: Vec<u64> = fg
                .iter()
                .filter(|&&f| {
                    if girthmin == 1 && girthmax == 255 {
                        return true;
                    }
                    let graph = Graph::from_g6(g);
                    let forest: Vec<u8> = BitPositions(f).map(|a| a as u8).collect();
                    let gr = graph.contract_multiple_neighborhoods(&forest);
                    let p = gr.girth();
                    p >= girthmin && p <= girthmax
                })
                .copied()
                .collect();
            (g, fr)
        })
        .collect();
    let graph_table = graph_table.unwrap_or_else(|| {
        GraphTable::new(
            sfgs.iter()
                .map(|(s, v)| (s.to_string(), v.as_slice()))
                .chain(
                    dcs.iter()
                        .flat_map(|dc| dc.contracted_graphs())
                        .map(|(g, fs)| (g.clone(), fs.as_slice())),
                ),
        )
    });
    let csum = graph_table.size();
    println!("du + dc differential rows: {}", csum);
    println!("Number of target graphs: {}", graph_table.num_graphs());

    let mut column = 0;
    for (g6, du) in g6s.iter().zip(dus.iter()) {
        for entries in du.columns() {
            for (m, s) in entries {
                let Some(j) = graph_table.get_index(g6, *m) else {
                    continue;
                };
                if transpose {
                    writeln!(mf, "{} {} {}", column + 1, j + 1, s).unwrap();
                } else {
                    writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
                }
            }
            column += 1;
        }
    }
    let mut column = 0;
    for dc in &dcs {
        for entries in dc.columns() {
            for ((i, m), s) in entries {
                let (g, _) = &dc.contracted_graphs()[*i];
                let Some(j) = graph_table.get_index(g, *m) else {
                    continue;
                };
                if transpose {
                    writeln!(mf, "{} {} {}", column + 1, j + 1, s).unwrap();
                } else {
                    writeln!(mf, "{} {} {}", j + 1, column + 1, s).unwrap();
                }
            }
            column += 1;
        }
    }
    writeln!(mf, "0 0 0").unwrap();
    drop(mf);
    let mut mf = File::options().write(true).open(&filename).unwrap();
    if transpose {
        write!(mf, "{} {}", columns, csum).unwrap();
    } else {
        write!(mf, "{} {}", csum, columns).unwrap();
    }
    println!("{} written", &filename);

    if registry {
        let cols = if transpose { "cols" } else { "rows" };
        let rows = if transpose { "rows" } else { "cols" };
        let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}{t}.{cols}");
        let mf = File::create(&filename).unwrap();
        let mut mf = BufWriter::new(mf);
        for fg in fgs.iter().filter(|g| !g.subforests().is_empty()) {
            writeln!(mf, "{fg}").unwrap();
        }

        let filename = format!("{matrix_dir}/{matrix_name}_f{forest_size}{t}.{rows}");
        let mf = File::create(&filename).unwrap();
        let mut mf = BufWriter::new(mf);
        writeln!(mf, "{}", graph_table).unwrap();
    }

    let total_time = start.elapsed();
    println!("---");
    println!("Total time: {:.3} s", total_time.as_secs_f64());
    println!(
        "Avg per graph: {:.3} ms",
        total_time.as_secs_f64() * 1e3 / n_graphs as f64
    );
    println!();
}

fn print_dimensions(
    rank: u8,
    three_connected: bool,
    girthmin: u8,
    girthmax: u8,
    odd: bool,
    hairs: u8,
) {
    let dims = compute_dimensions(
        &read_all_graphs(rank, three_connected, hairs),
        girthmin,
        girthmax,
        odd,
        hairs,
    );
    print!("e\\p\t");
    for i in 0..(2 * rank - 2) {
        print! {"{i}\t"};
    }
    println!();
    let mut e = 2 * rank - 3 + hairs;
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
    let mut rank = args.rank;

    if args.girthmax < args.girthmin {
        print!("Maximum girth can not be less than minimum girth!");
        return;
    }

    if args.dimensions {
        print_dimensions(
            rank,
            !args.all && args.hairs == 0,
            args.girthmin,
            args.girthmax,
            args.odd,
            args.hairs,
        );
        return;
    }

    fs::create_dir_all(&args.matrix_dir).unwrap();

    let prefix = if !args.full && args.dc && !args.du {
        "dc_"
    } else if !args.full && args.du && !args.dc {
        "du_"
    } else {
        ""
    };
    let stem = if args.full {
        "f"
    } else if args.all {
        "a"
    } else {
        ""
    };
    let sign_convention = if args.odd { "o" } else { "" };
    let hairs = if args.hairs > 0 {
        format!("h{}_", args.hairs)
    } else {
        "".into()
    };
    let girthmin = if args.girthmin > 1 {
        format!("g{}_", args.girthmin)
    } else {
        "".into()
    };
    let girthmax = if args.girthmax < 255 {
        format!("G{}_", args.girthmax)
    } else {
        "".into()
    };
    let matrix_name = if let Some(ref graphfile) = args.graphfile {
        let gf = Path::new(graphfile)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        format!("{prefix}{stem}{sign_convention}{hairs}{girthmin}{girthmax}{gf}")
    } else {
        format!("{prefix}{stem}{sign_convention}{hairs}{girthmin}{girthmax}r{rank}")
    };

    if args.all_excesses {
        let graphs = read_all_graphs(rank, !args.all, args.hairs);
        for (e, gs) in graphs.iter().rev().enumerate() {
            let mn = format!("{matrix_name}_e{e}");
            for d in 1..(2 * rank - 2 - e as u8) {
                compute_matrix(gs, (d, args.hairs), (&args.matrix_dir, &mn), true, &args);
            }
        }
        return;
    }

    let mut min_vertices = if args.full { 2 } else { 2 * rank - 2 };
    let mut max_vertices = 2 * rank - 2;
    let mut min_degree = 4 * rank / 5;
    if args.hairs > 0 {
        min_degree = (rank - 3) / 2;
        rank += args.hairs;
        min_vertices = 2 * rank - 2 - args.hairs;
        max_vertices = 2 * rank - 2 - args.hairs;
    }
    let mut graphs = if let Some(ref graphfile) = args.graphfile {
        read_graphfile(graphfile, !args.all && args.hairs == 0, args.hairs)
    } else {
        read_graphs(
            rank,
            min_vertices,
            max_vertices,
            !args.all && args.hairs == 0,
            args.hairs,
        )
    };
    if args.hairs > 0 {
        graphs.retain(|g| {
            (0..g.num_vertices)
                .filter(|&v| g.adj[v as usize].count_ones() == 1)
                .count()
                >= args.hairs as usize
        });
    }

    let degrees = args
        .degree
        .map(|d| vec![d])
        .unwrap_or_else(|| (min_degree..max_vertices).collect());
    for d in degrees {
        if args.full {
            let fgs: Vec<_> = graphs
                .par_iter()
                .map(|g| ForestedGraph::new(g, d as usize - 1, true, false))
                .collect();
            let gt = GraphTable::from_forested_graphs(fgs);
            compute_matrix_full(
                &graphs,
                d,
                (&args.matrix_dir, &matrix_name),
                Some(gt),
                &args,
            );
        } else {
            compute_matrix(
                &graphs,
                (d, args.hairs),
                (&args.matrix_dir, &matrix_name),
                args.all,
                &args,
            );
        }
    }
}
