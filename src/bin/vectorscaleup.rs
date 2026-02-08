use argh::FromArgs;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Scale vectors to an expanded typed basis.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
struct Args {
    /// input vector file (one per line)
    #[argh(positional)]
    vector_file: String,

    /// original basis file
    #[argh(positional)]
    basis_file: String,

    /// expanded basis file
    #[argh(positional)]
    expanded_basis_file: String,
}

fn parse_vector(line: &str) -> Vec<i64> {
    line.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|x| x.trim().parse::<i64>().expect("Invalid integer"))
        .collect()
}

fn parse_typed_basis(contents: &str) -> Vec<(String, String)> {
    let mut basis = Vec::new();
    for line in contents.lines() {
        let mut parts = line.split_whitespace();
        let ty = parts.next().expect("Empty line in basis file").to_string();
        for elem in parts {
            basis.push((ty.clone(), elem.to_string()));
        }
    }
    basis
}

fn format_vector(vec: &[i64]) -> String {
    let mut s = String::from("[");
    for (i, val) in vec.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&val.to_string());
    }
    s.push(']');
    s
}

fn validate_expanded_basis_contains_original(
    basis: &[(String, String)],
    expanded_basis: &[(String, String)],
) {
    let expanded_set: HashSet<(String, String)> = expanded_basis.iter().cloned().collect();

    let missing: Vec<_> = basis
        .iter()
        .filter(|&e| !expanded_set.contains(e))
        .cloned()
        .collect();

    if !missing.is_empty() {
        eprintln!("Error: expanded basis is missing elements:");
        for (ty, elem) in missing {
            eprintln!("  ({}, {})", ty, elem);
        }
        std::process::exit(1);
    }
}

fn make_scaled_filename(vector_file: &str, expanded_basis_file: &str) -> String {
    let path = Path::new(vector_file);
    let parent = path.parent().unwrap_or(Path::new(""));
    let stem = path.file_stem().unwrap().to_string_lossy();
    let basis_name = Path::new(expanded_basis_file)
        .file_stem()
        .unwrap()
        .to_string_lossy();

    let ext = path.extension().map(|e| e.to_string_lossy());
    let name = match ext {
        Some(e) => format!("{}_scaled_to_{}.{}", stem, basis_name, e),
        None => format!("{}_scaled_to_{}", stem, basis_name),
    };

    parent.join(name).to_string_lossy().to_string()
}

fn main() {
    let args: Args = argh::from_env();

    let vector_contents =
        fs::read_to_string(&args.vector_file).expect("Failed to read vector file");
    let basis_contents = fs::read_to_string(&args.basis_file).expect("Failed to read basis file");
    let expanded_basis_contents =
        fs::read_to_string(&args.expanded_basis_file).expect("Failed to read expanded basis file");

    let vectors: Vec<Vec<i64>> = vector_contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(parse_vector)
        .collect();

    let basis = parse_typed_basis(&basis_contents);
    let expanded_basis = parse_typed_basis(&expanded_basis_contents);

    validate_expanded_basis_contains_original(&basis, &expanded_basis);

    let mut output_lines = Vec::new();

    for v in vectors {
        if v.len() != basis.len() {
            panic!(
                "Vector length ({}) does not match basis size ({})",
                v.len(),
                basis.len()
            );
        }

        let mut coeff_map: HashMap<(String, String), i64> = HashMap::new();
        for (key, coeff) in basis.iter().cloned().zip(v.iter()) {
            coeff_map.insert(key, *coeff);
        }

        let scaled: Vec<i64> = expanded_basis
            .iter()
            .map(|k| *coeff_map.get(k).unwrap_or(&0))
            .collect();

        output_lines.push(format_vector(&scaled));
    }

    let output_file = make_scaled_filename(&args.vector_file, &args.expanded_basis_file);
    fs::write(&output_file, output_lines.join("\n")).expect("Failed to write scaled vectors");

    println!("Scaled vectors written to {}", output_file);
}
