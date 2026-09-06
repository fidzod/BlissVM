use crate::error::FsError;

const MAGIC_NUMBER: u32 = 0xB2155F2D;
pub const BLOCK_SIZE: u32 = 512;

#[derive(Debug, Clone, PartialEq)]
pub struct Superblock {
    block_size: u32,
    inode_count: u32,
    data_start: u32,
    free_blocks: u32,
}

fn write(bytes: &mut [u8], offset: &mut usize, src: &[u8]) {
    bytes[*offset..*offset + src.len()].copy_from_slice(src);
    *offset += src.len();
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> u32 {
    let n = u32::from_be_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    n
}

fn read_byte(bytes: &[u8], offset: &mut usize) -> u8 {
    let n = bytes[*offset];
    *offset += 1;
    n
}

fn read_bytes<const N: usize>(bytes: &[u8], offset: &mut usize) -> [u8; N] {
    let out = bytes[*offset..*offset + N].try_into().unwrap();
    *offset += N;
    out
}

impl Superblock {
    pub fn new(block_count: u32) -> Self {
        Self {
            block_size: BLOCK_SIZE,
            inode_count: 32,
            data_start: 6,
            free_blocks: block_count
        }
    }

    pub fn to_bytes(&self) -> [u8; 20] {
        let mut bytes = [0u8; 20];
        let mut offset = 0;

        write(&mut bytes, &mut offset, &MAGIC_NUMBER.to_be_bytes());
        write(&mut bytes, &mut offset, &self.block_size.to_be_bytes());
        write(&mut bytes, &mut offset, &self.inode_count.to_be_bytes());
        write(&mut bytes, &mut offset, &self.data_start.to_be_bytes());
        write(&mut bytes, &mut offset, &self.free_blocks.to_be_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8; 20]) -> Result<Self, FsError> {
        let mut offset = 0;

        let magic_number = read_u32(bytes, &mut offset);

        if magic_number != MAGIC_NUMBER {
            return Err(FsError::BadMagic);
        }

        Ok(Self {
            block_size: read_u32(bytes, &mut offset),
            inode_count: read_u32(bytes, &mut offset),
            data_start: read_u32(bytes, &mut offset),
            free_blocks: read_u32(bytes, &mut offset),
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InodeType {
    Unused,
    File,
    Directory,
}

impl TryFrom<u8> for InodeType {
    type Error = FsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InodeType::Unused),
            1 => Ok(InodeType::File),
            2 => Ok(InodeType::Directory),
            _ => Err(FsError::InvalidInodeType(value))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inode {
    pub size: u32,
    pub kind: InodeType,
    pub block_pointers: [u32; 8],
}

impl Inode {
    pub fn new(kind: InodeType) -> Self {
        Self {
            size: 0,
            kind,
            block_pointers: [0u32; 8]
        }
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        let mut offset = 0;

        write(&mut bytes, &mut offset, &self.size.to_be_bytes());
        write(&mut bytes, &mut offset, &[self.kind as u8]);
        write(&mut bytes, &mut offset, &[0u8; 3]);

        for block in self.block_pointers {
            write(&mut bytes, &mut offset, &block.to_be_bytes());
        }

        bytes
    }

    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self, FsError> {
        let mut offset = 0;

        let size = read_u32(bytes, &mut offset);
        let kind = InodeType::try_from(read_byte(bytes, &mut offset))?;

        offset += 3;

        let mut block_pointers = [0u32; 8];

        for item in &mut block_pointers {
            *item = read_u32(bytes, &mut offset);
        }

        Ok(Self {
            size,
            kind,
            block_pointers
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirEntry {
    filename: [u8; 24],
    pub inode: u32,
}

impl DirEntry {
    pub fn new(filename: String, inode: u32) -> Result<Self, FsError> {
        if filename.len() > 24 {
            return Err(FsError::NameTooLong(filename))
        }

        Ok(Self {
            filename: {
                let mut buf = [0u8; 24];
                buf[..filename.len()].copy_from_slice(filename.as_bytes());
                buf
            },
            inode
        })
    }

    pub fn to_bytes(&self) -> [u8; 28] {
        let mut bytes = [0u8; 28];
        let mut offset = 0;

        write(&mut bytes, &mut offset, &self.filename);
        write(&mut bytes, &mut offset, &self.inode.to_be_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8; 28]) -> Result<Self, FsError> {
        let mut offset = 0;

        Ok(Self {
            filename: read_bytes(bytes, &mut offset),
            inode: read_u32(bytes, &mut offset)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superblock_round_trip() {
        let superblock = Superblock {
            block_size: 512,
            inode_count: 32,
            data_start: 6,
            free_blocks: 10,
        };

        let to_bytes = superblock.to_bytes();
        let from_bytes = Superblock::from_bytes(&to_bytes).unwrap();
        assert_eq!(from_bytes, superblock);
    }

    #[test]
    fn superblock_bad_magic() {
        let superblock = Superblock {
            block_size: 512,
            inode_count: 32,
            data_start: 6,
            free_blocks: 10,
        };

        let mut to_bytes = superblock.to_bytes();
        to_bytes[0..4].copy_from_slice(&0xBEEF_CAFE_u32.to_be_bytes());

        assert_eq!(Superblock::from_bytes(&to_bytes), Err(FsError::BadMagic));
    }

    #[test]
    fn direntry_round_trip() {
        let mut filename = [0u8; 24];
        let name = b"foo.bar";
        filename[..name.len()].copy_from_slice(name);
        let direntry = DirEntry {
            filename,
            inode: 8
        };

        let to_bytes = direntry.to_bytes();
        let from_bytes = DirEntry::from_bytes(&to_bytes).unwrap();
        assert_eq!(from_bytes, direntry);
    }

    #[test]
    fn inode_file_round_trip() {
        let inode = Inode {
            size: 100,
            kind: InodeType::File,
            block_pointers: [0u32; 8],
        };

        let to_bytes = inode.to_bytes();
        let from_bytes = Inode::from_bytes(&to_bytes).unwrap();
        assert_eq!(from_bytes, inode);
    }

    #[test]
    fn inode_dir_round_trip() {
        let inode = Inode {
            size: 100,
            kind: InodeType::Directory,
            block_pointers: [0u32; 8],
        };

        let to_bytes = inode.to_bytes();
        let from_bytes = Inode::from_bytes(&to_bytes).unwrap();
        assert_eq!(from_bytes, inode);
    }

    #[test]
    fn inode_unused_round_trip() {
        let inode = Inode {
            size: 0,
            kind: InodeType::Unused,
            block_pointers: [0u32; 8],
        };

        let to_bytes = inode.to_bytes();
        let from_bytes = Inode::from_bytes(&to_bytes).unwrap();
        assert_eq!(from_bytes, inode);
    }

    #[test]
    fn inode_invalid() {
        let inode = Inode {
            size: 0,
            kind: InodeType::Unused,
            block_pointers: [0u32; 8],
        };

        let mut to_bytes = inode.to_bytes();
        to_bytes[4] = 3;

        assert_eq!(Inode::from_bytes(&to_bytes), Err(FsError::InvalidInodeType(3)));
    }
}
