use crate::error::FsError;
use crate::layout::{BLOCK_SIZE, DirEntry, Inode, InodeType, Superblock};

pub fn format(block_count: u32) -> Vec<u8> {
    let mut disk = vec![0u8; block_count as usize * BLOCK_SIZE as usize];

    let superblock = Superblock::new(block_count - 7);
    disk[0..20].copy_from_slice(&superblock.to_bytes());

    // Mark the first data block (block 6) as used — it holds the root directory.
    disk[BLOCK_SIZE as usize * 5] |= 1;

    let mut root_inode = Inode::new(InodeType::Directory);
    root_inode.block_pointers[0] = 6;
    let root_inode_start = BLOCK_SIZE as usize + 64;
    disk[root_inode_start..root_inode_start + 64].copy_from_slice(&root_inode.to_bytes());

    disk
}

// Returns the inode *index* (not a byte offset). Inode 0 is the reserved null
// inode, so the search starts at 1. Byte offset = BLOCK_SIZE + index * 64.
fn find_free_inode(disk: &[u8]) -> Result<u32, FsError> {
    for i in 1..32usize {
        let inode_start = BLOCK_SIZE as usize + i * 64;
        // Byte 4 of a serialised Inode is the `kind` field; 0 == InodeType::Unused.
        if disk[inode_start + 4] == 0 {
            return Ok(i as u32);
        }
    }
    Err(FsError::OutOfFreeInodes)
}

// Returns a data *block number* (not a byte offset). Callers multiply by
// BLOCK_SIZE when they need a byte address.
fn find_free_block(disk: &mut [u8]) -> Option<u32> {
    let bitmap_start = BLOCK_SIZE as usize * 5;
    for byte_index in 0..512usize {
        for bit in 0..8u8 {
            if (disk[bitmap_start + byte_index] & (1 << bit)) == 0 {
                disk[bitmap_start + byte_index] |= 1 << bit;
                let data_block_index = byte_index * 8 + bit as usize;
                return Some(data_block_index as u32 + 6);
            }
        }
    }
    None
}

