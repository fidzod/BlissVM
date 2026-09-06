use std::fmt;

#[derive(Debug, PartialEq)]
pub enum FsError {
    BadMagic,
    InvalidInodeType(u8),
    NameTooLong(String),
    FileTooLarge,
    OutOfFreeInodes,
    OutOfFreeBlocks,
    DirectoryFull,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FsError::BadMagic => write!(f, "Invalid magic number"),
            FsError::InvalidInodeType(it) => write!(f, "Invalid inode type: {}", it),
            FsError::NameTooLong(n) => write!(f, "Name '{}' is exceeds max length", n),
            FsError::FileTooLarge => write!(f, "File too large"),
            FsError::OutOfFreeInodes => write!(f, "Out of free inodes"),
            FsError::OutOfFreeBlocks => write!(f, "Out of free blocks"),
            FsError::DirectoryFull => write!(f, "Directory full"),
        }
    }
}

impl std::error::Error for FsError {}
