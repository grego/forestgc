use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};

fn main() {
    let mut args = env::args().skip(1);
    let Some(regfile) = args.next() else {
        println!("provide a registry file as the first argument");
        return;
    };
    let (graphs, indices, forests) =
        read_registry(BufReader::new(File::open(regfile).unwrap())).unwrap();
    for line in stdin().lock().lines() {
        let line = line.unwrap();
        let index: usize = match line.parse::<usize>() {
            Ok(i) => i - 1,
            Err(e) => {
                eprintln!("Invalid index: {e}");
                continue;
            }
        };
        let j = indices.iter().cloned().position(|i| i > index).unwrap_or(1) - 1;
        println!("{} {:X}", graphs[j], forests[index]);
    }
}

fn read_registry<R: BufRead>(reader: R) -> io::Result<(Vec<String>, Vec<usize>, Vec<u64>)> {
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
