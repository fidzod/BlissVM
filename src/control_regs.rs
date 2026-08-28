#[repr(usize)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ControlReg {
    Tvec, Epc, Cause, Mode
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    User,
    Supervisor
}

impl TryFrom<u32> for ControlReg {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < 4 {
            Ok(unsafe { std::mem::transmute::<usize, Self>(value as usize) })
        } else {
            Err(())
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ControlRegs([u32; 4]);

impl Default for ControlRegs {
    fn default() -> Self {
        Self([0, 0, 0, 1])
    }
}

impl ControlRegs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, control_reg: ControlReg) -> u32 {
        self.0[control_reg as usize]
    }

    pub fn get_mut(&mut self, control_reg: ControlReg) -> &mut u32 {
        &mut self.0[control_reg as usize]
    }

    pub fn mode(&self) -> Mode {
        match self.get(ControlReg::Mode) {
            0 => Mode::User,
            1 => Mode::Supervisor,
            _ => unreachable!("Invalid mode")
        }
    }
}
