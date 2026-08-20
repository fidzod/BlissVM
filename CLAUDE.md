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

## Architecture Decisions

- **VM type:** Register-based
- **Word size:** 32-bit (flat address space, no banking/segmentation tricks needed)
- **Registers:** 16 total, all general-purpose; r15=PC, r14=LR, r13=SP reserved by convention; no flags register
- **Instruction encoding:** Fixed-width 32-bit instructions
- **Memory model:** Load/store — arithmetic operates only on registers; explicit LOAD/STORE for memory access; CALL/RET are assembler pseudo-instructions
- **Branching:** Comparison-in-branch — no flags register, no CMP; branch instructions take two register operands and compare directly (BEQ, BNE, BLT, BGE, BLTU, BGEU); targets are PC-relative offsets
- **Memory access variants:** LOAD/STORE in 8, 16, and 32-bit widths; byte/halfword loads zero-extend into the register
- **I/O:** Memory-mapped — no special I/O instructions; devices appear at fixed addresses
- **Multiply/divide:** MUL in hardware (32-bit truncated); DIV as a software routine in the assembler stdlib
- **Assembler stdlib:** A small standard library of subroutines (DIV, string ops, etc.) linked by the assembler
- **Assembler role:** Pseudo-instructions are fine, but should expand to at most a handful of real instructions

## Constraints and Principles

- Keep the VM minimal — every instruction and register should earn its place.
- Decisions about the OS do not need to be made before the VM is designed, but
  the VM design should not foreclose OS possibilities.
- No GUI — terminal/serial interface only.

_Expand as modules are added._
