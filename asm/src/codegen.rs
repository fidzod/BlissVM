use std::collections::HashMap;

use bliss::{control_regs::ControlReg, register::Register};

use crate::parser::{Item, Operand};

#[derive(Debug)]
pub enum CodegenError {
    UnknownMnemonic(String),
    UnknownDirective(String),
    WrongNumberOfOperands {
        mnemonic: String,
        expected: usize,
        got: usize,
    },
    WrongOperandType {
        mnemonic: String,
        position: usize,
        expected: &'static str,
    },
    UndefinedLabel(String),
    ImmediateOutOfRange {
        value: i64,
        bits: u32,
    },
}

fn build_symbol_table(items: &[Item]) -> Result<HashMap<String, u32>, CodegenError> {
    let mut current_address = 0;
    let mut symbol_table: HashMap<String, u32> = HashMap::new();

    for item in items {
        match item {
            Item::Label(name) => {
                symbol_table.insert(name.clone(), current_address);
            }
            Item::Instruction { mnemonic, .. } => {
                current_address += match mnemonic.as_str() {
                    "li" => 8,
                    _ => 4,
                }
            }
            Item::Directive { name, values } => match name.as_str() {
                "word" => current_address += 4 * values.len() as u32,
                "byte" | "str" => current_address += values.len() as u32,
                _ => (),
            },
        }
    }

    Ok(symbol_table)
}

fn check_operand_count(
    mnemonic: &str,
    operands: &[Operand],
    expected: usize,
) -> Result<(), CodegenError> {
    if operands.len() != expected {
        Err(CodegenError::WrongNumberOfOperands {
            mnemonic: mnemonic.to_string(),
            expected,
            got: operands.len(),
        })
    } else {
        Ok(())
    }
}

fn expect_reg(operands: &[Operand], pos: usize, mnemonic: &str) -> Result<Register, CodegenError> {
    match &operands[pos] {
        Operand::Reg(r) => Ok(*r),
        _ => Err(CodegenError::WrongOperandType {
            mnemonic: mnemonic.to_string(),
            position: pos,
            expected: "register",
        }),
    }
}

fn expect_ctrl_reg(
    operands: &[Operand],
    pos: usize,
    mnemonic: &str,
) -> Result<ControlReg, CodegenError> {
    match &operands[pos] {
        Operand::ControlReg(r) => Ok(*r),
        _ => Err(CodegenError::WrongOperandType {
            mnemonic: mnemonic.to_string(),
            position: pos,
            expected: "control register",
        }),
    }
}

fn expect_imm(operands: &[Operand], pos: usize, mnemonic: &str) -> Result<i64, CodegenError> {
    match &operands[pos] {
        Operand::Imm(i) => Ok(*i),
        _ => Err(CodegenError::WrongOperandType {
            mnemonic: mnemonic.to_string(),
            position: pos,
            expected: "immediate",
        }),
    }
}

fn expect_memref(
    operands: &[Operand],
    pos: usize,
    mnemonic: &str,
) -> Result<(Register, i32), CodegenError> {
    match &operands[pos] {
        Operand::MemRef { base, offset } => Ok((*base, *offset)),
        _ => Err(CodegenError::WrongOperandType {
            mnemonic: mnemonic.to_string(),
            position: pos,
            expected: "memref",
        }),
    }
}

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
fn enc_mfcr(rd: u32, cr: u32) -> u32 {
    (0x1F << 26) | (rd << 22) | (cr << 18)
}
fn enc_mtcr(cr: u32, rs: u32) -> u32 {
    (0x20 << 26) | (cr << 22) | (rs << 18)
}

fn encode_rrr_instr(
    op: u32,
    mnemonic: &str,
    operands: &[Operand],
) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 3)?;
    let dst = expect_reg(operands, 0, mnemonic)?;
    let src1 = expect_reg(operands, 1, mnemonic)?;
    let src2 = expect_reg(operands, 2, mnemonic)?;
    Ok(vec![enc_rrr(op, dst as u32, src1 as u32, src2 as u32)])
}

fn encode_rr_instr(
    op: u32,
    mnemonic: &str,
    operands: &[Operand],
) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 2)?;
    let dst = expect_reg(operands, 0, mnemonic)?;
    let src = expect_reg(operands, 1, mnemonic)?;
    Ok(vec![enc_rr(op, dst as u32, src as u32)])
}

fn encode_nullary(op: u32, mnemonic: &str, operands: &[Operand]) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 0)?;
    Ok(vec![op << 26])
}

