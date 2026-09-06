use std::error::Error;
use std::{fs, path, process};

use blissfs::fs::{format, write_file};

static USAGE: &str = "mkfs <output.img> <file1> [file2 ...]";
static DEFAULT_BLOCK_COUNT: u32 = 128;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    let output_path = args.get(1).unwrap_or_else(|| {
        eprintln!("{}", USAGE);
        process::exit(1);
    });

    let input_files = &args[2..];
    if input_files.is_empty() {
        eprintln!("{}", USAGE);
        process::exit(1);
    }

    let mut disk = format(DEFAULT_BLOCK_COUNT);

    for input_file in input_files {
        let filename = path::Path::new(input_file)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_else(|| {
                eprintln!("invalid input file path: {}", input_file);
                process::exit(1);
            });
        let data = fs::read(input_file).unwrap_or_else(|e| {
            eprintln!("failed to read {}: {}", input_file, e);
            process::exit(1);
        });
        write_file(&mut disk, filename, &data)?;
    }

    fs::write(output_path, disk).unwrap_or_else(|_| {
        eprintln!("failed to write disk to {}", output_path);
        process::exit(1);
    });

    println!(
        "successfully wrote [{}] to {}",
        input_files.join(", "),
        output_path
    );

    Ok(())
}
