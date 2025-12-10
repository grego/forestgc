#![allow(clippy::needless_range_loop)]
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::mem;
use std::path::Path;
use std::time::Instant;

use argh::FromArgs;
use rayon::prelude::*;

use graphc::graph::BigGraph;

/// Forested graph complex computations.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
struct Args {
    /// compute the registry for row and column indices in the pruned matrix
    #[argh(switch, short = 'r')]
    registry: bool,
    /// try to separate the matrix into block diagonal components
    #[argh(switch, short = 'b')]
    blocks: bool,
    /// actually don't prune
    #[argh(switch)]
    no_prune: bool,
    /// separate into two matrices after the provided number of rows
    #[argh(option)]
    row_sep: Option<u32>,
    /// join with another file before pruning
    #[argh(option, short = 'j')]
    join: Option<String>,
    /// prune the matrix registry file to keep only the elements corresponding
    /// to nonzeros of a vector
    #[argh(option)]
    vec: Option<String>,
    /// the name of the matrix to prune
    #[argh(positional)]
    filename: String,
}

/// Prune the matrix by deleting all rows/columns containing a single non-zero entry
/// and the columns/rows where the entry is.
fn prune_matrix<const BY_COLUMNS: usize>(
    m: &mut Vec<([u32; 2], i8)>,
    dims: [usize; 2],
    mut row_sep: Option<&mut u32>,
) -> ([usize; 2], [Vec<u32>; 2]) {
    let [h, w] = dims;
    let mut indices = [vec![0_u32; h], vec![0_u32; w]];

    for &(t, s) in m.iter() {
        if s == 0 {
            println!("0 found {} {}", t[0], t[1]);
        };
        indices[BY_COLUMNS][t[BY_COLUMNS] as usize - 1] += 1;
    }

    for &(t, _) in m.iter() {
        if indices[BY_COLUMNS][t[BY_COLUMNS] as usize - 1] == 1 {
            indices[BY_COLUMNS ^ 1][t[BY_COLUMNS ^ 1] as usize - 1] = 1;
        }
    }

    let mut other_removed = 0;
    for &n in &indices[BY_COLUMNS ^ 1] {
        if n == 1 {
            other_removed += 1;
        }
    }

    let mut removed = 0;
    let mut shift = 0;
    for i in 0..h {
        if indices[0][i] == 1 || (BY_COLUMNS == 0 && indices[0][i] == 0) {
            if BY_COLUMNS == 0 {
                removed += 1;
            }
            shift += 1;
            indices[0][i] = u32::MAX;
        } else {
            indices[0][i] = i as u32 - shift;
        }
    }
    let mut shift = 0;
    for i in 0..w {
        if indices[1][i] == 1 || (BY_COLUMNS == 1 && indices[1][i] == 0) {
            if BY_COLUMNS == 1 {
                removed += 1;
            }
            shift += 1;
            indices[1][i] = u32::MAX;
        } else {
            indices[1][i] = i as u32 - shift;
        }
    }

    let mb = mem::take(m);
    *m = mb
        .into_par_iter()
        .filter_map(|([i, j], s)| {
            if indices[0][i as usize - 1] != u32::MAX && indices[1][j as usize - 1] != u32::MAX {
                Some((
                    [
                        indices[0][i as usize - 1] + 1,
                        indices[1][j as usize - 1] + 1,
                    ],
                    s,
                ))
            } else {
                None
            }
        })
        .collect();

    let mut ret = [0, 0];
    ret[BY_COLUMNS] = dims[BY_COLUMNS] - removed;
    ret[BY_COLUMNS ^ 1] = dims[BY_COLUMNS ^ 1] - other_removed;
    if let Some(ref mut sep) = row_sep {
        **sep = indices[0][**sep as usize];
    }
    (ret, indices)
}

fn read_sms_file<R: BufRead>(reader: &mut R) -> ([usize; 2], Vec<([u32; 2], i8)>) {
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

    let mut dims_iter = first
        .split_whitespace()
        .map(|i| i.parse::<usize>().unwrap());
    let dims = [dims_iter.next().unwrap(), dims_iter.next().unwrap()];

    (dims, lines)
}

fn write_sms_file<W: Write>(dims: [usize; 2], mat: Vec<([u32; 2], i8)>, w: &mut W) {
    writeln!(w, "{} {} M", dims[0], dims[1]).unwrap();
    for ([i, j], s) in mat {
        writeln!(w, "{i} {j} {s}").unwrap();
    }
    writeln!(w, "0 0 0").unwrap();
}

