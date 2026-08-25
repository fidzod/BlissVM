#[repr(usize)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[rustfmt::skip]
pub enum Register {
    R0,  R1,  R2,  R3,  R4,  R5,  R6,  R7,
    R8,  R9, R10, R11, R12,  SP,  LR,  PC,
}

impl TryFrom<u32> for Register {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < 16 {
            Ok(unsafe { std::mem::transmute::<usize, Register>(value as usize) })
        } else {
            Err(())
        }
    }
}

#[derive(Default)]
pub struct Registers([u32; 16]);

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, register: Register) -> u32 {
        self.0[register as usize]
    }

    pub fn get_mut(&mut self, register: Register) -> &mut u32 {
        &mut self.0[register as usize]
    }
}