fn encode_ri_instr(
    op: u32,
    mnemonic: &str,
    operands: &[Operand],
) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 2)?;
    let dst = expect_reg(operands, 0, mnemonic)?;
    let imm = expect_imm(operands, 1, mnemonic)?;
    if !(0..=0xFFFF).contains(&imm) {
        Err(CodegenError::ImmediateOutOfRange {
            value: imm,
            bits: 16,
        })
    } else {
        Ok(vec![enc_ri(op, dst as u32, imm as u16)])
    }
}

fn encode_li(
    mnemonic: &str,
    operands: &[Operand],
    symbols: &HashMap<String, u32>,
) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 2)?;
    let dst = expect_reg(operands, 0, mnemonic)?;
    let imm = match &operands[1] {
        Operand::Imm(i) => Ok(*i),
        Operand::Label(name) => symbols
            .get(name)
            .map(|&a| a as i64)
            .ok_or(CodegenError::UndefinedLabel(name.clone())),
        _ => Err(CodegenError::WrongOperandType {
            mnemonic: mnemonic.to_string(),
            position: 1,
            expected: "immediate or label",
        }),
    }?;
    Ok(vec![
        enc_ri(0x04, dst as u32, (imm & 0xFFFF) as u16),
        enc_ri(0x05, dst as u32, (imm >> 16) as u16),
    ])
}

fn encode_rri_instr(
    op: u32,
    mnemonic: &str,
    operands: &[Operand],
) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 2)?;
    let reg = expect_reg(operands, 0, mnemonic)?;
    let (base, offset) = expect_memref(operands, 1, mnemonic)?;
    if !(-(1 << 17)..(1 << 17)).contains(&offset) {
        Err(CodegenError::ImmediateOutOfRange {
            value: offset as i64,
            bits: 18,
        })
    } else {
        Ok(vec![enc_rri(op, reg as u32, base as u32, offset)])
    }
}

fn resolve_offset(
    operands: &[Operand],
    pos: usize,
    mnemonic: &str,
    current_address: u32,
    symbols: &HashMap<String, u32>,
) -> Result<i32, CodegenError> {
    match &operands[pos] {
        Operand::Imm(i) => Ok(*i as i32),
        Operand::Label(name) => Ok(*symbols
            .get(name)
            .ok_or(CodegenError::UndefinedLabel(name.clone()))?
            as i32
            - current_address as i32),
        _ => Err(CodegenError::WrongOperandType {
            mnemonic: mnemonic.to_string(),
            position: pos,
            expected: "immediate or label",
        }),
    }
}

fn encode_branch(
    op: u32,
    mnemonic: &str,
    operands: &[Operand],
    current_address: u32,
    symbols: &HashMap<String, u32>,
) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 3)?;
    let lhs = expect_reg(operands, 0, mnemonic)?;
    let rhs = expect_reg(operands, 1, mnemonic)?;
    let offset = resolve_offset(operands, 2, mnemonic, current_address, symbols)?;
    if !(-(1 << 17)..(1 << 17)).contains(&offset) {
        Err(CodegenError::ImmediateOutOfRange {
            value: offset as i64,
            bits: 18,
        })
    } else {
        Ok(vec![enc_rri(op, lhs as u32, rhs as u32, offset)])
    }
}

fn encode_bal(
    mnemonic: &str,
    operands: &[Operand],
    current_address: u32,
    symbols: &HashMap<String, u32>,
) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 2)?;
    let link = expect_reg(operands, 0, mnemonic)?;
    let offset = resolve_offset(operands, 1, mnemonic, current_address, symbols)?;
    if !(-(1 << 21)..(1 << 21)).contains(&offset) {
        Err(CodegenError::ImmediateOutOfRange {
            value: offset as i64,
            bits: 22,
        })
    } else {
        Ok(vec![enc_bal(link as u32, offset)])
    }
}

fn encode_mfcr(mnemonic: &str, operands: &[Operand]) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 2)?;
    let rd = expect_reg(operands, 0, mnemonic)?;
    let cr = expect_ctrl_reg(operands, 1, mnemonic)?;
    Ok(vec![enc_mfcr(rd as u32, cr as u32)])
}

fn encode_mtcr(mnemonic: &str, operands: &[Operand]) -> Result<Vec<u32>, CodegenError> {
    check_operand_count(mnemonic, operands, 2)?;
    let cr = expect_ctrl_reg(operands, 0, mnemonic)?;
    let rs = expect_reg(operands, 1, mnemonic)?;
    Ok(vec![enc_mtcr(cr as u32, rs as u32)])
}

