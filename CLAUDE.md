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

### VM

- **VM type:** Register-based
- **Word size:** 32-bit (flat address space, no banking/segmentation tricks needed)
- **Registers:** 16 general-purpose; r15=PC, r14=LR, r13=SP reserved by convention; no flags register
- **Instruction encoding:** Fixed-width 32-bit instructions
- **Memory model:** Load/store — arithmetic operates only on registers; explicit LOAD/STORE for memory access
- **Branching:** Comparison-in-branch — no flags register; BEQ, BNE, BLT, BGE, BLTU, BGEU take two register operands and compare directly; targets are PC-relative offsets
- **Memory access variants:** 8, 16, and 32-bit widths; byte/halfword loads zero-extend
- **I/O:** Memory-mapped — devices appear at fixed addresses in the upper address space
- **Multiply/divide:** MUL in hardware (32-bit truncated); DIV as a software routine
- **Privilege modes:** User and supervisor (1-bit mode flag). To be added before OS work.
- **Trap mechanism:** `ecall` instruction triggers a trap from user→supervisor; `eret` returns.
  Four control registers added alongside the general-purpose file:
  - `tvec` — trap vector: address of the OS trap handler
  - `epc` — exception PC: return address saved on trap
  - `cause` — reason for the trap (syscall, illegal instruction, …)
  - `mode` — current privilege level
- **Memory protection:** Not planned for now. OS and user code are trusted. Can be revisited.

### Assembler

- **Crate:** `asm` — library + binary in one crate (lib.rs exposes `assemble()`)
- **Pseudo-instructions:** Expand to at most a handful of real instructions (`li` → ldi32l + ldi32h; `call`/`ret` TBD)
- **Stdlib:** Small library of subroutines (DIV, string ops, etc.) to be linked by the assembler
- **Output:** Flat binary (`.bin`)

### Memory Map

- `0x0000_0000` — code and data (RAM, 4 KB currently, will grow)
- `0xFFFF_0000`+ — MMIO device space
  - `0xFFFF_0000` — serial port TX (write a byte → stdout)
  - `0xFFFF_0001` — serial port RX (read a byte ← stdin)
  - Storage device address TBD

### OS Roadmap

1. VM runner binary (`bliss <file.bin>`) — execute silently; report errors and exit 1
2. Add `ecall`/`eret` + control registers to VM and assembler
3. MMIO serial device — programs can print/read without syscalls
4. Minimal OS kernel: boots in supervisor mode, installs trap handler, handles a `write` syscall
5. Loader: OS loads a user program from the storage device, runs it in user mode
6. Storage MMIO device (simulated disk — `Vec<u8>` in the VM, mapped to an address range)
7. Filesystem — custom design (superblock, inode table, free-block bitmap, data blocks)
8. Interactive shell

### Filesystem Design (planned, not started)

Custom design rather than ext2. Classic Unix shape: superblock at block 0,
inode table in early blocks, free-block bitmap, data blocks. Block size TBD.
The goal is something comprehensible and hand-implementable, not POSIX-complete.

## Constraints and Principles

- Keep the VM minimal — every instruction and register should earn its place.
- No GUI — terminal/serial interface only.
- No memory protection for now — add it only if it becomes necessary.
- Prefer designing our own formats (filesystem, binary format) over copying existing ones,
  so every byte is understood.
