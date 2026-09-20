use std::cmp::Reverse;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use rayon::prelude::*;

struct Csr {
    indices: Vec<u32>,
    values: Vec<i16>,
    rows: Vec<u32>,
    _columns: usize,
}

impl Csr {
    fn from_coo(coo: impl Iterator<Item = ([u32; 2], i16)>, [h, w]: [usize; 2]) -> Self {
        let (mut indices, mut values) = (Vec::new(), Vec::new());
        let mut rows = vec![0; h + 1];
        let mut r = 0;
        let mut k = 0;
        for ([i, j], s) in coo {
            indices.push(j - 1);
            values.push(s);
            if i - 1 > r {
                r = i - 1;
                rows[r as usize] = k as u32;
            }
            k += 1;
        }
        rows[h] = k as u32;

        Self {
            indices,
            values,
            rows,
            _columns: w,
        }
    }

    fn _multiply_vectors(&self, vs: &[i16], chunk_size: usize) -> Vec<i16> {
        (0..(self.rows.len() - 1))
            .into_par_iter()
            .map(|r| {
                let mut res = vec![0; chunk_size];
                for i in self.rows[r]..self.rows[r + 1] {
                    let idx = self.indices[i as usize] as usize;
                    let v = &vs[idx * chunk_size..(idx + 1) * chunk_size];
                    let val = self.values[i as usize];
                    for k in 0..chunk_size {
                        res[k] += val * v[k];
                    }
                }
                res
            })
            .flatten()
            .collect()
    }

    fn multiplies_to_zero(&self, vs: &[i16], chunk_size: usize) -> u64 {
        (0..(self.rows.len() - 1))
            .into_par_iter()
            .fold(
                || 0,
                |mut m, r| {
                    let mut res = vec![0; chunk_size];
                    for i in self.rows[r]..self.rows[r + 1] {
                        let idx = self.indices[i as usize] as usize;
                        let v = &vs[idx * chunk_size..(idx + 1) * chunk_size];
                        let val = self.values[i as usize];
                        for k in 0..chunk_size {
                            res[k] += val * v[k];
                        }
                    }
                    for (k, &rs) in res.iter().enumerate() {
                        if rs != 0 {
                            m |= 1 << k;
                        }
                    }
                    m
                },
            )
            .reduce(|| 0, |m, n| m | n)
    }
}

fn read_sms_file<R: BufRead>(
    reader: &mut R,
) -> ([usize; 2], impl Iterator<Item = ([u32; 2], i16)>) {
    let mut lines = reader.lines();
    let first = lines.next().unwrap().unwrap();
    let lines = lines
        .map(|s| {
            let s = s.unwrap();
            let mut numbers = s.split_whitespace().map(|i| i.parse::<i64>().unwrap());
            (
                [
                    numbers.next().unwrap() as u32,
                    numbers.next().unwrap() as u32,
                ],
                numbers.next().unwrap().try_into().unwrap(),
            )
        })
        .filter(|&([i, j], s)| i != 0 && j != 0 && s != 0);

    let mut dims_iter = first
        .split_whitespace()
        .map(|i| i.parse::<usize>().unwrap());
    let dims = [dims_iter.next().unwrap(), dims_iter.next().unwrap()];

    (dims, lines)
}

const PRIMES: [u16; 16] = [
    43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109,
];

fn read_nullvector_file<R: BufRead>(
    reader: &mut R,
    prime: u16,
    max: Option<usize>,
) -> (Vec<u16>, usize) {
    let mut rows = Vec::new();
    for (k, s) in reader.lines().enumerate() {
        if let Some(m) = max
            && k >= m
        {
            break;
        }
        let s = s.unwrap();
        let row: Vec<_> = s
            .split_whitespace()
            .map(|i| i.parse::<u16>().unwrap())
            .collect();
        rows.push(row);
    }
    let mut rows = Rows::new(rows, prime);
    rows.eliminate();
    rows.print_first(10);

    let mut nonzeros = rows.rows.len();
    for (i, row) in rows.rows.iter().enumerate() {
        let mut support = 0;
        for &r in row {
            support += (r != 0) as usize;
        }
        if support == 0 {
            nonzeros = i;
            break;
        }
        println!("{i} nnzs: {support}")
    }

    let mut tr = vec![0; rows.rows[0].len()];
    for (i, row) in rows.rows.iter().enumerate() {
        for (k, &r) in row.iter().enumerate() {
            tr[k] = (tr[k] + r * PRIMES[15 - i]) % prime;
        }
    }
    let s: usize = tr.into_iter().map(|r| (r != 0) as usize).sum();
    println!("big sum: {s}");

    let mut v = vec![0; nonzeros * rows.rows[0].len()];
    for i in 0..nonzeros {
        for j in 0..rows.rows[i].len() {
            v[j * nonzeros + i] = rows.rows[i][j];
        }
    }

    (v, nonzeros)
}

const USAGE: &str = "Usage: representatives <.sms file> <nullvector file> <modulus>";

