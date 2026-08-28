# Bliss

A minimal virtual machine for a custom 32-bit ISA, built in Rust.

The project is an exercise in building a complete system from scratch: a
register-based VM, an assembler for it, memory-mapped I/O, a privilege
mechanism, and eventually a bare-metal OS with a simple filesystem and
interactive shell. Every layer is hand-rolled — the goal is a system where
every byte is understood.

The ISA draws on RISC-V and early ARM: fixed-width 32-bit instructions,
load/store architecture, 16 general-purpose registers, comparison-in-branch
(no flags register), and MMIO for device access.

## Status

| Component         | State       |
|-------------------|-------------|
| VM                | Working     |
| Assembler         | Working     |
| MMIO serial port  | Working     |
| Privilege / traps | Working     |
| OS kernel         | In Progress |
| Storage device    | In Progress |
| Filesystem        | Planned     |
| Shell             | Planned     |

## Crates

- **`bliss`** — the VM library and runner binary (`bliss <file.bin>`)
- **`asm`** — the assembler library and CLI (`asm <file.asm>` → `file.bin`)

## Quick start

```sh
cargo build --release

# Assemble a program
./target/release/asm examples/hello.asm

# Run it
./target/release/bliss examples/hello.bin
```

## AI usage

This project is written by hand as a learning exercise. Claude acts as a
teacher and guide — explaining concepts, asking questions, pointing out
tradeoffs — and is explicitly instructed not to generate code. All
implementation is my own.

## ISA overview

See [SPEC.md](SPEC.md) for the full instruction set, encoding formats, memory
map, and privilege mechanism.
