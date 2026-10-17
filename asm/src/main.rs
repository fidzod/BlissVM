use std::{fs, path::PathBuf};

fn main() {
    let source_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: asm <source.asm>");
        std::process::exit(1)
    });

    let pic_mode = match std::env::args().nth(2).as_deref() {
        Some("-pic") => true,
        Some(_) => {
            eprintln!("Unknown flag");
            std::process::exit(1)
        }
        None => false,
    };

    let out_path = PathBuf::from(source_path.clone()).with_extension("bin");

    let source = std::fs::read_to_string(source_path.clone()).unwrap_or_else(|e| {
        eprintln!("Error opening file '{}': {}", source_path, e);
        std::process::exit(1)
    });

    let source_pathbuf = PathBuf::from(&source_path);
    let base_dir = source_pathbuf
        .parent()
        .expect("source path has no parent dir");
    let preprocessed = asm::preprocessor::expand(&source, base_dir).unwrap_or_else(|e| {
        eprintln!("Error during preprocessing: {:?}", e);
        std::process::exit(1)
    });

    let assembled = asm::assemble(&preprocessed, pic_mode).unwrap_or_else(|e| {
        eprintln!("Error during assembly: {:?}", e);
        std::process::exit(1)
    });

    fs::write(out_path.clone(), assembled).unwrap_or_else(|e| {
        eprintln!("Error writing binary: {}", e);
        std::process::exit(1)
    });

    println!(
        "Assembled {} -> {} {}",
        source_path,
        out_path.display(),
        (if pic_mode { "(pic)" } else { "" })
    );
}
