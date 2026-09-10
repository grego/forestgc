use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

use rayon::prelude::*;

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
            let mut numbers = s.split_whitespace().map(|i| i.parse::<i32>().unwrap());
            [
                numbers.next().unwrap(),
                numbers.next().unwrap(),
                numbers.next().unwrap(),
            ]
        })
        .collect::<Vec<_>>();
    lines.pop();
    println!("Read {}", filename);

    let t = Instant::now();
    lines.par_sort_unstable();
    println!("Sorted in {:.3}ms", t.elapsed().as_secs_f64() * 1e3);

    let file = File::create(filename).unwrap();
    let mut file = BufWriter::with_capacity(500_000_000, file);
    writeln!(&mut file, "{first}").unwrap();
    for line in lines {
        writeln!(&mut file, "{} {} {}", line[0], line[1], line[2]).unwrap();
    }
    writeln!(&mut file, "0 0 0").unwrap();
}