fn main() {
    let mut args: Args = argh::from_env();
    let path = Path::new(&args.filename);
    let matrix_stem = path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or("matrix".into());
    let new_stem = format!("pruned_{matrix_stem}");
    let parent = path.parent().unwrap_or(Path::new("."));

    if let Some(ref vecname) = args.vec {
        let reg = prune_by_vec(vecname).unwrap();
        let colname = parent.join(matrix_stem.as_ref()).with_extension("cols");
        let new_stem = format!("vpruned_{matrix_stem}");
        let newcolname = parent.join(&new_stem).with_extension("cols");
        prune_registry(&colname, &newcolname, &reg);
        println!("Pruned into {} entries", reg.len());
        return;
    }

    let file = File::open(&args.filename).unwrap();
    let mut reader = BufReader::with_capacity(500_000_000, file);
    let (mut dims, mut lines) = read_sms_file(&mut reader);
    println!("Read {}", &args.filename);
    println!("Original dimensions: {} {}", dims[0], dims[1]);

    if let Some(ref afile) = args.join {
        let file = File::open(afile).unwrap();
        let mut reader = BufReader::with_capacity(500_000_000, file);
        let (adims, alines) = read_sms_file(&mut reader);
        lines.extend(
            alines
                .into_iter()
                .map(|([i, j], s)| ([i + dims[0] as u32, j], s)),
        );
        dims[0] += adims[0];
        println!("Read {}", &afile);
        println!("Original dimensions: {} {}", adims[0], adims[1]);
    }

    let t = Instant::now();
    lines.par_sort_unstable();
    println!("Sorted in {:.3}ms", t.elapsed().as_secs_f64() * 1e3);
    println!();

    if args.no_prune {
        let file = File::create(parent.join(&new_stem).with_extension("sms")).unwrap();
        let mut file = BufWriter::with_capacity(500_000_000, file);
        write_sms_file(dims, lines, &mut file);
        return;
    }

    let mut rankp = 0;
    let dims0 = dims;

    let mut registry: Option<[Vec<_>; 2]> = None;
    loop {
        let (ndims, indices) = prune_matrix::<0>(&mut lines, dims, args.row_sep.as_mut());
        if dims == ndims {
            break;
        }
        println!(
            "pruned {} rows, {} columns",
            dims[0] - ndims[0],
            dims[1] - ndims[1]
        );
        if args.registry {
            let mut reg = [vec![0; ndims[0]], vec![0; ndims[1]]];
            for i in 0..=1 {
                for j in 0..dims[i] {
                    if indices[i][j] != u32::MAX {
                        reg[i][indices[i][j] as usize] = if let Some(ref r) = registry {
                            r[i][j]
                        } else {
                            j
                        };
                    }
                }
            }
            registry = Some(reg)
        }
        dims = ndims;
    }
    if dims0 != dims {
        rankp += dims0[1] - dims[1];
    }

    let dims1 = dims;
    loop {
        let (ndims, indices) = prune_matrix::<1>(&mut lines, dims, args.row_sep.as_mut());
        if dims == ndims {
            break;
        }
        println!(
            "c pruned {} rows, {} columns",
            dims[0] - ndims[0],
            dims[1] - ndims[1]
        );
        if args.registry {
            let mut reg = [vec![0; ndims[0]], vec![0; ndims[1]]];
            for i in 0..=1 {
                for j in 0..dims[i] {
                    if indices[i][j] != u32::MAX {
                        reg[i][indices[i][j] as usize] = if let Some(ref r) = registry {
                            r[i][j]
                        } else {
                            j
                        };
                    }
                }
            }
            registry = Some(reg)
        }
        dims = ndims;
    }
    if dims1 != dims {
        rankp += dims1[0] - dims[0];
    }

    println!("Final dimensions: {} {}", dims[0], dims[1]);
    {
        let mut file = File::create(parent.join(&new_stem).with_extension("rankp")).unwrap();
        write!(&mut file, "{rankp}").unwrap();
    }

    if let Some(ref sep) = args.row_sep {
        let ustem = format!("du_{new_stem}");
        let cstem = format!("dc_{new_stem}");
        let ufile = File::create(parent.join(ustem).with_extension("sms")).unwrap();
        let mut ufile = BufWriter::with_capacity(500_000_000, ufile);
        let cfile = File::create(parent.join(cstem).with_extension("sms")).unwrap();
        let mut cfile = BufWriter::with_capacity(500_000_000, cfile);

        writeln!(&mut ufile, "{} {} M", sep, dims[1]).unwrap();
        writeln!(&mut cfile, "{} {} M", dims[0] - *sep as usize, dims[1]).unwrap();
        for ([i, j], s) in &lines {
            if i <= sep {
                writeln!(&mut ufile, "{i} {j} {s}").unwrap();
            } else {
                writeln!(&mut cfile, "{} {j} {s}", i - sep).unwrap();
            }
        }
        writeln!(&mut ufile, "0 0 0").unwrap();
        writeln!(&mut cfile, "0 0 0").unwrap();

        return;
    }

    let (nc, comps) = if args.blocks {
        let edges = lines
            .iter()
            .map(|&([i, j], _)| (i as usize - 1, j as usize - 1 + dims[0]))
            .collect();
        let graph = BigGraph::new(dims[0] + dims[1], edges);
        graph.connected_components()
    } else {
        (1, Vec::with_capacity(0))
    };

    println!("{nc} block components");

    if nc > 1 {
        for c in 1..=nc {
            let mut remap = [vec![0; dims[0]], vec![0; dims[1]]];
            let mut x = 0;
            for k in 0..dims[0] {
                if comps[k] == c {
                    remap[0][k] = x;
                    x += 1;
                }
            }
            let mut y = 0;
            for k in dims[0]..dims[0] + dims[1] {
                if comps[k] == c {
                    remap[1][k - dims[0]] = y;
                    y += 1;
                }
            }

            let stem = format!("{new_stem}_b{c}");
            let file = File::create(parent.join(stem).with_extension("sms")).unwrap();
            let mut file = BufWriter::with_capacity(500_000_000, file);
            writeln!(&mut file, "{x} {y} M").unwrap();
            for &([i, j], s) in lines.iter() {
                if comps[i as usize - 1] != c {
                    continue;
                }
                writeln!(
                    &mut file,
                    "{} {} {s}",
                    remap[0][i as usize - 1] + 1,
                    remap[1][j as usize - 1] + 1
                )
                .unwrap();
            }
            writeln!(&mut file, "0 0 0").unwrap();
        }
        return;
    }

    let file = File::create(parent.join(&new_stem).with_extension("sms")).unwrap();
    let mut file = BufWriter::with_capacity(500_000_000, file);
    write_sms_file(dims, lines, &mut file);

    let Some(reg) = registry else {
        return;
    };
    let file = File::create(parent.join(&new_stem).with_extension("rowr")).unwrap();
    let mut file = BufWriter::with_capacity(500_000_000, file);
    for &j in &reg[0] {
        file.write_all(&j.to_le_bytes()).unwrap();
    }

    let file = File::create(parent.join(&new_stem).with_extension("colr")).unwrap();
    let mut file = BufWriter::with_capacity(500_000_000, file);
    for &j in &reg[1] {
        file.write_all(&j.to_le_bytes()).unwrap();
    }

    let rowname = parent.join(matrix_stem.as_ref()).with_extension("rows");
    let newrowname = parent.join(&new_stem).with_extension("rows");
    prune_registry(&rowname, &newrowname, &reg[0]);

    let colname = parent.join(matrix_stem.as_ref()).with_extension("cols");
    let newcolname = parent.join(&new_stem).with_extension("cols");
    prune_registry(&colname, &newcolname, &reg[1]);
}

