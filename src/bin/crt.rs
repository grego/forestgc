use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

fn read_vectors(name: &str) -> Vec<Vec<i32>> {
    let file = File::open(name).unwrap();
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for s in reader.lines() {
        let s = s.unwrap();
        let row: Vec<_> = s
            .split_whitespace()
            .map(|i| i.parse::<i32>().unwrap())
            .collect();
        rows.push(row);
    }
    rows
}

fn extended_euclid(a: i32, b: i32) -> (i32, i32, i32) {
    if b == 0 {
        return (a, 1, 0);
    }
    let (gcd, x1, y1) = extended_euclid(b, a % b);
    let x = y1;
    let y = x1 - (a / b) * y1;
    return (gcd, x, y);
}

const USAGE: &str = "Usage: program <vectors 1> <prime 1> <vectors 2> <prime 2> <output>";

fn main() {
    let mut args = env::args().skip(1);
    let Some(vecname1) = args.next() else {
        eprintln!("{USAGE}");
        return;
    };
    let Some(prime1) = args.next().and_then(|p| p.parse::<i32>().ok()) else {
        eprintln!("{USAGE}");
        return;
    };
    let Some(vecname2) = args.next() else {
        eprintln!("{USAGE}");
        return;
    };
    let Some(prime2) = args.next().and_then(|p| p.parse::<i32>().ok()) else {
        eprintln!("{USAGE}");
        return;
    };
    let Some(output) = args.next() else {
        eprintln!("{USAGE}");
        return;
    };

    let vecs1 = read_vectors(&vecname1);
    println!("read 1");
    let vecs2 = read_vectors(&vecname2);
    println!("read 2");

    let out = File::create(&output).unwrap();
    let mut out = BufWriter::new(out);

    let (gcd, x, y) = extended_euclid(prime1, prime2);
    assert_eq!(gcd, 1);
    let prod = prime1 * prime2;
    for v in &vecs1 {
        for w in vecs2.iter() {
            assert_eq!(v.len(), w.len());
            let mut u = Vec::with_capacity(v.len());
            for i in 0..v.len() {
                let a = (x * prime1 * w[i] + y * prime2 * v[i]).rem_euclid(prod);
                u.push(a);
            }

            for x in u {
                write!(out, "{x} ").unwrap();
            }
            writeln!(out).unwrap();
        }
    }
}
