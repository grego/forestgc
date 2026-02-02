use argh::FromArgs;
use std::fs;
use std::path::Path;
use std::collections::{HashMap, HashSet};


/// Scale a vector to an expanded typed basis.
#[derive(FromArgs)]
#[argh(help_triggers("-h", "--help", "help"))]
struct Args {
    /// input vector file (e.g. vector.txt)
    #[argh(positional)]
    vector_file: String,

    /// original basis file
    #[argh(positional)]
    basis_file: String,

    /// expanded basis file
    #[argh(positional)]
    expanded_basis_file: String,
}

fn parse_vector(contents: &str) -> Vec<i64> {
    contents
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
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

fn validate_expanded_basis_contains_original( basis: &[(String, String)], expanded_basis: &[(String, String)],) {
        let expanded_set: HashSet<(String, String)> = expanded_basis.iter().cloned().collect();

        let missing: Vec<(String, String)> = basis.iter().cloned().filter(|elem| !expanded_set.contains(elem)).collect();
        if !missing.is_empty() {
            eprintln!("Error: expanded basis is missing the following elements:");
            for (ty, elem) in missing {
                eprintln!("  ({}, {})", ty, elem);
            }
            std::process::exit(1);
        }
}

fn main() {
    let args: Args = argh::from_env();

    let vector_file = &args.vector_file;
    let basis_file = &args.basis_file;
    let expanded_basis_file = &args.expanded_basis_file;

    let vector_contents = fs::read_to_string(vector_file).expect("Failed to read vector file");
    let basis_contents = fs::read_to_string(basis_file).expect("Failed to read basis file");
    let expanded_basis_contents =
        fs::read_to_string(expanded_basis_file).expect("Failed to read expanded basis file");

    let coefficients = parse_vector(&vector_contents);
    let basis = parse_typed_basis(&basis_contents);
    let expanded_basis = parse_typed_basis(&expanded_basis_contents);

    if coefficients.len() != basis.len() {
        panic!(
            "Vector length ({}) does not match basis size ({})",
            coefficients.len(),
            basis.len()
        );
    }

    // Validate that file 3 contains file 2
    validate_expanded_basis_contains_original(&basis, &expanded_basis);

    // Compute support (non-zero part)
    let mut supp_coeffs = Vec::new();
    let mut supp_basis = Vec::new();

    for ((ty, elem), coeff) in basis.iter().cloned().zip(coefficients.iter()) {
        if *coeff != 0 {
            supp_coeffs.push(*coeff);
            supp_basis.push((ty, elem));
        }
    }

    println!("The vector has support: {}", supp_coeffs.len());

    // Write support vector
    let supp_vector_file = make_suffixed_filename(vector_file, "supp");
    let supp_vector_string = format_vector(&supp_coeffs);

    fs::write(&supp_vector_file, supp_vector_string).expect("Failed to write support vector file");

    println!("Support vector written to {}", supp_vector_file);

    // Write support basis
    let supp_basis_file = make_suffixed_filename(basis_file, "supp");
    let supp_basis_string = format_typed_basis(&supp_basis);

    fs::write(&supp_basis_file, supp_basis_string).expect("Failed to write support basis file");

    println!("Support basis written to {}", supp_basis_file);

    // Scale up vector
    let mut coeff_map: HashMap<(String, String), i64> = HashMap::new();
    for (key, coeff) in basis.into_iter().zip(coefficients.into_iter()) {
        coeff_map.insert(key, coeff);
    }

    let output: Vec<i64> = expanded_basis
        .iter()
        .map(|key| *coeff_map.get(key).unwrap_or(&0))
        .collect();

    let output_string = {
        let mut s = String::from("[");
        for (i, val) in output.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&val.to_string());
        }
        s.push(']');
        s
    };

    let output_file = make_suffixed_filename(vector_file, "scaled");
    fs::write(&output_file, output_string).expect("Failed to write output vector file");

    println!("Scaled vector written to {}", output_file);
}
