use std::error::Error;

use blissfs::layout::{BLOCK_SIZE, DirEntry, Inode, Superblock};

static USAGE: &str = "lsfs <disk.img>";

fn main() -> Result<(), Box<dyn Error>> {
    let disk_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("{}", USAGE);
        std::process::exit(1);
    });

    let disk_name = std::path::Path::new(&disk_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_else(|| {
            eprintln!("invalid disk path: {}", disk_path);
            std::process::exit(1);
        });

    let disk = std::fs::read(&disk_path).unwrap_or_else(|e| {
        eprintln!("failed to read {}: {}", disk_path, e);
        std::process::exit(1);
    });

    let superblock = Superblock::from_bytes(disk[0..20].try_into()?)?;

    let root_inode = Inode::from_bytes(disk[576..640].try_into()?)?;
    let root_dir_block = root_inode.block_pointers[0];
    let dir_offset = root_dir_block as usize * BLOCK_SIZE as usize;

    // Collect all entries first so we can measure column widths.
    let mut entries: Vec<(String, u32)> = Vec::new();
    for i in 0..18 {
        let entry_addr = dir_offset + i * 28;
        let entry_bytes: [u8; 28] = disk[entry_addr..entry_addr + 28].try_into()?;
        let entry = DirEntry::from_bytes(&entry_bytes)?;

        if entry.inode == 0 {
            continue;
        }

        let inode_offset = BLOCK_SIZE as usize + entry.inode as usize * 64;
        let inode = Inode::from_bytes(disk[inode_offset..inode_offset + 64].try_into()?)?;
        entries.push((entry.filename().to_owned(), inode.size));
    }

    let name_width = entries.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    let total_bytes: u32 = entries.iter().map(|(_, s)| s).sum();
    let sep = "-".repeat(name_width + 14);

    println!("{} - {} free blocks", disk_name, superblock.free_blocks);
    println!("{}", sep);

    for (name, size) in &entries {
        println!("  {:<name_width$}  {:>6} B", name, size, name_width = name_width);
    }

    println!("{}", sep);
    println!("  {} file{}, {} B total",
        entries.len(),
        if entries.len() == 1 { "" } else { "s" },
        total_bytes,
    );

    Ok(())
}
