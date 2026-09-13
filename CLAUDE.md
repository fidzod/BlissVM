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
- Prefer designing our own formats (filesystem, binary format) over copying
  existing ones, so every byte is understood.
- SPEC.md is the single source of truth for architecture, instruction set,
  memory map, and filesystem design.
- Simple first, then add complexity once you can *see* why it's needed. The
  goal is discovery, not emulation of a known design.
- The OS should be real. Each piece — boot, scheduler, context switch — should
  be genuinely implemented and fully explainable, not approximated.
- Memory protection is planned but postponed until after Ikari; add it when
  there is something worth protecting.
- Known limitation: `hlt` is currently honoured unconditionally by the VM. In
  a real processor it is a privileged instruction; user-mode `hlt` should
  fault. This will be addressed when privilege levels are added.

## Roadmap

1. ✅ VM runner binary
2. ✅ `ecall`/`eret` + control registers
3. ✅ MMIO serial device
4. ✅ Minimal OS kernel with write syscall
5. ✅ Storage device + disk loader
6. ✅ Filesystem (BlissFS)
7. ✅ Stack + calling convention (BALR, pseudo-ops, register ABI)
8. ✅ Syscall dispatcher (write + read, trap handler)
9. 🔄 Standard library + assembler includes
10. ⬜ Real OS — boot process, process model, scheduler, exit/exec syscalls
11. ⬜ Ikari (interactive shell)
12. ⬜ Memory protection + privilege levels
13. ⬜ Gendo (compiled language targeting Bliss ISA)

## Current Focus

**Standard library + assembler includes (Phase 3 — in progress).**

The assembler needs a `.include "file.asm"` directive. Implementation: before
the first pass, recursively expand includes by splicing file contents in at the
point of the directive, producing one flat line stream for the existing two-pass
assembler. No changes to the assembler itself are required.

`stdlib.asm` is the first included file. Initial contents: `stdlib_readline` —
reads chars via `sys_read` into a caller-supplied buffer, returns on `\n`,
gives back length. End goal: `user.asm` uses `stdlib_readline` rather than
doing it inline.

**Real OS — boot, process model, scheduler (Phase 4).**

Before Ikari, the kernel needs to become a real operating system:

- A genuine boot process: the CPU starts at a fixed address; the kernel sets
  itself up, then explicitly hands control to user space.
- A process table: fixed-size array of slots, each holding saved registers,
  PC, SP, and status.
- Cooperative scheduling to start (switch only on `exit` or `yield`); add
  preemption once the VM has a timer interrupt and you can see why it's needed.
- New syscalls: `exit` (0), `exec` (load a named file from disk into a process
  slot and schedule it), `yield`. Disk access for user programs goes through
  syscalls — no direct MMIO.
- Each process gets its own stack region; the memory map will need updating.
- Argument passing convention: r1 = argc, r2 = pointer to argv array.

**Ikari (Phase 5).**

With stdlib and a real process model in place, Ikari is mostly assembly:
print a prompt, call `stdlib_readline`, parse the result (split on spaces,
argv[0] is the command), call `exec`. Programs live on disk (e.g. `/bin/echo`)
and are loaded by the kernel on `exec`.