fn main() {
    let mut args = env::args().skip(1);
    let Some(filename) = args.next() else {
        eprintln!("{USAGE}");
        return;
    };
    let Some(nullfilename) = args.next() else {
        eprintln!("{USAGE}");
        return;
    };
    let Some(prime) = args.next().and_then(|p| p.parse::<u16>().ok()) else {
        eprintln!("{USAGE}");
        return;
    };
    let maxvecs = args.next().and_then(|p| p.parse::<usize>().ok());

    let file = File::open(&nullfilename).unwrap();
    let mut reader = BufReader::with_capacity(500_000_000, file);
    let (v, nonzeros) = read_nullvector_file(&mut reader, prime, maxvecs);
    let breadth = nonzeros;
    dbg!(breadth);
    let mut nonzeros = (1 << nonzeros) - 1;

    let file = File::open(&filename).unwrap();
    let mut reader = BufReader::with_capacity(500_000_000, file);
    let (dims, lines) = read_sms_file(&mut reader);
    let csr = Csr::from_coo(lines, dims);
    println!("Read {filename}");

    let mut coeffs = vec![0; breadth];
    for k in 1..(prime / 2) {
        let mut w = vec![0; v.len()];
        for (i, &n) in v.iter().enumerate() {
            // let n = n.rem_euclid(PRIME);
            let mut m = multp(k, n, prime) as i16;
            if 2 * m > prime as i16 {
                m -= prime as i16;
            }
            w[i] = m;
        }
        println!("trying {k}-multiples");
        let nz = csr.multiplies_to_zero(&w, breadth);
        for (j, c) in coeffs.iter_mut().enumerate() {
            if (!nz & nonzeros & (1 << j)) != 0 {
                println!("vec {j} zero by {k}");
                *c = k;
            }
        }
        nonzeros &= nz;
        if nonzeros == 0 {
            break;
        }
    }

    let intfilename = format!("{nullfilename}_integer");
    let file = File::create(&intfilename).unwrap();
    let mut writer = std::io::BufWriter::with_capacity(500_000_000, file);
    for i in 0..(v.len() / breadth) {
        for j in 0..breadth {
            let mut a = v[i * breadth + j];
            a = multp(a, coeffs[j], prime);
            let mut a = a as i16;
            if 2 * a > prime as i16 {
                a -= prime as i16;
            }
            write!(&mut writer, "{a} ").unwrap();
        }
        writeln!(&mut writer).unwrap();
    }
    writer.flush().unwrap();
    println!("{intfilename} written");
}

pub struct Rows {
    pub rows: Vec<Vec<u16>>,
    pub prime: u16,
}

impl Rows {
    pub fn new(rows: Vec<Vec<u16>>, prime: u16) -> Self {
        assert_ne!(rows.len(), 0);
        for row in &rows {
            assert_eq!(row.len(), rows[0].len());
        }
        Self { rows, prime }
    }

    pub fn multiply_row(&mut self, i: usize, a: u16) {
        for b in &mut self.rows[i] {
            *b = multp(a, *b, self.prime);
        }
    }

    pub fn add_row(&mut self, (i, a): (usize, u16), j: usize) {
        for k in 0..self.rows[i].len() {
            self.rows[j][k] = ((self.rows[j][k] as u32
                + multp(self.rows[i][k], a, self.prime) as u32)
                % self.prime as u32) as u16;
        }
    }

    pub fn eliminate(&mut self) {
        let (mut row, mut col) = (0, 0);
        while row < self.rows.len() && col < self.rows[0].len() {
            self.rows[row..].sort_unstable_by_key(|r| Reverse(r[col]));
            let pivot = self.rows[row][col];
            if pivot == 0 {
                col += 1;
                continue;
            }

            let Some(inv) = mod_inverse(pivot, self.prime) else {
                col += 1;
                continue;
            };
            self.multiply_row(row, inv);
            for i in 0..self.rows.len() {
                let p = self.rows[i][col];
                if p == 0 || i == row {
                    continue;
                }
                self.add_row((row, self.prime - p), i);
            }
            col += 1;
            row += 1;
        }
    }

    pub fn print_first(&self, k: usize) {
        for row in &self.rows {
            for v in &row[..k] {
                print!("{v} ");
            }
            println!();
        }
    }
}

// fn square_and_multiply(a: u16, mut n: u16, p: u16) -> u16 {
//     let mut res: u32 = 1;
//     let mut x: u32 = a as u32;
//     let p = p as u32;
//     while n != 0 {
//         if n & 1 != 0 {
//             res = (res * x) % p;
//         }
//         x = (x * x) % p;
//         n = n >> 1;
//     }
//     res as u16
// }

fn extended_euclid(a: u16, b: u16) -> (u16, i16, i16) {
    if b == 0 {
        return (a, 1, 0);
    }
    let (gcd, x1, y1) = extended_euclid(b, a % b);
    let x = y1;
    let y = x1 - (a / b) as i16 * y1;
    (gcd, x, y)
}

pub fn mod_inverse(a: u16, p: u16) -> Option<u16> {
    let (gcd, _, mut x) = extended_euclid(p, a);
    if gcd != 1 {
        return None;
    }
    if x < 0 {
        x += p as i16;
    }
    // let y = square_and_multiply(a, p - 2, p);
    // assert_eq!(x as u16, y);
    Some(x as u16)
}

pub fn multp(a: u16, b: u16, p: u16) -> u16 {
    ((a as u32 * b as u32) % p as u32) as u16
}

#[cfg(test)]
pub mod test {
    use crate::Rows;

    #[test]
    fn simple_elim() {
        let v = vec![vec![1, 0, 1], vec![0, 2, 1], vec![0, 1, 1]];
        let mut rows = Rows::new(v, 11);
        rows.eliminate();
        rows.print_first(3);
        assert_eq!(rows.rows, vec![vec![1, 0, 0], vec![0, 1, 0], vec![0, 0, 1]]);
    }
}
