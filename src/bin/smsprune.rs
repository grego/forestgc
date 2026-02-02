#![allow(clippy::needless_range_loop)]
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;
use std::{array, mem};

use argh::FromArgs;
use rayon::prelude::*;

use graphc::graph::BigGraph;
use rustc_hash::{FxHashMap, FxHashSet};

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
    /// prune also rows with 2 nnz
    #[argh(switch, short = '2')]
    twocol: bool,
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
    m: &mut Vec<([u32; 2], i32)>,
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

/// For the rows which have 2 nonzero entries, zero the first entry by adding the appropriate
/// multiple of the first column to the second.
///
/// Returns true if something was merged.
fn merge_2cols(m: &mut Vec<([u32; 2], i32)>, [x, y]: [usize; 2]) -> [usize; 2] {
    let mut ii = 1;
    let mut nz = 0;
    let mut entries = Vec::new();
    let mut ops = Vec::new();
    let mut row_ranges = Vec::with_capacity(x);
    let last = ([x as u32 + 1, y as u32 + 1], 1);
    let mut last_k = 0;
    for (k, &([i, j], s)) in m.iter().chain(std::iter::once(&last)).enumerate() {
        if i != ii {
            if nz == 2 {
                let (j0, s0) = entries[0];
                let (j1, s1) = entries[1];
                let d = gcd(s0, s1);
                ops.push(([j0, j1], [s0 / d, s1 / d]));
            }
            row_ranges.push(last_k..k);
            last_k = k;
            ii = i;
            nz = 0;
            entries.drain(..);
        }
        if s == 0 {
            println!("0 found!");
        }
        nz += 1;
        entries.push((j, s));
    }

    ops.par_sort_by(
        |([_, j10], [s00, s10]), ([_, j11], [s01, s11])| match j10.cmp(j11) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => (s00.abs() + s10.abs()).cmp(&(s01.abs() + s11.abs())),
        },
    );
    ops.dedup_by_key(|([_, j], _)| *j);
    ops.par_sort();
    let mut opindices = vec![u32::MAX; y + 1];
    for (k, &([_, j], _)) in ops.iter().enumerate() {
        opindices[j as usize] = k as u32;
    }

    for k in (0..ops.len()).rev() {
        let ([j0, _], [_, s1]) = ops[k];
        if let Ok(l) = ops[0..k].binary_search_by_key(&j0, |&([j, _], _)| j) {
            ops[l].1[0] *= s1;
            let mut ii = l + 1;
            while let Some(([jj, _], _)) = ops.get(ii).cloned()
                && jj == j0
                && ii < k
            {
                ops[ii].1[0] *= s1;
                ii += 1;
            }
            let mut ii = l - 1;
            while let Some(([jj, _], _)) = ops.get(ii).cloned()
                && jj == j0
            {
                ops[ii].1[0] *= s1;
                ii -= 1;
            }
        }

        let i = opindices[j0 as usize];
        if i < k as u32 {
            ops[i as usize].1[1] *= s1;
        }
    }

    println!("{} ops found", ops.len());
    if ops.is_empty() {
        return [x, y];
    }

    let mut shift = 0;
    let mut ny = 0;
    let colindices: Vec<_> = (0..=y)
        .map(|i| {
            if opindices[i] != u32::MAX {
                shift += 1;
                u32::MAX
            } else {
                ny = i as u32 - shift;
                ny
            }
        })
        .collect();

    let mb = mem::take(m);
    entries.drain(..);
    *m = row_ranges
        .into_par_iter()
        .flat_map(|row_entries| {
            let mut entries = mb[row_entries].to_vec();
            let Some(([i, _], _)) = entries.first().cloned() else {
                return entries;
            };

            let mut indices = FxHashSet::default();
            // operations relevant to the current row
            let mut rops = Vec::new();
            // stack of indices of the current row operations
            let mut stack = Vec::new();
            for &([_, j], _) in &entries {
                stack.push(j);
                indices.insert(j);
            }

            while let Some(j) = stack.pop() {
                if let Ok(i) = ops.binary_search_by_key(&j, |&([j, _], _)| j) {
                    rops.push(ops[i]);
                    let ([_, j1], _) = ops[i];
                    if indices.insert(j1) {
                        stack.push(j1);
                    }
                    let mut ii = i + 1;
                    while let Some(([jj, j1], [s0, s1])) = ops.get(ii)
                        && *jj == j
                    {
                        rops.push(([*jj, *j1], [*s0, *s1]));
                        if indices.insert(*j1) {
                            stack.push(*j1);
                        }
                        ii += 1;
                    }

                    if i == 0 {
                        continue;
                    }
                    let mut ii = i - 1;
                    while let Some(([jj, j1], [s0, s1])) = ops.get(ii)
                        && *jj == j
                    {
                        rops.push(([*jj, *j1], [*s0, *s1]));
                        if indices.insert(*j1) {
                            stack.push(*j1);
                        }
                        if ii == 0 {
                            break;
                        }
                        ii -= 1;
                    }
                }

                let i = opindices[j as usize];
                if i != u32::MAX {
                    rops.push(ops[i as usize]);
                    let ([j0, _], _) = ops[i as usize];
                    if indices.insert(j0) {
                        stack.push(j0);
                    }
                }
            }
            drop(indices);
            drop(stack);
            rops.sort_unstable();
            rops.dedup();

            for ([j0, j1], [s0, s1]) in rops.into_iter().rev() {
                let k1 = entries.iter().position(|([_, j], _)| *j == j1);
                let v1 = k1.map(|k| entries[k].1).unwrap_or(0);
                let k0 = entries.iter().position(|([_, j], _)| *j == j0);
                let v0 = k0.map(|k| entries.swap_remove(k).1).unwrap_or(0);
                let new = s1 * v0 - s0 * v1;
                if new != 0 {
                    entries.push(([i, j0], new))
                }
            }

            let mut i = 0;
            while i < entries.len() {
                let e = &mut entries[i];
                let c = colindices[e.0[1] as usize];
                if c == u32::MAX {
                    entries.swap_remove(i);
                } else {
                    e.0[1] = c;
                    i += 1;
                }
            }

            entries.sort_unstable();
            entries
        })
        .collect();

    [x, ny as usize]
}

