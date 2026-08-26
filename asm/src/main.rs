use std::{fs, path::PathBuf};

fn main() {
    let source_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: asm <source.asm>");
        std::process::exit(1)
    });

    let out_path = PathBuf::from(source_path.clone()).with_extension("bin");

    let source = std::fs::read_to_string(source_path.clone()).unwrap_or_else(|e| {
        eprintln!("Error opening file '{}': {}", source_path, e);
        std::process::exit(1)
    });

    let assembled = asm::assemble(&source).unwrap_or_else(|e| {
        eprintln!("Error during assembly: {:?}", e);
        std::process::exit(1)
    });

    fs::write(out_path.clone(), assembled).unwrap_or_else(|e| {
        eprintln!("Error writing binary: {}", e);
        std::process::exit(1)
    });

    println!("Assembled {} -> {}", source_path, out_path.display());
}
