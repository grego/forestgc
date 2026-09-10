use argh::FromArgs;
use std::fs;
use std::path::Path;

/// Scale a vector to an expanded typed basis.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
struct Args {
    /// file containing vectors, one per line
    #[argh(positional)]
    vector_file: String,

    /// optional basis file
    #[argh(positional)]
    basis_file: Option<String>,

    /// optional index (1-based)
    #[argh(option, short = 'i')]
    index: Option<usize>,
}

fn parse_vector(contents: &str) -> Vec<i64> {
    contents
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(' ')
        .map(|x| x.trim().parse::<i64>().expect("Invalid integer"))
        .collect()
}

// Returns a vector of (type, element) pairs, in order
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

fn format_typed_basis(basis: &[(String, String)]) -> String {
    let mut result = String::new();
    let mut current_type: Option<&String> = None;

    for (ty, elem) in basis {
        if current_type != Some(ty) {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(ty);
            current_type = Some(ty);
        }
        result.push(' ');
        result.push_str(elem);
    }

    result
}

fn make_suffixed_filename(input_path: &str, suffix: &str) -> String {
    let path = Path::new(input_path);
    let parent = path.parent().unwrap_or(Path::new(""));

    let stem = path
        .file_stem()
        .expect("Invalid file name")
        .to_string_lossy();

    let new_name = match path.extension() {
        Some(ext) => format!("{}_{}.{}", stem, suffix, ext.to_string_lossy()),
        None => format!("{}_{}", stem, suffix),
    };

    parent.join(new_name).to_string_lossy().to_string()
}

fn main() {
    let args: Args = argh::from_env();

    if args.index.is_some() && args.basis_file.is_none() {
        panic!("--index/-i requires basis_file");
    }

    let vector_contents =
        fs::read_to_string(&args.vector_file).expect("Failed to read vector file");

    let vectors: Vec<Vec<i64>> = vector_contents
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(parse_vector)
        .collect();

    let mut supp_lines = Vec::new();

    for (i, v) in vectors.iter().enumerate() {
        let supp: Vec<i64> = v.iter().cloned().filter(|x| *x != 0).collect();
        println!("Vector {} has support length {}", i + 1, supp.len());
        supp_lines.push(format_vector(&supp));
    }

    let supp_vector_file = make_suffixed_filename(&args.vector_file, "supp");
    fs::write(&supp_vector_file, supp_lines.join("\n")).expect("Failed to write support vectors");
    println!("Support vectors written to {}", supp_vector_file);

    // Optional support basis for the n-th vector
    if let (Some(basis_file), Some(n)) = (&args.basis_file, args.index) {
        if n == 0 || n > vectors.len() {
            panic!("Index out of range");
        }

        let basis_contents = fs::read_to_string(basis_file).expect("Failed to read basis file");
        let basis = parse_typed_basis(&basis_contents);
        let vector = &vectors[n - 1];

        if vector.len() != basis.len() {
            panic!(
                "Vector length ({}) does not match basis size ({})",
                vector.len(),
                basis.len()
            );
        }

        let mut supp_basis = Vec::new();
        for ((ty, elem), coeff) in basis.into_iter().zip(vector.iter()) {
            if *coeff != 0 {
                supp_basis.push((ty, elem));
            }
        }

        let suffix = format!("supp_{}", n);
        let supp_basis_file = make_suffixed_filename(basis_file, &suffix);
        fs::write(&supp_basis_file, format_typed_basis(&supp_basis))
            .expect("Failed to write support basis");

        println!("Support basis written to {}", supp_basis_file);
    }
}
