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
6. ✅ Filesystem (BlissFS)
7. 🔄 Interactive shell (Ikari)

## Current Focus

The next goal is Ikari — an interactive shell that reads commands from serial
input and executes them. Before writing the shell itself, two prerequisites are
worth addressing first:

**Stack + calling convention (Phase 1 — in progress).** The ISA needs one new
instruction, `BALR rd, rs` (branch-and-link to register), to make subroutine
returns possible. Once that is in the VM and assembler, five pseudo-instructions
follow: `PUSH`, `POP`, `CALL`, `RET`, and `JMP`. The kernel then initialises SP
to `0x1FFC` at boot. Register convention: r0 is assembler scratch (always
clobberable), r1–r7 are caller-saved, r8–r12 are callee-saved, r13 is SP,
r14 is LR. Full details in SPEC.md.

**Proper syscall dispatch (Phase 2).** The trap handler currently assumes every
trap is a write syscall. A real dispatcher reads `cause` and `r0` (syscall
number) and branches to the right handler. The shell needs at minimum write (1)
and read (2) — read blocking on a byte from the serial RX port.

Once those two are in place, the shell can be built without running into dead
ends. After the shell is working, remaining loose ends to address include:

- Exception handling (invalid instruction, out-of-bounds, etc.)
- Assembly imports/includes for a small standard library (division, string ops)
- A cleaner kernel boot model (the current kernel is instructive but rough)
- Memory protection — user code can currently access any address including MMIO
  and kernel memory; worth adding once the shell gives us something worth protecting
- Gendo — a small compiled language targeting the Bliss ISA; a natural next step
  once the full stack (VM, assembler, OS, shell) is solid