fn encode_instruction(
    mnemonic: &str,
    operands: &[Operand],
    current_address: u32,
    symbols: &HashMap<String, u32>,
) -> Result<Vec<u32>, CodegenError> {
    match mnemonic {
        "hlt" => encode_nullary(0x00, mnemonic, operands),
        "nop" => encode_nullary(0x01, mnemonic, operands),
        "mov" => encode_rr_instr(0x02, mnemonic, operands),
        "ldi16" => encode_ri_instr(0x03, mnemonic, operands),
        "ldi32l" => encode_ri_instr(0x04, mnemonic, operands),
        "ldi32h" => encode_ri_instr(0x05, mnemonic, operands),
        "li" => encode_li(mnemonic, operands, symbols),
        "ldm8" => encode_rri_instr(0x06, mnemonic, operands),
        "ldm16" => encode_rri_instr(0x07, mnemonic, operands),
        "ldm32" => encode_rri_instr(0x08, mnemonic, operands),
        "str8" => encode_rri_instr(0x09, mnemonic, operands),
        "str16" => encode_rri_instr(0x0A, mnemonic, operands),
        "str32" => encode_rri_instr(0x0B, mnemonic, operands),
        "add" => encode_rrr_instr(0x0C, mnemonic, operands),
        "sub" => encode_rrr_instr(0x0D, mnemonic, operands),
        "mul" => encode_rrr_instr(0x0E, mnemonic, operands),
        "and" => encode_rrr_instr(0x0F, mnemonic, operands),
        "or" => encode_rrr_instr(0x10, mnemonic, operands),
        "xor" => encode_rrr_instr(0x11, mnemonic, operands),
        "not" => encode_rr_instr(0x12, mnemonic, operands),
        "sll" => encode_rrr_instr(0x13, mnemonic, operands),
        "slr" => encode_rrr_instr(0x14, mnemonic, operands),
        "sar" => encode_rrr_instr(0x15, mnemonic, operands),
        "beq" => encode_branch(0x16, mnemonic, operands, current_address, symbols),
        "bne" => encode_branch(0x17, mnemonic, operands, current_address, symbols),
        "blt" => encode_branch(0x18, mnemonic, operands, current_address, symbols),
        "bge" => encode_branch(0x19, mnemonic, operands, current_address, symbols),
        "bltu" => encode_branch(0x1A, mnemonic, operands, current_address, symbols),
        "bgeu" => encode_branch(0x1B, mnemonic, operands, current_address, symbols),
        "bal" => encode_bal(mnemonic, operands, current_address, symbols),
        "ecall" => encode_nullary(0x1D, mnemonic, operands),
        "eret" => encode_nullary(0x1E, mnemonic, operands),
        "mfcr" => encode_mfcr(mnemonic, operands),
        "mtcr" => encode_mtcr(mnemonic, operands),
        _ => Err(CodegenError::UnknownMnemonic(mnemonic.to_string())),
    }
}

fn emit(items: &[Item], symbols: &HashMap<String, u32>) -> Result<Vec<u8>, CodegenError> {
    let mut output: Vec<u8> = Vec::new();
    let mut current_address: u32 = 0;

    for item in items {
        match item {
            Item::Label(_) => {}
            Item::Directive { name, values } => match name.as_str() {
                "word" => {
                    for &v in values {
                        output.extend_from_slice(&(v as u32).to_be_bytes());
                        current_address += 4;
                    }
                }
                "byte" | "str" => {
                    for &v in values {
                        output.push(v as u8);
                        current_address += 1;
                    }
                }
                _ => return Err(CodegenError::UnknownDirective(name.clone())),
            },
            Item::Instruction { mnemonic, operands } => {
                let words = encode_instruction(mnemonic, operands, current_address, symbols)?;
                for word in &words {
                    output.extend_from_slice(&word.to_be_bytes());
                }
                current_address += 4 * words.len() as u32;
            }
        }
    }

    Ok(output)
}