fn divide_cols(m: &mut Vec<([u32; 2], i32)>, [_, y]: [usize; 2]) {
    let mut col_divisors = vec![0; y + 1];
    for &([_, j], s) in &*m {
        col_divisors[j as usize] = gcd(col_divisors[j as usize], s);
    }

    let mut divided = 0;
    let mut max = 0;
    for t in m {
        let d = col_divisors[t.0[1] as usize];
        if d > 1 {
            t.1 /= d;
            divided += 1;
            max = max.max(d);
        }
    }
    println! {"divided {divided} cols, max by {max}"};
}

fn count_statistics(m: &[([u32; 2], i32)], [_, y]: [usize; 2]) {
    let mut counts = vec![0; y];
    for &([_, j], _) in m {
        counts[j as usize - 1] += 1;
    }

    const THRESHOLD: usize = 1000;
    let mut c = 0;
    let min = counts.iter().min().copied().unwrap_or(0);
    let max = counts.iter().max().copied().unwrap_or(0);
    let mut large = 0;
    for &cc in &counts {
        if cc == min {
            c += 1;
        }
        if cc > THRESHOLD {
            large += 1;
        }
    }
    drop(counts);

    println!("remaing {c} columns with {min} entries");
    println!("{large} columns with over {THRESHOLD} entries, max {max}");
}

