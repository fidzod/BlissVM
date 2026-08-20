use crate::error::VmError;
use crate::register::Register;

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Hlt,
    Nop,
    Mov {
        dst: Register,
        src: Register,
    },
    Ldi16 {
        dst: Register,
        imm: u16,
    },
    Ldi32l {
        dst: Register,
        imm: u16,
    },
    Ldi32h {
        dst: Register,
        imm: u16,
    },
    Ldm8 {
        dst: Register,
        base: Register,
        offset: i32,
    },
    Ldm16 {
        dst: Register,
        base: Register,
        offset: i32,
    },
    Ldm32 {
        dst: Register,
        base: Register,
        offset: i32,
    },
    Str8 {
        src: Register,
        base: Register,
        offset: i32,
    },
    Str16 {
        src: Register,
        base: Register,
        offset: i32,
    },
    Str32 {
        src: Register,
        base: Register,
        offset: i32,
    },
    Add {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Sub {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Mul {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    And {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Or {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Xor {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Not {
        dst: Register,
        src: Register,
    },
    Sll {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Slr {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Sar {
        dst: Register,
        src1: Register,
        src2: Register,
    },
    Beq {
        lhs: Register,
        rhs: Register,
        offset: i32,
    },
    Bne {
        lhs: Register,
        rhs: Register,
        offset: i32,
    },
    Blt {
        lhs: Register,
        rhs: Register,
        offset: i32,
    },
    Bge {
        lhs: Register,
        rhs: Register,
        offset: i32,
    },
    Bltu {
        lhs: Register,
        rhs: Register,
        offset: i32,
    },
    Bgeu {
        lhs: Register,
        rhs: Register,
        offset: i32,
    },
    Bal {
        link: Register,
        offset: i32,
    },
}

impl Instruction {
    pub fn decode(i: u32) -> Result<Instruction, VmError> {
        let opcode = i >> (32 - 6);

        fn reg(i: u32, pos: u32) -> Register {
            Register::try_from((i >> pos) & 0xF).expect("register field masked to 4 bits")
        }

        fn i18(i: u32) -> i32 {
            ((i << 14) as i32) >> 14
        }

        fn rd(i: u32) -> Register {
            reg(i, 22)
        }

        fn rs1(i: u32) -> Register {
            reg(i, 18)
        }

        fn rs2(i: u32) -> Register {
            reg(i, 14)
        }

        match opcode {
            0x00 => Ok(Instruction::Hlt),
            0x01 => Ok(Instruction::Nop),

            0x02 => Ok(Instruction::Mov {
                dst: rd(i),
                src: rs1(i),
            }),

            0x03 => Ok(Instruction::Ldi16 {
                dst: rd(i),
                imm: (i & 0xffff) as u16,
            }),
            0x04 => Ok(Instruction::Ldi32l {
                dst: rd(i),
                imm: (i & 0xffff) as u16,
            }),
            0x05 => Ok(Instruction::Ldi32h {
                dst: rd(i),
                imm: (i & 0xffff) as u16,
            }),

            0x06 => Ok(Instruction::Ldm8 {
                dst: rd(i),
                base: rs1(i),
                offset: i18(i),
            }),
            0x07 => Ok(Instruction::Ldm16 {
                dst: rd(i),
                base: rs1(i),
                offset: i18(i),
            }),
            0x08 => Ok(Instruction::Ldm32 {
                dst: rd(i),
                base: rs1(i),
                offset: i18(i),
            }),
            0x09 => Ok(Instruction::Str8 {
                src: rd(i),
                base: rs1(i),
                offset: i18(i),
            }),
            0x0A => Ok(Instruction::Str16 {
                src: rd(i),
                base: rs1(i),
                offset: i18(i),
            }),
            0x0B => Ok(Instruction::Str32 {
                src: rd(i),
                base: rs1(i),
                offset: i18(i),
            }),

            0x0C => Ok(Instruction::Add {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x0D => Ok(Instruction::Sub {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x0E => Ok(Instruction::Mul {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x0F => Ok(Instruction::And {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x10 => Ok(Instruction::Or {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x11 => Ok(Instruction::Xor {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x12 => Ok(Instruction::Not {
                dst: rd(i),
                src: rs1(i),
            }),
            0x13 => Ok(Instruction::Sll {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x14 => Ok(Instruction::Slr {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),
            0x15 => Ok(Instruction::Sar {
                dst: rd(i),
                src1: rs1(i),
                src2: rs2(i),
            }),

            0x16 => Ok(Instruction::Beq {
                lhs: rd(i),
                rhs: rs1(i),
                offset: i18(i),
            }),
            0x17 => Ok(Instruction::Bne {
                lhs: rd(i),
                rhs: rs1(i),
                offset: i18(i),
            }),
            0x18 => Ok(Instruction::Blt {
                lhs: rd(i),
                rhs: rs1(i),
                offset: i18(i),
            }),
            0x19 => Ok(Instruction::Bge {
                lhs: rd(i),
                rhs: rs1(i),
                offset: i18(i),
            }),
            0x1A => Ok(Instruction::Bltu {
                lhs: rd(i),
                rhs: rs1(i),
                offset: i18(i),
            }),
            0x1B => Ok(Instruction::Bgeu {
                lhs: rd(i),
                rhs: rs1(i),
                offset: i18(i),
            }),

            0x1C => Ok(Instruction::Bal {
                link: rd(i),
                offset: ((i << 10) as i32) >> 10,
            }),

            _ => Err(VmError::UnknownOpcode(opcode as u8)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Instruction word encoders matching the decode field layout.
    fn enc_rr(op: u32, a: u32, b: u32) -> u32 {
        (op << 26) | (a << 22) | (b << 18)
    }
    fn enc_rrr(op: u32, a: u32, b: u32, c: u32) -> u32 {
        (op << 26) | (a << 22) | (b << 18) | (c << 14)
    }
    fn enc_ri(op: u32, dst: u32, imm: u16) -> u32 {
        (op << 26) | (dst << 22) | (imm as u32)
    }
    fn enc_rri(op: u32, a: u32, b: u32, offset: i32) -> u32 {
        (op << 26) | (a << 22) | (b << 18) | ((offset as u32) & 0x3_FFFF)
    }
    fn enc_bal(link: u32, offset: i32) -> u32 {
        (0x1C_u32 << 26) | (link << 22) | ((offset as u32) & 0x3F_FFFF)
    }

    const HLT: u32 = 0;

    #[test]
    fn decode_hlt() {
        assert_eq!(Instruction::decode(HLT), Ok(Instruction::Hlt));
    }

    #[test]
    fn decode_nop() {
        assert_eq!(Instruction::decode(0x01 << 26), Ok(Instruction::Nop));
    }

    #[test]
    fn decode_unknown_opcode() {
        assert_eq!(
            Instruction::decode(0x3F << 26),
            Err(VmError::UnknownOpcode(0x3F))
        );
    }

    #[test]
    fn decode_mov() {
        assert_eq!(
            Instruction::decode(enc_rr(0x02, 3, 5)),
            Ok(Instruction::Mov {
                dst: Register::R3,
                src: Register::R5
            })
        );
    }

    #[test]
    fn decode_ldi16() {
        assert_eq!(
            Instruction::decode(enc_ri(0x03, 1, 0xBEEF)),
            Ok(Instruction::Ldi16 {
                dst: Register::R1,
                imm: 0xBEEF
            })
        );
    }

    #[test]
    fn decode_ldi32l() {
        assert_eq!(
            Instruction::decode(enc_ri(0x04, 2, 0x1234)),
            Ok(Instruction::Ldi32l {
                dst: Register::R2,
                imm: 0x1234
            })
        );
    }

    #[test]
    fn decode_ldi32h() {
        assert_eq!(
            Instruction::decode(enc_ri(0x05, 0, 0xABCD)),
            Ok(Instruction::Ldi32h {
                dst: Register::R0,
                imm: 0xABCD
            })
        );
    }

    #[test]
    fn decode_ldm8() {
        assert_eq!(
            Instruction::decode(enc_rri(0x06, 1, 2, 7)),
            Ok(Instruction::Ldm8 {
                dst: Register::R1,
                base: Register::R2,
                offset: 7
            })
        );
    }

    #[test]
    fn decode_ldm16() {
        assert_eq!(
            Instruction::decode(enc_rri(0x07, 0, 3, -4)),
            Ok(Instruction::Ldm16 {
                dst: Register::R0,
                base: Register::R3,
                offset: -4
            })
        );
    }

    #[test]
    fn decode_ldm32() {
        assert_eq!(
            Instruction::decode(enc_rri(0x08, 4, 5, 0)),
            Ok(Instruction::Ldm32 {
                dst: Register::R4,
                base: Register::R5,
                offset: 0
            })
        );
    }

    #[test]
    fn decode_str8() {
        assert_eq!(
            Instruction::decode(enc_rri(0x09, 0, 1, 3)),
            Ok(Instruction::Str8 {
                src: Register::R0,
                base: Register::R1,
                offset: 3
            })
        );
    }

    #[test]
    fn decode_str16() {
        assert_eq!(
            Instruction::decode(enc_rri(0x0A, 2, 3, -1)),
            Ok(Instruction::Str16 {
                src: Register::R2,
                base: Register::R3,
                offset: -1
            })
        );
    }

    #[test]
    fn decode_str32() {
        assert_eq!(
            Instruction::decode(enc_rri(0x0B, 1, 0, 100)),
            Ok(Instruction::Str32 {
                src: Register::R1,
                base: Register::R0,
                offset: 100
            })
        );
    }

    #[test]
    fn decode_add() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x0C, 2, 0, 1)),
            Ok(Instruction::Add {
                dst: Register::R2,
                src1: Register::R0,
                src2: Register::R1
            })
        );
    }

    #[test]
    fn decode_sub() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x0D, 0, 1, 2)),
            Ok(Instruction::Sub {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_mul() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x0E, 0, 1, 2)),
            Ok(Instruction::Mul {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_and() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x0F, 0, 1, 2)),
            Ok(Instruction::And {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_or() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x10, 0, 1, 2)),
            Ok(Instruction::Or {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_xor() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x11, 0, 1, 2)),
            Ok(Instruction::Xor {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_not() {
        assert_eq!(
            Instruction::decode(enc_rr(0x12, 1, 0)),
            Ok(Instruction::Not {
                dst: Register::R1,
                src: Register::R0
            })
        );
    }

    #[test]
    fn decode_sll() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x13, 0, 1, 2)),
            Ok(Instruction::Sll {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_slr() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x14, 0, 1, 2)),
            Ok(Instruction::Slr {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_sar() {
        assert_eq!(
            Instruction::decode(enc_rrr(0x15, 0, 1, 2)),
            Ok(Instruction::Sar {
                dst: Register::R0,
                src1: Register::R1,
                src2: Register::R2
            })
        );
    }

    #[test]
    fn decode_beq() {
        assert_eq!(
            Instruction::decode(enc_rri(0x16, 0, 1, 8)),
            Ok(Instruction::Beq {
                lhs: Register::R0,
                rhs: Register::R1,
                offset: 8
            })
        );
    }

    #[test]
    fn decode_bne() {
        assert_eq!(
            Instruction::decode(enc_rri(0x17, 2, 3, -12)),
            Ok(Instruction::Bne {
                lhs: Register::R2,
                rhs: Register::R3,
                offset: -12
            })
        );
    }

    #[test]
    fn decode_blt() {
        assert_eq!(
            Instruction::decode(enc_rri(0x18, 0, 1, 4)),
            Ok(Instruction::Blt {
                lhs: Register::R0,
                rhs: Register::R1,
                offset: 4
            })
        );
    }

    #[test]
    fn decode_bge() {
        assert_eq!(
            Instruction::decode(enc_rri(0x19, 0, 1, 4)),
            Ok(Instruction::Bge {
                lhs: Register::R0,
                rhs: Register::R1,
                offset: 4
            })
        );
    }

    #[test]
    fn decode_bltu() {
        assert_eq!(
            Instruction::decode(enc_rri(0x1A, 0, 1, 4)),
            Ok(Instruction::Bltu {
                lhs: Register::R0,
                rhs: Register::R1,
                offset: 4
            })
        );
    }

    #[test]
    fn decode_bgeu() {
        assert_eq!(
            Instruction::decode(enc_rri(0x1B, 0, 1, 4)),
            Ok(Instruction::Bgeu {
                lhs: Register::R0,
                rhs: Register::R1,
                offset: 4
            })
        );
    }

    #[test]
    fn decode_branch_negative_offset() {
        // Verifies sign-extension of the 18-bit offset field
        assert_eq!(
            Instruction::decode(enc_rri(0x16, 4, 5, -420)),
            Ok(Instruction::Beq {
                lhs: Register::R4,
                rhs: Register::R5,
                offset: -420
            })
        );
    }

    #[test]
    fn decode_bal() {
        assert_eq!(
            Instruction::decode(enc_bal(14, 420420)),
            Ok(Instruction::Bal {
                link: Register::LR,
                offset: 420420
            })
        );
    }

    #[test]
    fn decode_bal_negative_offset() {
        assert_eq!(
            Instruction::decode(enc_bal(14, -8)),
            Ok(Instruction::Bal {
                link: Register::LR,
                offset: -8
            })
        );
    }
}
