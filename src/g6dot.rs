mod graph;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use graph::Graph;

fn main() {
    let mut args = env::args().skip(1);
    let Some(filename) = args.next() else {
        eprintln! {"Error: no .g6 file provided as the first argument"};
        return;
    };

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let graphs = reader
        .lines()
        .map(|g6| Graph::from_g6(&g6.unwrap()))
        .collect::<Vec<Graph>>();

    for graph in graphs {
        let g = graph.to_multigraph();
        print!("{}\n\n", g.to_dot());
    }
}
