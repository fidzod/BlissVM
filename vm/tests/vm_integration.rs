use bliss::instruction::Instruction;
use bliss::register::Register;
use bliss::vm::{StepResult, Vm};

fn run(vm: &mut Vm) {
    loop {
        let word = vm.fetch().unwrap();
        let instr = Instruction::decode(word).unwrap();
        match vm.execute(instr).unwrap() {
            StepResult::Halt => break,
            StepResult::Continue => {}
        }
    }
}

fn load_words(vm: &mut Vm, words: &[u32]) {
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_be_bytes()).collect();
    vm.load(0, &bytes).unwrap();
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

const HLT: u32 = 0;

#[test]
fn program_add_two_immediates() {
    // ldi16 r0 60 / ldi16 r1 7 / add r2 r0 r1 / hlt → r2 = 67
    let mut vm = Vm::new();
    load_words(
        &mut vm,
        &[
            enc_ri(0x03, 0, 60),
            enc_ri(0x03, 1, 7),
            enc_rrr(0x0C, 2, 0, 1),
            HLT,
        ],
    );
    run(&mut vm);
    assert_eq!(vm.reg(Register::R2), 67);
}

#[test]
fn program_countdown_loop() {
    // r0 counts from 5 down to 0 via repeated sub; r1=1, r2=0 (zero reg by convention)
    //
    // addr  0: ldi16 r0 5
    // addr  4: ldi16 r1 1
    // addr  8: sub r0 r0 r1     ← loop back target
    // addr 12: bne r0 r2 -4     → when r0 != 0, fetch-then-execute lands at addr 8
    // addr 16: hlt
    let mut vm = Vm::new();
    load_words(
        &mut vm,
        &[
            enc_ri(0x03, 0, 5),
            enc_ri(0x03, 1, 1),
            enc_rrr(0x0D, 0, 0, 1),
            enc_rri(0x17, 0, 2, -4),
            HLT,
        ],
    );
    run(&mut vm);
    assert_eq!(vm.reg(Register::R0), 0);
}

#[test]
fn program_call_and_return() {
    // bal jumps to a subroutine that doubles r0, then mov pc lr returns.
    //
    // addr  0: ldi16 r0 21
    // addr  4: bal lr 12        → LR=8, PC = 8-4+12 = 16
    // addr  8: hlt              ← return destination
    // addr 12: (pad)
    // addr 16: add r0 r0 r0     ← subroutine
    // addr 20: mov pc lr        → PC = LR = 8
    let mut vm = Vm::new();
    load_words(
        &mut vm,
        &[
            enc_ri(0x03, 0, 21),
            enc_bal(14, 12),
            HLT,
            HLT,
            enc_rrr(0x0C, 0, 0, 0),
            enc_rr(0x02, 15, 14),
        ],
    );
    run(&mut vm);
    assert_eq!(vm.reg(Register::R0), 42);
    assert_eq!(vm.reg(Register::LR), 8);
}

#[test]
fn program_store_and_load_roundtrip() {
    // Writes a 32-bit value to memory, then reads it back in two halfword loads.
    //
    // addr  0: ldi32l r0 0xBEEF
    // addr  4: ldi32h r0 0xDEAD  → r0 = 0xDEAD_BEEF
    // addr  8: ldi16 r1 0x200    → base address
    // addr 12: str32 r0 r1 0
    // addr 16: ldm16 r2 r1 0     → r2 = 0x0000_DEAD
    // addr 20: ldm16 r3 r1 2     → r3 = 0x0000_BEEF
    // addr 24: hlt
    let mut vm = Vm::new();
    load_words(
        &mut vm,
        &[
            enc_ri(0x04, 0, 0xBEEF),
            enc_ri(0x05, 0, 0xDEAD),
            enc_ri(0x03, 1, 0x200),
            enc_rri(0x0B, 0, 1, 0),
            enc_rri(0x07, 2, 1, 0),
            enc_rri(0x07, 3, 1, 2),
            HLT,
        ],
    );
    run(&mut vm);
    assert_eq!(vm.reg(Register::R0), 0xDEAD_BEEF);
    assert_eq!(vm.reg(Register::R2), 0x0000_DEAD);
    assert_eq!(vm.reg(Register::R3), 0x0000_BEEF);
}
