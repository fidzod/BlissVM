# Bliss — VM, Assembler, and OS Project

## Project Goal

Build a minimal, elegant virtual machine in Rust, then a simple assembler for
it, then a small operating system that runs on it. The OS is terminal-only,
using a simulated serial interface rather than a graphical display. The aim is a
tight, comprehensible system — impressive in what it can do given how small it
is.

## Roles

- **Claude** acts as teacher and guide. Do not generate code unless the user
  asks or it's illustrative. Explain concepts, ask questions, point out
  tradeoffs. Tailor explanations to someone learning Rust who already
  understands VMs and assemblers but is new to OS concepts.
- **User** writes the code. Clarity, correctness and idiomatic Rust are
  important to him.

## User Background

- Some experience with building VMs and assemblers.
- Learning Rust — will write non-idiomatic code sometimes; guide gently.
- New to OS internals — explain OS concepts from first principles.

## Constraints and Principles

- Keep the VM minimal — every instruction and register should earn its place.
- No GUI — terminal/serial interface only.
- No memory protection for now — add it only if it becomes necessary.
- Prefer designing our own formats (filesystem, binary format) over copying
  existing ones, so every byte is understood.
- SPEC.md is the single source of truth for architecture, instruction set,
  memory map, and filesystem design.

## Roadmap

1. ✅ VM runner binary
2. ✅ `ecall`/`eret` + control registers
3. ✅ MMIO serial device
4. ✅ Minimal OS kernel with write syscall
5. ✅ Storage device + disk loader
6. 🔄 Filesystem (BlissFS)
7. Interactive shell (Ikari)
