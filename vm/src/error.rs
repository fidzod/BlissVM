use crate::memory::MemoryError;

#[derive(Debug, PartialEq)]
pub enum VmError {
    Memory(MemoryError),
    UnknownOpcode(u8),
}

impl From<MemoryError> for VmError {
    fn from(value: MemoryError) -> Self {
        Self::Memory(value)
    }
}