/// Delete duplicate rows of size N.
fn delete_duplicate_rows<const N: usize>(
    m: &mut Vec<([u32; 2], i32)>,
    [x, _]: [usize; 2],
) -> usize {
    let mut rcounts = vec![0; x];
    for &([i, _], _) in m.iter() {
        rcounts[i as usize - 1] += 1;
    }

    let mut minrows = 0;
    for &rc in &rcounts {
        if rc == N {
            minrows += 1;
        }
    }
    println!("remaining {minrows} rows with {N} entries");

    let mut rowdelete = vec![false; x + 1];
    let mut rowset: FxHashMap<_, Vec<_>> = FxHashMap::default();
    let mut iter = m
        .iter()
        .filter(|&([i, _], _)| rcounts[*i as usize - 1] == N)
        .map(|&([r, i], si)| (r, i, si));
    let mut next_touple = || {
        let mut t = [Default::default(); N];
        for i in 0..N {
            t[i] = iter.next()?;
        }
        Some(t)
    };
    let mut duplicities = 0;
    'outer: while let Some(t) = next_touple() {
        let r = t[0].0;

        let indices: [_; N] = array::from_fn(|i| t[i].1);
        let mut values: [_; N] = array::from_fn(|i| t[i].2);

        let mut g = gcd_array(&values);
        if values[0] < 0 {
            g *= -1;
        }
        if g != 1 {
            for i in 0..N {
                values[i] /= g;
            }
        }
        let entries = rowset.entry(indices).or_default();
        for chunk in entries.chunks(N) {
            if chunk == values {
                duplicities += 1;
                rowdelete[r as usize] = true;
                continue 'outer;
            }
        }
        entries.extend_from_slice(&values);
    }

    let mut double_counts = 0;
    let mut more_counts = 0;
    let mut max = 0;
    for (_, c) in rowset {
        let count = c.len() / N;
        if count > 2 {
            more_counts += 1;
        } else if count == 2 {
            double_counts += 1;
        }
        max = max.max(c.len() / N);
    }
    println!(
        "{double_counts} double duplicities, {more_counts} more duplicities, max {max}, duplicate rows {duplicities}"
    );

    let mut shift = 0;
    let rindices: Vec<u32> = (0..(x as u32 + 1))
        .map(|i| {
            if rowdelete[i as usize] {
                shift += 1;
                u32::MAX
            } else {
                i - shift
            }
        })
        .collect();

    let mb = mem::take(m);
    let mut rows = 0;
    *m = mb
        .into_iter()
        .filter_map(|([i, j], s)| {
            if rowdelete[i as usize] {
                None
            } else {
                rows = rindices[i as usize];
                Some(([rows, j], s))
            }
        })
        .collect();
    rows as usize
}

fn read_sms_file<R: BufRead>(reader: &mut R) -> ([usize; 2], Vec<([u32; 2], i32)>) {
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
                numbers.next().unwrap() as i32,
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

fn write_sms_file<W: Write>(dims: [usize; 2], mat: Vec<([u32; 2], i32)>, w: &mut W) {
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

    let file = match File::open(&args.filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error reading {}: {e}", &args.filename);
            return;
        }
    };
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
        if args.twocol {
            let ndims = merge_2cols(&mut lines, dims);
            dbg!(ndims);
            if ndims == dims {
                break;
            }
            dims = ndims;
        } else {
            break;
        }
        // dbg!(&lines);
        println!("merged columns for rows with 2 entries");
        divide_cols(&mut lines, dims);
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

    // let mut nnzs = vec![0; 32];
    // let mut ii = 0;
    // let mut nz = 0;
    // for &([i, _], _) in &lines {
    //     if i != ii {
    //         ii = i;
    //         nnzs[nz] += 1;
    //         nz = 0;
    //     }
    //     nz += 1;
    // }

    dims[0] = delete_duplicate_rows::<3>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<4>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<5>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<6>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<7>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<8>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<9>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<10>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<11>(&mut lines, dims);
    dims[0] = delete_duplicate_rows::<12>(&mut lines, dims);
    count_statistics(&lines, dims);
    println!("Final dimensions: {} {}", dims[0], dims[1]);
    // dbg!(nnzs);
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

    if nc > 1 {
        println!("{nc} block components");
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
    let mut next_graph_idx = 1;
    let mut next_idx = indices[next_graph_idx];
    dbg!((graphs.len(), indices.len(), forests.len()));
    write!(file, "{}", graphs[0]).unwrap();
    for &j in pruned {
        if j >= next_idx {
            loop {
                next_graph_idx += 1;
                next_idx = indices[next_graph_idx];
                if next_idx > j {
                    writeln!(file).unwrap();
                    write!(file, "{}", graphs[next_graph_idx - 1]).unwrap();
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

fn gcd(a: i32, b: i32) -> i32 {
    let (a, b) = (a.abs(), b.abs());
    let mut t = (a.min(b), a.max(b));
    loop {
        if t.0 == 0 {
            return t.1;
        }
        t = (t.1 % t.0, t.0);
    }
}

fn gcd_array<const N: usize>(a: &[i32; N]) -> i32 {
    let mut g = a[0].abs();
    for i in 1..N {
        g = gcd(g, a[i]);
    }
    g
}
