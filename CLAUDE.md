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
9. ✅ Standard library + assembler includes
10. 🔄 Real OS — boot process, process model, scheduler, exit/exec syscalls
11. ⬜ Ikari (interactive shell)
12. ⬜ Memory protection + privilege levels
13. ⬜ Gendo (compiled language targeting Bliss ISA)

## Current Focus

**Real OS — boot, process model, scheduler (Phase 4 — in progress).**

Phase 3 is complete: `.include`, `.define`, `stdlib_readline`, and the full
trap frame are all working. The memory layout has been redesigned for
multi-process support:

```
0x0000–0x0FFF   Kernel code            (4 KB)
0x1000–0x11FF   Kernel disk scratch    (512 B, one sector)
0x1200–0x15FF   Process table          (4 slots × 256 B; slot N at 0x1200 + N<<8)
0x1600–0x17FF   Kernel data            (CURPID at 0x1600; rest reserved)
0x1800–0x1FFF   Kernel stack           (2 KB; SP starts at 0x1FFC)
0x2000–0x3FFF   Process 0              (8 KB; code at base, stack top 0x3FFC)
0x4000–0x5FFF   Process 1              (8 KB)
0x6000–0x7FFF   Process 2              (8 KB)
0x8000–0x9FFF   Process 3              (8 KB)
0xFFFF0000+     MMIO (unchanged)
```

Phase 4 checkpoints:

1. **Boot** — design the boot handoff: what is the "first process", how does
   the kernel create process 0 and ERET into it?
2. **Process table** — implement the 4-slot table in kernel.asm; formalize
   the per-slot layout (status, saved PC, saved SP, saved registers).
3. **Syscalls** — `exit` (0): mark slot dead, schedule next; `yield`: save
   state, schedule next; `exec`: load named file from disk into a slot, set
   it runnable.
4. **Scheduler** — a simple round-robin scan over the process table; called
   from `exit` and `yield`.
5. **Memory map** — update kernel.asm defines (SPINIT, USERSTART, per-process
   stack tops) to match new layout.
6. **Multi-process test** — a minimal two-process smoke test running on the
   new kernel.

**Ikari (Phase 5).**

With stdlib and a real process model in place, Ikari is mostly assembly:
print a prompt, call `stdlib_readline`, parse the result (split on spaces,
argv[0] is the command), call `exec`. Programs live on disk (e.g. `/bin/echo`)
and are loaded by the kernel on `exec`.
