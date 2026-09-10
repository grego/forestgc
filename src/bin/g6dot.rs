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
        let g = Graph::from_g6(&line);
        print!("{}\n\n", edges_to_dot(g.multigraph_edges()));
        // print!("{}\n", g.simplify(true).0.to_g6());
    }
}

/// Output the graph in the graphviz dot format
fn edges_to_dot(edges: impl IntoIterator<Item = (u8, u8)>) -> String {
    let mut s = "graph {\n".to_string();
    for (v, w) in edges {
        s.push_str(&format!("{v} -- {w}\n"));
    }
    s.push('}');
    s
}
