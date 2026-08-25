use asm::assemble;
use bliss::instruction::Instruction;
use bliss::register::Register;
use bliss::vm::{StepResult, Vm};

fn run_to_halt(bytes: &[u8]) -> Vm {
    let mut vm = Vm::new();
    vm.load(0, bytes).unwrap();
    for _ in 0..10_000 {
        let word = vm.fetch().unwrap();
        let instr = Instruction::decode(word).unwrap();
        match vm.execute(instr).unwrap() {
            StepResult::Halt => return vm,
            StepResult::Continue => {}
        }
    }
    panic!("program did not halt within step limit");
}

#[test]
fn hlt_halts_immediately() {
    let vm = run_to_halt(&assemble("hlt").unwrap());
    assert_eq!(vm.reg(Register::R0), 0);
}

#[test]
fn li_loads_32bit_immediate() {
    let vm = run_to_halt(&assemble("li r0, 0xDEADBEEF\nhlt").unwrap());
    assert_eq!(vm.reg(Register::R0), 0xDEAD_BEEF);
}

#[test]
fn add_two_registers() {
    let src = "
        li r0, 20
        li r1, 22
        add r2, r0, r1
        hlt
    ";
    let vm = run_to_halt(&assemble(src).unwrap());
    assert_eq!(vm.reg(Register::R2), 42);
}

#[test]
fn countdown_loop_with_branch() {
    let src = "
        li r0, 3
        li r1, 1
        li r2, 0
        loop:
          sub r0, r0, r1
          bne r0, r2, loop
        hlt
    ";
    let vm = run_to_halt(&assemble(src).unwrap());
    assert_eq!(vm.reg(Register::R0), 0);
}

#[test]
fn bal_jumps_and_saves_return_address() {
    let src = "
        bal lr, func
        hlt
        func:
          li r0, 42
          hlt
    ";
    let vm = run_to_halt(&assemble(src).unwrap());
    assert_eq!(vm.reg(Register::R0), 42);
    assert_eq!(vm.reg(Register::LR), 4);
}

#[test]
fn store_and_load_roundtrip() {
    let src = "
        li r0, 0x800
        li r1, 0xABCD
        str32 r1, [r0]
        ldm32 r2, [r0]
        hlt
    ";
    let vm = run_to_halt(&assemble(src).unwrap());
    assert_eq!(vm.reg(Register::R2), 0xABCD);
}
