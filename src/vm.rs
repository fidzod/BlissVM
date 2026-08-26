use crate::bus::Bus;
use crate::error::VmError;
use crate::instruction::Instruction;
use crate::register::{Register, Registers};

pub struct Vm {
    registers: Registers,
    bus: Bus,
}

pub enum StepResult {
    Continue,
    Halt,
}

impl Default for Vm {
    fn default() -> Self {
        Self {
            registers: Registers::new(),
            bus: Bus::new(),
        }
    }
}

impl Vm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, addr: u32, data: &[u8]) -> Result<(), VmError> {
        self.bus.load(addr, data)?;
        Ok(())
    }

    pub fn reg(&self, register: Register) -> u32 {
        self.registers.get(register)
    }

    pub fn fetch(&mut self) -> Result<u32, VmError> {
        let fetched = self.bus.get32(self.registers.get(Register::PC))?;
        *self.registers.get_mut(Register::PC) += 4;
        Ok(fetched)
    }

    fn branch_target(&self, offset: i32) -> u32 {
        (self.registers.get(Register::PC) as i32 - 4 + offset) as u32
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<StepResult, VmError> {
        match instruction {
            Instruction::Hlt => Ok(StepResult::Halt),
            Instruction::Nop => Ok(StepResult::Continue),
            Instruction::Mov { dst, src } => {
                *self.registers.get_mut(dst) = self.registers.get(src);
                Ok(StepResult::Continue)
            }
            Instruction::Ldi16 { dst, imm } => {
                *self.registers.get_mut(dst) = imm.into();
                Ok(StepResult::Continue)
            }
            Instruction::Ldi32l { dst, imm } => {
                let r = self.registers.get_mut(dst);
                *r &= 0xFFFF_0000;
                *r |= imm as u32;
                Ok(StepResult::Continue)
            }
            Instruction::Ldi32h { dst, imm } => {
                let r = self.registers.get_mut(dst);
                *r &= 0x0000_FFFF;
                *r |= (imm as u32) << 16;
                Ok(StepResult::Continue)
            }
            Instruction::Ldm8 { dst, base, offset } => {
                let adr = self.registers.get(base) as i32 + offset;
                *self.registers.get_mut(dst) = self.bus.get8(adr as u32)? as u32;
                Ok(StepResult::Continue)
            }
            Instruction::Ldm16 { dst, base, offset } => {
                let adr = self.registers.get(base) as i32 + offset;
                *self.registers.get_mut(dst) = self.bus.get16(adr as u32)? as u32;
                Ok(StepResult::Continue)
            }
            Instruction::Ldm32 { dst, base, offset } => {
                let adr = self.registers.get(base) as i32 + offset;
                *self.registers.get_mut(dst) = self.bus.get32(adr as u32)?;
                Ok(StepResult::Continue)
            }
            Instruction::Str8 { src, base, offset } => {
                let adr = self.registers.get(base) as i32 + offset;
                self.bus
                    .write8(adr as u32, self.registers.get(src) as u8)?;
                Ok(StepResult::Continue)
            }
            Instruction::Str16 { src, base, offset } => {
                let adr = self.registers.get(base) as i32 + offset;
                self.bus
                    .write16(adr as u32, self.registers.get(src) as u16)?;
                Ok(StepResult::Continue)
            }
            Instruction::Str32 { src, base, offset } => {
                let adr = self.registers.get(base) as i32 + offset;
                self.bus.write32(adr as u32, self.registers.get(src))?;
                Ok(StepResult::Continue)
            }
            Instruction::Add { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self
                    .registers
                    .get(src1)
                    .wrapping_add(self.registers.get(src2));
                Ok(StepResult::Continue)
            }
            Instruction::Sub { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self
                    .registers
                    .get(src1)
                    .wrapping_sub(self.registers.get(src2));
                Ok(StepResult::Continue)
            }
            Instruction::Mul { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self
                    .registers
                    .get(src1)
                    .wrapping_mul(self.registers.get(src2));
                Ok(StepResult::Continue)
            }
            Instruction::And { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self.registers.get(src1) & self.registers.get(src2);
                Ok(StepResult::Continue)
            }
            Instruction::Or { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self.registers.get(src1) | self.registers.get(src2);
                Ok(StepResult::Continue)
            }
            Instruction::Xor { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self.registers.get(src1) ^ self.registers.get(src2);
                Ok(StepResult::Continue)
            }
            Instruction::Not { dst, src } => {
                *self.registers.get_mut(dst) = !self.registers.get(src);
                Ok(StepResult::Continue)
            }
            Instruction::Sll { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self.registers.get(src1) << self.registers.get(src2);
                Ok(StepResult::Continue)
            }
            Instruction::Slr { dst, src1, src2 } => {
                *self.registers.get_mut(dst) = self.registers.get(src1) >> self.registers.get(src2);
                Ok(StepResult::Continue)
            }
            Instruction::Sar { dst, src1, src2 } => {
                *self.registers.get_mut(dst) =
                    ((self.registers.get(src1) as i32) >> self.registers.get(src2)) as u32;
                Ok(StepResult::Continue)
            }
            Instruction::Beq { lhs, rhs, offset } => {
                if self.registers.get(lhs) == self.registers.get(rhs) {
                    *self.registers.get_mut(Register::PC) = self.branch_target(offset);
                }
                Ok(StepResult::Continue)
            }
            Instruction::Bne { lhs, rhs, offset } => {
                if self.registers.get(lhs) != self.registers.get(rhs) {
                    *self.registers.get_mut(Register::PC) = self.branch_target(offset);
                }
                Ok(StepResult::Continue)
            }
            Instruction::Blt { lhs, rhs, offset } => {
                if (self.registers.get(lhs) as i32) < self.registers.get(rhs) as i32 {
                    *self.registers.get_mut(Register::PC) = self.branch_target(offset);
                }
                Ok(StepResult::Continue)
            }
            Instruction::Bge { lhs, rhs, offset } => {
                if (self.registers.get(lhs) as i32) >= self.registers.get(rhs) as i32 {
                    *self.registers.get_mut(Register::PC) = self.branch_target(offset);
                }
                Ok(StepResult::Continue)
            }
            Instruction::Bltu { lhs, rhs, offset } => {
                if self.registers.get(lhs) < self.registers.get(rhs) {
                    *self.registers.get_mut(Register::PC) = self.branch_target(offset);
                }
                Ok(StepResult::Continue)
            }
            Instruction::Bgeu { lhs, rhs, offset } => {
                if self.registers.get(lhs) >= self.registers.get(rhs) {
                    *self.registers.get_mut(Register::PC) = self.branch_target(offset);
                }
                Ok(StepResult::Continue)
            }
            Instruction::Bal { link, offset } => {
                *self.registers.get_mut(link) = self.registers.get(Register::PC);
                *self.registers.get_mut(Register::PC) = self.branch_target(offset);
                Ok(StepResult::Continue)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg(vm: &Vm, r: Register) -> u32 {
        vm.registers.get(r)
    }

    fn set_reg(vm: &mut Vm, r: Register, val: u32) {
        *vm.registers.get_mut(r) = val;
    }

    #[test]
    fn fetch_reads_big_endian_and_advances_pc() {
        let mut vm = Vm::new();
        vm.bus
            .load(0, &[0x13, 0x37, 0xC0, 0xDE, 0x00, 0x00, 0x00, 0x00])
            .unwrap();
        assert_eq!(vm.fetch().unwrap(), 0x1337_C0DE);
        assert_eq!(reg(&vm, Register::PC), 4);
        assert_eq!(vm.fetch().unwrap(), 0x0000_0000);
        assert_eq!(reg(&vm, Register::PC), 8);
    }

    #[test]
    fn execute_mov() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 42);
        vm.execute(Instruction::Mov {
            dst: Register::R1,
            src: Register::R0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R1), 42);
    }

    #[test]
    fn execute_ldi16_overwrites_full_register() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xDEAD_0000);
        vm.execute(Instruction::Ldi16 {
            dst: Register::R0,
            imm: 0x1234,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R0), 0x0000_1234);
    }

    #[test]
    fn execute_ldi32l_preserves_high_half() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xDEAD_BEEF);
        vm.execute(Instruction::Ldi32l {
            dst: Register::R0,
            imm: 0x1234,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R0), 0xDEAD_1234);
    }

    #[test]
    fn execute_ldi32h_preserves_low_half() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xDEAD_BEEF);
        vm.execute(Instruction::Ldi32h {
            dst: Register::R0,
            imm: 0x1234,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R0), 0x1234_BEEF);
    }

    #[test]
    fn execute_ldm8_zero_extends() {
        let mut vm = Vm::new();
        vm.bus.load(0x100, &[0xFF]).unwrap();
        set_reg(&mut vm, Register::R1, 0x100);
        vm.execute(Instruction::Ldm8 {
            dst: Register::R0,
            base: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R0), 0x0000_00FF);
    }

    #[test]
    fn execute_ldm16_zero_extends() {
        let mut vm = Vm::new();
        vm.bus.load(0x100, &[0xBE, 0xEF]).unwrap();
        set_reg(&mut vm, Register::R1, 0x100);
        vm.execute(Instruction::Ldm16 {
            dst: Register::R0,
            base: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R0), 0x0000_BEEF);
    }

    #[test]
    fn execute_ldm32() {
        let mut vm = Vm::new();
        vm.bus.load(0x100, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        set_reg(&mut vm, Register::R1, 0x100);
        vm.execute(Instruction::Ldm32 {
            dst: Register::R0,
            base: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R0), 0xDEAD_BEEF);
    }

    #[test]
    fn execute_load_with_negative_offset() {
        let mut vm = Vm::new();
        vm.bus.load(0x100, &[0xAB]).unwrap();
        set_reg(&mut vm, Register::R1, 0x105);
        vm.execute(Instruction::Ldm8 {
            dst: Register::R0,
            base: Register::R1,
            offset: -5,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R0), 0xAB);
    }

    #[test]
    fn execute_str8_truncates_to_byte() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xDEAD_BEEF);
        set_reg(&mut vm, Register::R1, 0x100);
        vm.execute(Instruction::Str8 {
            src: Register::R0,
            base: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(vm.bus.get8(0x100).unwrap(), 0xEF);
    }

    #[test]
    fn execute_str16_truncates_to_halfword() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xDEAD_BEEF);
        set_reg(&mut vm, Register::R1, 0x100);
        vm.execute(Instruction::Str16 {
            src: Register::R0,
            base: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(vm.bus.get16(0x100).unwrap(), 0xBEEF);
    }

    #[test]
    fn execute_str32() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xDEAD_BEEF);
        set_reg(&mut vm, Register::R1, 0x100);
        vm.execute(Instruction::Str32 {
            src: Register::R0,
            base: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(vm.bus.get32(0x100).unwrap(), 0xDEAD_BEEF);
    }

    #[test]
    fn execute_store_with_positive_offset() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xFF);
        set_reg(&mut vm, Register::R1, 0x100);
        vm.execute(Instruction::Str8 {
            src: Register::R0,
            base: Register::R1,
            offset: 4,
        })
        .unwrap();
        assert_eq!(vm.bus.get8(0x104).unwrap(), 0xFF);
    }

    #[test]
    fn execute_add() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 35);
        set_reg(&mut vm, Register::R1, 7);
        vm.execute(Instruction::Add {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 42);
    }

    #[test]
    fn execute_add_wraps_on_overflow() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, u32::MAX);
        set_reg(&mut vm, Register::R1, 1);
        vm.execute(Instruction::Add {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 0);
    }

    #[test]
    fn execute_sub() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 50);
        set_reg(&mut vm, Register::R1, 8);
        vm.execute(Instruction::Sub {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 42);
    }

    #[test]
    fn execute_sub_wraps_on_underflow() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0);
        set_reg(&mut vm, Register::R1, 1);
        vm.execute(Instruction::Sub {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), u32::MAX);
    }

    #[test]
    fn execute_mul() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 6);
        set_reg(&mut vm, Register::R1, 7);
        vm.execute(Instruction::Mul {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 42);
    }

    #[test]
    fn execute_mul_truncates_to_32_bits() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0x0001_0000);
        set_reg(&mut vm, Register::R1, 0x0001_0000);
        vm.execute(Instruction::Mul {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 0); // 2^32 truncated
    }

    #[test]
    fn execute_and() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xFF00_FF00);
        set_reg(&mut vm, Register::R1, 0x0FF0_0FF0);
        vm.execute(Instruction::And {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 0x0F00_0F00);
    }

    #[test]
    fn execute_or() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xFF00_0000);
        set_reg(&mut vm, Register::R1, 0x0000_00FF);
        vm.execute(Instruction::Or {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 0xFF00_00FF);
    }

    #[test]
    fn execute_xor_with_self_gives_zero() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xDEAD_BEEF);
        vm.execute(Instruction::Xor {
            dst: Register::R1,
            src1: Register::R0,
            src2: Register::R0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R1), 0);
    }

    #[test]
    fn execute_not() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0xAAAA_5555);
        vm.execute(Instruction::Not {
            dst: Register::R1,
            src: Register::R0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R1), 0x5555_AAAA);
    }

    #[test]
    fn execute_sll() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 1);
        set_reg(&mut vm, Register::R1, 8);
        vm.execute(Instruction::Sll {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 256);
    }

    #[test]
    fn execute_slr_does_not_sign_extend() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0x8000_0000);
        set_reg(&mut vm, Register::R1, 1);
        vm.execute(Instruction::Slr {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 0x4000_0000);
    }

    #[test]
    fn execute_sar_sign_extends_negative() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0x8000_0000); // -2147483648 as i32
        set_reg(&mut vm, Register::R1, 1);
        vm.execute(Instruction::Sar {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 0xC000_0000);
    }

    #[test]
    fn execute_sar_does_not_sign_extend_positive() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::R0, 0x7FFF_FFFE);
        set_reg(&mut vm, Register::R1, 1);
        vm.execute(Instruction::Sar {
            dst: Register::R2,
            src1: Register::R0,
            src2: Register::R1,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::R2), 0x3FFF_FFFF);
    }

    // In normal operation fetch increments PC by 4 before execute runs.
    // Branch target = PC_after_fetch - 4 + offset.
    // Tests below set PC to 4 (simulating fetch of instruction at address 0),
    // so the effective branch target equals `offset`.

    #[test]
    fn execute_beq_taken() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 7);
        set_reg(&mut vm, Register::R1, 7);
        vm.execute(Instruction::Beq {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 16,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 16);
    }

    #[test]
    fn execute_beq_not_taken() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 1);
        set_reg(&mut vm, Register::R1, 2);
        vm.execute(Instruction::Beq {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 16,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 4); // unchanged
    }

    #[test]
    fn execute_bne_taken() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 1);
        set_reg(&mut vm, Register::R1, 2);
        vm.execute(Instruction::Bne {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 0);
    }

    #[test]
    fn execute_bne_not_taken() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 5);
        set_reg(&mut vm, Register::R1, 5);
        vm.execute(Instruction::Bne {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 0,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 4);
    }

    #[test]
    fn execute_blt_signed_taken() {
        // 0xFFFF_FFFF is -1 as i32; -1 < 0 is true
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 0xFFFF_FFFF);
        set_reg(&mut vm, Register::R1, 0);
        vm.execute(Instruction::Blt {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 8);
    }

    #[test]
    fn execute_blt_not_taken_when_equal() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 5);
        set_reg(&mut vm, Register::R1, 5);
        vm.execute(Instruction::Blt {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 4);
    }

    #[test]
    fn execute_bge_taken_when_greater() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 1);
        set_reg(&mut vm, Register::R1, 0);
        vm.execute(Instruction::Bge {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 8);
    }

    #[test]
    fn execute_bge_taken_when_equal() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 3);
        set_reg(&mut vm, Register::R1, 3);
        vm.execute(Instruction::Bge {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 8);
    }

    #[test]
    fn execute_bltu_not_taken_for_large_unsigned_value() {
        // 0xFFFF_FFFF > 0 unsigned → NOT taken; as signed -1 < 0 → would be taken
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 0xFFFF_FFFF);
        set_reg(&mut vm, Register::R1, 0);
        vm.execute(Instruction::Bltu {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 4);
    }

    #[test]
    fn execute_bltu_taken() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 0);
        set_reg(&mut vm, Register::R1, 1);
        vm.execute(Instruction::Bltu {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 8);
    }

    #[test]
    fn execute_bgeu_taken_for_large_unsigned_value() {
        // 0xFFFF_FFFF >= 0 unsigned → taken
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 0xFFFF_FFFF);
        set_reg(&mut vm, Register::R1, 0);
        vm.execute(Instruction::Bgeu {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 8);
    }

    #[test]
    fn execute_bgeu_not_taken() {
        let mut vm = Vm::new();
        set_reg(&mut vm, Register::PC, 4);
        set_reg(&mut vm, Register::R0, 0);
        set_reg(&mut vm, Register::R1, 1);
        vm.execute(Instruction::Bgeu {
            lhs: Register::R0,
            rhs: Register::R1,
            offset: 8,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::PC), 4);
    }

    #[test]
    fn execute_bal_saves_return_address_and_jumps() {
        let mut vm = Vm::new();
        // PC=8: simulates fetch having advanced from instruction at addr 4
        set_reg(&mut vm, Register::PC, 8);
        vm.execute(Instruction::Bal {
            link: Register::LR,
            offset: 16,
        })
        .unwrap();
        assert_eq!(reg(&vm, Register::LR), 8); // saved before jump
        assert_eq!(reg(&vm, Register::PC), 20); // 8 - 4 + 16
    }
}