pub fn codegen(items: &[Item]) -> Result<Vec<u8>, CodegenError> {
    let symbols = build_symbol_table(items)?;
    emit(items, &symbols)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bliss::register::Register;

    fn instr(mnemonic: &str, operands: Vec<Operand>) -> Item {
        Item::Instruction {
            mnemonic: mnemonic.to_string(),
            operands,
        }
    }

    fn reg(r: Register) -> Operand {
        Operand::Reg(r)
    }

    fn ctrl_reg(cr: ControlReg) -> Operand {
        Operand::ControlReg(cr)
    }

    fn imm(i: i64) -> Operand {
        Operand::Imm(i)
    }

    fn label(name: &str) -> Operand {
        Operand::Label(name.to_string())
    }

    fn memref(base: Register, offset: i32) -> Operand {
        Operand::MemRef { base, offset }
    }

    fn bytes_of(words: &[u32]) -> Vec<u8> {
        words.iter().flat_map(|w| w.to_be_bytes()).collect()
    }

    #[test]
    fn encode_hlt() {
        assert_eq!(
            codegen(&[instr("hlt", vec![])]).unwrap(),
            bytes_of(&[0x00 << 26])
        );
    }

    #[test]
    fn encode_add() {
        let items = vec![instr(
            "add",
            vec![reg(Register::R2), reg(Register::R3), reg(Register::R1)],
        )];
        let expected = (0x0C_u32 << 26) | (2 << 22) | (3 << 18) | (1 << 14);
        assert_eq!(codegen(&items).unwrap(), bytes_of(&[expected]));
    }

    #[test]
    fn encode_ecall() {
        assert_eq!(
            codegen(&[instr("ecall", vec![])]).unwrap(),
            bytes_of(&[0x1D << 26])
        );
    }

    #[test]
    fn encode_mfcr() {
        let items = vec![instr(
            "mfcr",
            vec![reg(Register::R1), ctrl_reg(ControlReg::Cause)],
        )];
        let expected = (0x1F << 26) | (1 << 22) | (2 << 18);
        assert_eq!(codegen(&items).unwrap(), bytes_of(&[expected]));
    }

    #[test]
    fn encode_mtcr() {
        let items = vec![instr(
            "mtcr",
            vec![ctrl_reg(ControlReg::Cause), reg(Register::R1)],
        )];
        let expected = (0x20 << 26) | (2 << 22) | (1 << 18);
        assert_eq!(codegen(&items).unwrap(), bytes_of(&[expected]));
    }

    #[test]
    fn encode_li_expands_to_two_words() {
        let items = vec![instr("li", vec![reg(Register::R0), imm(0xDEAD_BEEF)])];
        let bytes = codegen(&items).unwrap();
        assert_eq!(bytes.len(), 8);
        let lo_word = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let hi_word = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        assert_eq!(lo_word & 0xFFFF, 0xBEEF);
        assert_eq!(hi_word & 0xFFFF, 0xDEAD);
    }

    #[test]
    fn encode_branch_resolves_label() {
        let items = vec![
            Item::Label("target".to_string()),
            instr("nop", vec![]),
            instr(
                "beq",
                vec![reg(Register::R0), reg(Register::R1), label("target")],
            ),
        ];
        let bytes = codegen(&items).unwrap();
        // beq is at address 4; target is at address 0; offset = 0 - 4 = -4
        let beq_word = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let offset_field = ((beq_word << 14) as i32) >> 14;
        assert_eq!(offset_field, -4);
    }

    #[test]
    fn word_directive_emits_big_endian() {
        let items = vec![Item::Directive {
            name: "word".to_string(),
            values: vec![0xDEAD_BEEF],
        }];
        assert_eq!(codegen(&items).unwrap(), vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn error_undefined_label() {
        let items = vec![instr(
            "beq",
            vec![reg(Register::R0), reg(Register::R1), label("nowhere")],
        )];
        assert!(matches!(
            codegen(&items),
            Err(CodegenError::UndefinedLabel(name)) if name == "nowhere"
        ));
    }

    #[test]
    fn error_immediate_out_of_range() {
        let items = vec![instr("ldi16", vec![reg(Register::R0), imm(0x10000)])];
        assert!(matches!(
            codegen(&items),
            Err(CodegenError::ImmediateOutOfRange {
                value: 0x10000,
                bits: 16
            })
        ));
    }

    #[test]
    fn error_wrong_number_of_operands() {
        let items = vec![instr("add", vec![reg(Register::R0)])];
        assert!(matches!(
            codegen(&items),
            Err(CodegenError::WrongNumberOfOperands {
                expected: 3,
                got: 1,
                ..
            })
        ));
    }

    #[test]
    fn error_wrong_operand_type() {
        let items = vec![instr(
            "add",
            vec![imm(0), reg(Register::R0), reg(Register::R1)],
        )];
        assert!(matches!(
            codegen(&items),
            Err(CodegenError::WrongOperandType { position: 0, .. })
        ));
    }

    #[test]
    fn symbol_table_accounts_for_li_size() {
        // li is 8 bytes; label after it should be at address 8
        let items = vec![
            instr("li", vec![reg(Register::R0), imm(0)]),
            Item::Label("after".to_string()),
            instr("hlt", vec![]),
        ];
        let symbols = build_symbol_table(&items).unwrap();
        assert_eq!(symbols["after"], 8);
    }

    #[test]
    fn encode_memref_with_negative_offset() {
        let items = vec![instr(
            "ldm32",
            vec![reg(Register::R0), memref(Register::R1, -4)],
        )];
        let bytes = codegen(&items).unwrap();
        let word = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let offset_field = ((word << 14) as i32) >> 14;
        assert_eq!(offset_field, -4);
    }
}