fn read_registry<R: BufRead>(reader: R) -> std::io::Result<(Vec<String>, Vec<usize>, Vec<u64>)> {
    let (mut graphs, mut indices, mut forests) = (Vec::new(), Vec::new(), Vec::new());
    for line in reader.lines() {
        let line = line?;
        let mut numbers = line.split_whitespace();
        let Some(graph) = numbers.next() else {
            continue;
        };
        graphs.push(graph.to_string());
        indices.push(forests.len());
        for m in numbers.map(|s| u64::from_str_radix(s, 16).unwrap()) {
            forests.push(m);
        }
    }
    indices.push(forests.len());
    Ok((graphs, indices, forests))
}

fn prune_registry(old: &Path, new: &Path, pruned: &[usize]) {
    let Ok(file) = File::open(old) else {
        return;
    };
    let file = BufReader::with_capacity(500_000_000, file);
    let (graphs, indices, forests) = read_registry(file).unwrap();
    let file = File::create(new).unwrap();
    let mut file = BufWriter::with_capacity(500_000_000, file);
    let mut metaidx = 1;
    let mut idx = indices[metaidx];
    dbg!((graphs.len(), indices.len(), forests.len()));
    write!(file, "{}", graphs[0]).unwrap();
    for &j in pruned {
        if j >= idx {
            writeln!(file).unwrap();
            write!(file, "{}", graphs[metaidx]).unwrap();
            loop {
                metaidx += 1;
                idx = indices[metaidx];
                if idx > j {
                    break;
                };
            }
        }
        write!(file, " {:X}", forests[j]).unwrap();
    }
}

fn prune_by_vec(vecname: &str) -> std::io::Result<Vec<usize>> {
    let v = std::fs::read(vecname)?;
    let (mut nv, mut idxs) = (Vec::new(), Vec::new());
    println!("loaded {vecname} with {} entries", v.len());

    for (i, &a) in v.iter().enumerate() {
        if a != 0 {
            nv.push(a);
            idxs.push(i);
        }
    }

    let vecname = Path::new(vecname);
    let name = vecname
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or("vec".into());
    let new_name = format!("vpruned_{name}");
    let parent = vecname.parent().unwrap_or(Path::new("."));

    std::fs::write(parent.join(new_name), &nv)?;

    Ok(idxs)
}
