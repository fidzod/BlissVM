use std::fs;

use bliss::{
    error::VmError,
    instruction::Instruction,
    vm::{StepResult, Vm},
};

fn run(vm: &mut Vm) -> Result<(), VmError> {
    loop {
        let word = vm.fetch()?;
        let instr = Instruction::decode(word)?;
        match vm.execute(instr)? {
            StepResult::Halt => break,
            StepResult::Continue => {}
        }
    }
    Ok(())
}

fn main() {
    let bin_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: blissvm <program.bin>");
        std::process::exit(1)
    });

    let mut vm = Vm::new();
    let source = fs::read(bin_path.clone()).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", bin_path, e);
        std::process::exit(1)
    });

    vm.load(0, &source).unwrap_or_else(|e| {
        eprintln!("Error loading program: {:?}", e);
        std::process::exit(1)
    });

    run(&mut vm).unwrap_or_else(|e| {
        eprintln!("Error during program execution: {:?}", e);
        std::process::exit(1)
    });
}
