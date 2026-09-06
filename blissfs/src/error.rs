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
