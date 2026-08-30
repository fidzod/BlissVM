#[derive(Debug, PartialEq)]
pub enum FsError {
    BadMagic,
    InvalidInodeType(u8),
}
