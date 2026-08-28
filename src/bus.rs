use std::io::Write;

use crate::{
    memory::{Memory, MemoryError},
    storage::Storage,
};

pub struct Bus {
    memory: Memory,
    storage: Option<Storage>,
    disk_sector: u32,
    disk_buffer: u32,
}

const MMIO_BASE: u32 = 0xFFFF_0000;
const TX: u32 = 0xFFFF_0000;
const DISK_SECTOR: u32 = 0xFFFF_0010;
const DISK_BUFFER: u32 = 0xFFFF_0014;
const DISK_COMMAND: u32 = 0xFFFF_0018;

impl Default for Bus {
    fn default() -> Self {
        Self {
            memory: Memory::new(),
            storage: None,
            disk_sector: 0,
            disk_buffer: 0,
        }
    }
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get8(&self, addr: u32) -> Result<u8, MemoryError> {
        if addr < MMIO_BASE {
            self.memory.get8(addr)
        } else {
            Ok(0)
        }
    }

    pub fn get16(&self, addr: u32) -> Result<u16, MemoryError> {
        if addr < MMIO_BASE {
            self.memory.get16(addr)
        } else {
            Ok(0)
        }
    }

    pub fn get32(&self, addr: u32) -> Result<u32, MemoryError> {
        if addr < MMIO_BASE {
            self.memory.get32(addr)
        } else {
            Ok(0)
        }
    }

    pub fn write8(&mut self, addr: u32, data: u8) -> Result<(), MemoryError> {
        if addr < MMIO_BASE {
            self.memory.write8(addr, data)
        } else if addr == TX {
            std::io::stdout()
                .write_all(&[data])
                .expect("Could not write to stdout");
            std::io::stdout().flush().expect("Could not flush stdout");
            Ok(())
        } else if addr == DISK_SECTOR {
            self.disk_sector = data as u32;
            Ok(())
        } else if addr == DISK_BUFFER {
            self.disk_buffer = data as u32;
            Ok(())
        } else if addr == DISK_COMMAND {
            if data == 0 && let Some(ref storage) = self.storage {
                self.memory.load(self.disk_buffer, &storage.read_sector(self.disk_sector))?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn write16(&mut self, addr: u32, data: u16) -> Result<(), MemoryError> {
        if addr < MMIO_BASE {
            self.memory.write16(addr, data)
        } else if addr == TX {
            std::io::stdout()
                .write_all(&[data as u8])
                .expect("Could not write to stdout");
            std::io::stdout().flush().expect("Could not flush stdout");
            Ok(())
        } else if addr == DISK_SECTOR {
            self.disk_sector = data as u32;
            Ok(())
        } else if addr == DISK_BUFFER {
            self.disk_buffer = data as u32;
            Ok(())
        } else if addr == DISK_COMMAND {
            if data == 0 && let Some(ref storage) = self.storage {
                self.memory.load(self.disk_buffer, &storage.read_sector(self.disk_sector))?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn write32(&mut self, addr: u32, data: u32) -> Result<(), MemoryError> {
        if addr < MMIO_BASE {
            self.memory.write32(addr, data)
        } else if addr == TX {
            std::io::stdout()
                .write_all(&[data as u8])
                .expect("Could not write to stdout");
            std::io::stdout().flush().expect("Could not flush stdout");
            Ok(())
        } else if addr == DISK_SECTOR {
            self.disk_sector = data;
            Ok(())
        } else if addr == DISK_BUFFER {
            self.disk_buffer = data;
            Ok(())
        } else if addr == DISK_COMMAND {
            if data == 0 && let Some(ref storage) = self.storage {
                self.load(self.disk_buffer, &storage.read_sector(self.disk_sector))?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn load(&mut self, addr: u32, data: &[u8]) -> Result<(), MemoryError> {
        if addr < MMIO_BASE {
            self.memory.load(addr, data)
        } else {
            Ok(())
        }
    }

    pub fn attach_storage(&mut self, data: Vec<u8>) {
        self.storage = Some(Storage::from_data(data));
    }
}