pub fn write_file(disk: &mut [u8], name: &str, data: &[u8]) -> Result<(), FsError> {
    if name.len() > 24 {
        return Err(FsError::NameTooLong(name.to_string()));
    }
    if data.len() > 8 * BLOCK_SIZE as usize {
        return Err(FsError::FileTooLarge);
    }

    let free_inode = find_free_inode(disk)?;

    // Allocate one data block per 512-byte chunk of the file.
    let required_blocks = data.len().div_ceil(BLOCK_SIZE as usize);
    let mut block_nums: Vec<u32> = Vec::with_capacity(required_blocks);
    while block_nums.len() < required_blocks {
        match find_free_block(disk) {
            Some(n) => block_nums.push(n),
            None => return Err(FsError::OutOfFreeBlocks),
        }
    }

    // Write data, clamping the last chunk to the actual remaining bytes.
    for (i, &block_num) in block_nums.iter().enumerate() {
        let disk_offset = block_num as usize * BLOCK_SIZE as usize;
        let data_start = i * BLOCK_SIZE as usize;
        let data_end = data.len().min((i + 1) * BLOCK_SIZE as usize);
        let chunk = &data[data_start..data_end];
        disk[disk_offset..disk_offset + chunk.len()].copy_from_slice(chunk);
    }

    // Write the inode with its size and block pointers populated.
    let inode_addr = BLOCK_SIZE as usize + free_inode as usize * 64;
    let mut inode = Inode::new(InodeType::File);
    inode.size = data.len() as u32;
    for (i, &bn) in block_nums.iter().enumerate() {
        inode.block_pointers[i] = bn;
    }
    disk[inode_addr..inode_addr + 64].copy_from_slice(&inode.to_bytes());

    // Add a directory entry to the root directory.
    // Root inode is always at index 1; read its first block pointer to find the dir data.
    let inode_1_bytes: [u8; 64] = disk[BLOCK_SIZE as usize + 64..BLOCK_SIZE as usize + 128]
        .try_into()
        .expect("root inode slice is exactly 64 bytes");
    let root_inode = Inode::from_bytes(&inode_1_bytes)?;
    let root_dir_block = root_inode.block_pointers[0];

    let new_entry = DirEntry::new(name.to_string(), free_inode)?;

    let mut found_slot = false;
    for i in 0..18 {
        let entry_addr = root_dir_block as usize * BLOCK_SIZE as usize + i * 28;
        let entry_bytes: [u8; 28] = disk[entry_addr..entry_addr + 28]
            .try_into()
            .expect("dir entry slice is exactly 28 bytes");
        let existing = DirEntry::from_bytes(&entry_bytes)?;
        if existing.inode == 0 {
            disk[entry_addr..entry_addr + 28].copy_from_slice(&new_entry.to_bytes());
            found_slot = true;
            break;
        }
    }
    if !found_slot {
        return Err(FsError::DirectoryFull)
    }

    // Update the free-block count in the superblock.
    let free_before = u32::from_be_bytes(disk[16..20].try_into().unwrap());
    disk[16..20].copy_from_slice(&(free_before - required_blocks as u32).to_be_bytes());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{DirEntry, Inode, InodeType, Superblock};

    const DISK_BLOCKS: u32 = 64;

    #[test]
    fn format_baseline() {
        let disk = format(DISK_BLOCKS);

        // Superblock parses without error (magic number is correct).
        Superblock::from_bytes(disk[0..20].try_into().unwrap()).unwrap();
        // free_blocks is at superblock bytes 16–19: total blocks minus the 6
        // reserved blocks minus 1 for the pre-allocated root dir data block.
        let free_blocks = u32::from_be_bytes(disk[16..20].try_into().unwrap());
        assert_eq!(free_blocks, DISK_BLOCKS - 7);

        // Bitmap (block 5): bit 0 must be set — disk block 6 holds the root dir.
        assert_eq!(disk[BLOCK_SIZE as usize * 5] & 1, 1);

        // Root inode (index 1): Directory pointing to block 6.
        let inode_bytes: [u8; 64] = disk[BLOCK_SIZE as usize + 64..BLOCK_SIZE as usize + 128]
            .try_into().unwrap();
        let root = Inode::from_bytes(&inode_bytes).unwrap();
        assert_eq!(root.kind, InodeType::Directory);
        assert_eq!(root.block_pointers[0], 6);
    }

    #[test]
    fn write_file_roundtrip() {
        let mut disk = format(DISK_BLOCKS);
        let data = b"hello world";
        write_file(&mut disk, "hello.txt", data).unwrap();

        // Inode 2 is the first free slot (inode 0 is null, inode 1 is root dir).
        let inode_addr = BLOCK_SIZE as usize + 2 * 64;
        let inode_bytes: [u8; 64] = disk[inode_addr..inode_addr + 64].try_into().unwrap();
        let file_inode = Inode::from_bytes(&inode_bytes).unwrap();

        assert_eq!(file_inode.kind, InodeType::File);
        assert_eq!(file_inode.size, data.len() as u32);

        // Block 6 is pre-allocated to root dir, so the first free data block is 7.
        let data_block = file_inode.block_pointers[0];
        assert_eq!(data_block, 7);

        // Data lives at block_num * BLOCK_SIZE.
        let disk_offset = data_block as usize * BLOCK_SIZE as usize;
        assert_eq!(&disk[disk_offset..disk_offset + data.len()], data as &[u8]);

        // Root dir (block 6): slot 0 should hold the new entry.
        let entry_addr = 6 * BLOCK_SIZE as usize;
        let entry_bytes: [u8; 28] = disk[entry_addr..entry_addr + 28].try_into().unwrap();
        let entry = DirEntry::from_bytes(&entry_bytes).unwrap();
        assert_eq!(entry.inode, 2);

        // Superblock free_blocks decremented by one data block.
        let free_blocks = u32::from_be_bytes(disk[16..20].try_into().unwrap());
        assert_eq!(free_blocks, DISK_BLOCKS - 7 - 1);
    }
}
