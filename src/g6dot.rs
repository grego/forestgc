use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};

use graphc::graph::Graph;

fn main() {
    let mut args = env::args().skip(1);
    if let Some(filename) = args.next() {
        let file = File::open(filename).unwrap();
        g6dot(BufReader::new(file))
    } else {
        g6dot(stdin().lock())
    }
}

fn g6dot(reader: impl BufRead) {
    for line in reader.lines() {
        let line = line.unwrap();
        let g = Graph::from_g6(&line).to_multigraph();
        print!("{}\n\n", g.to_dot());
    }
}
