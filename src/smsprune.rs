#![allow(clippy::needless_range_loop)]
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;
use std::{env, mem};

use rayon::prelude::*;

use graphc::graph::BigGraph;

/// Prune the matrix by deleting all rows/columns containing a single non-zero entry
/// and the columns/rows where the entry is.
fn prune_matrix<const BY_COLUMNS: usize>(
    m: &mut Vec<([u32; 2], i8)>,
    dims: [usize; 2],
) -> [usize; 2] {
    let [h, w] = dims;
    let mut indices = [vec![0_u32; h], vec![0_u32; w]];

    for &(t, s) in m.iter() {
        if s == 0 {
            println!("0 found {} {}", t[0], t[1]);
        };
        indices[BY_COLUMNS][t[BY_COLUMNS] as usize - 1] += 1;
    }

    let mut ones = 0;
    for &(t, _) in m.iter() {
        if indices[BY_COLUMNS][t[BY_COLUMNS] as usize - 1] == 1 {
            ones += 1;
            indices[BY_COLUMNS ^ 1][t[BY_COLUMNS ^ 1] as usize - 1] = 1;
        }
    }

    let mut ones_other = 0;
    for &n in &indices[BY_COLUMNS ^ 1] {
        if n == 1 {
            ones_other += 1;
        }
    }

    let mut row_shift: Vec<u32> = (0..h as u32).collect();
    let mut col_shift: Vec<u32> = (0..w as u32).collect();
    let mut shift = 0;
    for i in 0..h {
        if indices[0][i] == 1 {
            shift += 1;
        } else if BY_COLUMNS == 0 && indices[0][i] == 0 {
            ones += 1;
            shift += 1;
        }
        row_shift[i] -= shift;
    }
    let mut shift = 0;
    for i in 0..w {
        if indices[1][i] == 1 {
            shift += 1;
        } else if BY_COLUMNS == 1 && indices[1][i] == 0 {
            ones += 1;
            shift += 1;
        }
        col_shift[i] -= shift;
    }

    let mb = mem::take(m);
    *m = mb
        .into_par_iter()
        .filter_map(|([i, j], s)| {
            if indices[0][i as usize - 1] != 1 && indices[1][j as usize - 1] != 1 {
                Some((
                    [row_shift[i as usize - 1] + 1, col_shift[j as usize - 1] + 1],
                    s,
                ))
            } else {
                None
            }
        })
        .collect();

    let mut ret = [0, 0];
    ret[BY_COLUMNS] = dims[BY_COLUMNS] - ones;
    ret[BY_COLUMNS ^ 1] = dims[BY_COLUMNS ^ 1] - ones_other;
    ret
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(filename) = args.next() else {
        eprintln!("Usage: smssort <.sms file>");
        return;
    };

    let file = File::open(&filename).unwrap();
    let reader = BufReader::with_capacity(500_000_000, file);
    let mut lines = reader.lines();
    let first = lines.next().unwrap().unwrap();
    let mut lines = lines
        .map(|s| {
            let s = s.unwrap();
            let mut numbers = s.split_whitespace().map(|i| i.parse::<i64>().unwrap());
            (
                [
                    numbers.next().unwrap() as u32,
                    numbers.next().unwrap() as u32,
                ],
                numbers.next().unwrap() as i8,
            )
        })
        .collect::<Vec<_>>();
    lines.pop();
    println!("Read {}", &filename);

    let t = Instant::now();
    lines.par_sort_unstable();
    println!("Sorted in {:.3}ms", t.elapsed().as_secs_f64() * 1e3);
    println!();

    let mut dims_iter = first
        .split_whitespace()
        .map(|i| i.parse::<usize>().unwrap());
    let mut dims = [dims_iter.next().unwrap(), dims_iter.next().unwrap()];
    println!("Original dimensions: {} {}", dims[0], dims[1]);

    loop {
        let dims0 = dims;
        dims = prune_matrix::<0>(&mut lines, dims);
        if dims != dims0 {
            println!(
                "pruned {} rows, {} columns",
                dims0[0] - dims[0],
                dims0[1] - dims[1]
            );
        } else {
            break;
        }
    }

    loop {
        let dims0 = dims;
        dims = prune_matrix::<1>(&mut lines, dims);
        if dims != dims0 {
            println!(
                "c pruned {} rows, {} columns",
                dims0[0] - dims[0],
                dims0[1] - dims[1]
            );
        } else {
            break;
        }
    }

    println!("Final dimensions: {} {}", dims[0], dims[1]);

    let edges = lines
        .iter()
        .map(|&([i, j], _)| (i as usize - 1, j as usize - 1 + dims[0]))
        .collect();
    let graph = BigGraph::new(dims[0] + dims[1], edges);
    let (c, _) = graph.connected_components();

    println!("{c} block components");

    let path = Path::new(&filename);
    let matrix_name = path
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or("matrix".into());
    let new_name = format!("pruned_{matrix_name}");
    let parent = path.parent().unwrap_or(Path::new("."));

    let file = File::create(parent.join(new_name)).unwrap();
    let mut file = BufWriter::with_capacity(500_000_000, file);
    writeln!(&mut file, "{} {} M", dims[0], dims[1]).unwrap();
    for ([i, j], s) in lines {
        writeln!(&mut file, "{i} {j} {s}").unwrap();
    }
    writeln!(&mut file, "0 0 0").unwrap();
}
