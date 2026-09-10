use std::fs;

use bliss::{
    error::VmError,
    instruction::Instruction,
    vm::{StepResult, Vm},
};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW, VMIN, VTIME};

struct RawMode {
    saved: Termios,
}

impl RawMode {
    fn enter() -> Option<Self> {
        Termios::from_fd(0).ok().map(|mut termios| {
            let saved = termios.clone();
            termios.c_lflag &= !(ICANON | ECHO);
            termios.c_cc[VMIN] = 1;
            termios.c_cc[VTIME] = 0;
            tcsetattr(0, TCSANOW, &termios).expect("Could not set raw mode");
            RawMode { saved }
        })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        tcsetattr(0, TCSANOW, &self.saved).ok();
    }
}

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

    if let Some(disk_path) = std::env::args().nth(2) {
        let disk_data = fs::read(disk_path.clone()).unwrap_or_else(|e| {
            eprintln!("Failed to read disk {}: {}", disk_path, e);
            std::process::exit(1)
        });
        vm.load_disk(disk_data);
    }

    let _raw = RawMode::enter();

    if let Err(e) = run(&mut vm) {
        drop(_raw);
        eprintln!("Error during program execution: {:?}", e);
        std::process::exit(1);
    }
}
