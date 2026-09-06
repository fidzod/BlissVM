use std::error::Error;
use std::io::Write;

use blissfs::layout::{BLOCK_SIZE, DirEntry, Inode, Superblock};

static USAGE: &str = "catfs <disk.img> <filename>";

fn main() -> Result<(), Box<dyn Error>> {
    let disk_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("{}", USAGE);
        std::process::exit(1);
    });
    let target = std::env::args().nth(2).unwrap_or_else(|| {
        eprintln!("{}", USAGE);
        std::process::exit(1);
    });

    let disk = std::fs::read(&disk_path).unwrap_or_else(|e| {
        eprintln!("failed to read {}: {}", disk_path, e);
        std::process::exit(1);
    });

    // Validate the magic number.
    Superblock::from_bytes(disk[0..20].try_into()?)?;

    // Root inode is always at index 1.
    let root_inode = Inode::from_bytes(disk[576..640].try_into()?)?;
    let dir_offset = root_inode.block_pointers[0] as usize * BLOCK_SIZE as usize;

    // Search the root directory for the target filename.
    let mut file_inode: Option<Inode> = None;
    for i in 0..18 {
        let entry_addr = dir_offset + i * 28;
        let entry_bytes: [u8; 28] = disk[entry_addr..entry_addr + 28].try_into()?;
        let entry = DirEntry::from_bytes(&entry_bytes)?;

        if entry.inode == 0 {
            break;
        }

        if entry.filename() == target {
            let inode_offset = BLOCK_SIZE as usize + entry.inode as usize * 64;
            file_inode = Some(Inode::from_bytes(disk[inode_offset..inode_offset + 64].try_into()?)?);
            break;
        }
    }

    let inode = file_inode.unwrap_or_else(|| {
        eprintln!("{}: file not found", target);
        std::process::exit(1);
    });

    // Read and print the file data, block by block.
    // Stop at inode.size so the last partial block is not over-read.
    let mut remaining = inode.size as usize;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for &block_num in inode.block_pointers.iter() {
        if block_num == 0 || remaining == 0 {
            break;
        }
        let offset = block_num as usize * BLOCK_SIZE as usize;
        let chunk_len = remaining.min(BLOCK_SIZE as usize);
        out.write_all(&disk[offset..offset + chunk_len])?;
        remaining -= chunk_len;
    }

    Ok(())
}
