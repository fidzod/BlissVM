use std::io::{Read, Write};

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
const RX: u32 = 0xFFFF_0004;
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

    fn mmio_read(&mut self, addr: u32) -> u8 {
        if addr == RX {
            let mut buf = [0u8; 1];
            std::io::stdin()
                .lock()
                .read_exact(&mut buf)
                .expect("Could not read from stdin");
            buf[0]
        } else {
            0
        }
    }

    fn mmio_write(&mut self, addr: u32, val: u32) -> Result<(), MemoryError> {
        if addr == TX {
            std::io::stdout()
                .write_all(&[val as u8])
                .expect("Could not write to stdout");
            std::io::stdout().flush().expect("Could not flush stdout");
            Ok(())
        } else if addr == DISK_SECTOR {
            self.disk_sector = val;
            Ok(())
        } else if addr == DISK_BUFFER {
            self.disk_buffer = val;
            Ok(())
        } else if addr == DISK_COMMAND {
            if val == 0
                && let Some(ref storage) = self.storage
            {
                self.memory
                    .load(self.disk_buffer, &storage.read_sector(self.disk_sector))?;
            }
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn get8(&mut self, addr: u32) -> Result<u8, MemoryError> {
        if addr < MMIO_BASE {
            self.memory.get8(addr)
        } else {
            Ok(self.mmio_read(addr))
        }
    }

    pub fn get16(&mut self, addr: u32) -> Result<u16, MemoryError> {
        if addr < MMIO_BASE {
            self.memory.get16(addr)
        } else {
            Ok(self.mmio_read(addr) as u16)
        }
    }

    pub fn get32(&mut self, addr: u32) -> Result<u32, MemoryError> {
        if addr < MMIO_BASE {
            self.memory.get32(addr)
        } else {
            Ok(self.mmio_read(addr) as u32)
        }
    }

    pub fn write8(&mut self, addr: u32, data: u8) -> Result<(), MemoryError> {
        if addr < MMIO_BASE {
            self.memory.write8(addr, data)
        } else {
            self.mmio_write(addr, data as u32)
        }
    }

    pub fn write16(&mut self, addr: u32, data: u16) -> Result<(), MemoryError> {
        if addr < MMIO_BASE {
            self.memory.write16(addr, data)
        } else {
            self.mmio_write(addr, data as u32)
        }
    }

    pub fn write32(&mut self, addr: u32, data: u32) -> Result<(), MemoryError> {
        if addr < MMIO_BASE {
            self.memory.write32(addr, data)
        } else {
            self.mmio_write(addr, data as u32)
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
