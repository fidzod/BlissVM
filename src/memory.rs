const MEMORY_SIZE: usize = 1024 * 4;

pub struct Memory([u8; MEMORY_SIZE]);

#[derive(Debug, PartialEq)]
pub enum MemoryError {
    OutOfBounds,
}

impl Default for Memory {
    fn default() -> Self {
        Self([0; MEMORY_SIZE])
    }
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get8(&self, addr: u32) -> Result<u8, MemoryError> {
        self.0
            .get(addr as usize)
            .copied()
            .ok_or(MemoryError::OutOfBounds)
    }

    pub fn get16(&self, addr: u32) -> Result<u16, MemoryError> {
        self.0
            .get(addr as usize..addr as usize + 2)
            .and_then(|slice| slice.try_into().ok())
            .map(u16::from_be_bytes)
            .ok_or(MemoryError::OutOfBounds)
    }

    pub fn get32(&self, addr: u32) -> Result<u32, MemoryError> {
        self.0
            .get(addr as usize..addr as usize + 4)
            .and_then(|slice| slice.try_into().ok())
            .map(u32::from_be_bytes)
            .ok_or(MemoryError::OutOfBounds)
    }

    pub fn write8(&mut self, addr: u32, data: u8) -> Result<(), MemoryError> {
        self.0
            .get_mut(addr as usize)
            .ok_or(MemoryError::OutOfBounds)
            .map(|b| *b = data)
    }

    pub fn write16(&mut self, addr: u32, data: u16) -> Result<(), MemoryError> {
        self.0
            .get_mut(addr as usize..addr as usize + 2)
            .ok_or(MemoryError::OutOfBounds)?
            .copy_from_slice(&u16::to_be_bytes(data));
        Ok(())
    }

    pub fn write32(&mut self, addr: u32, data: u32) -> Result<(), MemoryError> {
        self.0
            .get_mut(addr as usize..addr as usize + 4)
            .ok_or(MemoryError::OutOfBounds)?
            .copy_from_slice(&u32::to_be_bytes(data));
        Ok(())
    }

    pub fn load(&mut self, addr: u32, data: &[u8]) -> Result<(), MemoryError> {
        self.0
            .get_mut(addr as usize..addr as usize + data.len())
            .ok_or(MemoryError::OutOfBounds)?
            .copy_from_slice(data);
        Ok(())
    }
}
