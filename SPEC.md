# Bliss ISA Specification

## Architecture

- 32-bit register-based VM
- 16 general-purpose 32-bit registers (r0–r15)
  - r0  — assembler scratch / return value (reserved; see Calling Convention)
  - r13 — stack pointer (SP)
  - r14 — link register (LR)
  - r15 — program counter (PC, not directly writable)
- Fixed-width 32-bit instructions
- Load/store: arithmetic on registers only; memory accessed via explicit loads/stores
- Memory-mapped I/O; no special I/O instructions
- Flat 32-bit address space

## Privilege Modes

Two privilege levels: **user** (0) and **supervisor** (1). The current mode is tracked in
the `mode` control register. The CPU boots in supervisor mode.

Four **control registers** exist alongside the general-purpose file:

| Name    | Index | Description                                         |
|---------|-------|-----------------------------------------------------|
| `tvec`  | 0     | Trap vector: address of the supervisor trap handler |
| `epc`   | 1     | Exception PC: return address saved on trap          |
| `cause` | 2     | Trap cause code (0 = syscall, …)                    |
| `mode`  | 3     | Current privilege level (0 = user, 1 = supervisor)  |

Control registers are read and written with `MFCR`/`MTCR`.

### Trap flow (`ECALL`)

When `ECALL` executes:
1. `epc ← PC` (address of the instruction following `ECALL`)
2. `cause ← 0` (syscall)
3. `mode ← supervisor`
4. `PC ← tvec`

### Return flow (`ERET`)

When `ERET` executes:
1. `mode ← user`
2. `PC ← epc`

No registers are saved or restored automatically; that is the trap handler's responsibility.

## Memory Map

| Address range                   | Use                                                |
|---------------------------------|----------------------------------------------------|
| `0x0000_0000` – `0x0000_0FFF`  | Kernel code and data                               |
| `0x0000_1000` – `0x0000_11FF`  | Kernel scratch buffer (disk I/O, one sector)       |
| `0x0000_1200` – `0x0000_1FFB`  | Kernel stack (grows downward; SP starts at 0x1FFC) |
| `0x0000_2000` –                | User program                                       |
| `0xFFFF_0000`                  | Serial TX (write byte → stdout)                    |
| `0xFFFF_0004`                  | Serial RX (read byte ← stdin, blocks until ready)  |
| `0xFFFF_0010`                  | Disk sector register                               |
| `0xFFFF_0014`                  | Disk buffer register                               |
| `0xFFFF_0018`                  | Disk command (0 = read sector)                     |

Memory protection is not implemented; user code may access any address.

## Instruction Set

### Data Movement
- `MOV`              — register to register
- `LDI16`            — load 16-bit immediate (zero-extended)
- `LDI32L`           — load immediate into lower 16 bits of register
- `LDI32H`           — load immediate into upper 16 bits of register
- `LDM8/16/32`       — load from memory into register (zero-extended for 8/16)
- `STR8/16/32`       — store register to memory

### Arithmetic
- `ADD`, `SUB`, `MUL`
- `AND`, `OR`, `XOR`, `NOT`
- `SLL`, `SLR`       — logical left/right shift
- `SAR`              — arithmetic right shift

### Control Flow
- `BEQ`, `BNE`       — branch if equal/not equal
- `BLT`, `BGE`       — branch if less than/greater-or-equal (signed)
- `BLTU`, `BGEU`     — branch if less than/greater-or-equal (unsigned)
- `BAL`              — branch and link: PC-relative jump, save return address to register
- `BALR`             — branch and link to register: jump to address in register, save return address
- `HLT`              — halt
- `NOP`              — no operation

### Privilege
- `ECALL`            — trap from user mode to supervisor (see above)
- `ERET`             — return from trap handler to user mode (see above)
- `MFCR rd, <cr>`    — move from control register into general-purpose register
- `MTCR <cr>, rs`    — move to control register from general-purpose register

## Instruction Formats

All instructions are 32 bits wide. The opcode is always 6 bits [31:26].

**R — register operations** `| 6: opcode | 4: rd | 4: rs1 | 4: rs2 | 14: — |`
ADD, SUB, MUL, AND, OR, XOR, SLL, SLR, SAR, MOV, NOT, BALR (rd=link dest, rs1=target, rs2 unused)

**I — two registers + immediate** `| 6: opcode | 4: r1 | 4: r2 | 18: imm |`
Loads (r1=dst, r2=base, imm=offset), stores (r1=src, r2=base, imm=offset),
branches (r1=rs1, r2=rs2, imm=signed PC-relative offset, ±512 KB reach)

**U — one register + large immediate** `| 6: opcode | 4: rd | 22: imm |`
LDI16/LDI32L/LDI32H (16-bit imm, upper bits unused), BAL (22-bit signed PC-relative offset, ±8 MB reach)

**C — control register access** `| 6: opcode | 4: rd/cr | 4: cr/rs | 14: — |`
MFCR (rd=destination GPR, cr=control register index 0–3),
MTCR (cr=control register index 0–3, rs=source GPR)

**N — no operands** `| 6: opcode | 26: — |`
HLT, NOP, ECALL, ERET

## Opcode Table

| Dec | Hex  | Mnemonic | Format |
|-----|------|----------|--------|
|   0 | 0x00 | `HLT`    | N      |
|   1 | 0x01 | `NOP`    | N      |
|   2 | 0x02 | `MOV`    | R      |
|   3 | 0x03 | `LDI16`  | U      |
|   4 | 0x04 | `LDI32L` | U      |
|   5 | 0x05 | `LDI32H` | U      |
|   6 | 0x06 | `LDM8`   | I      |
|   7 | 0x07 | `LDM16`  | I      |
|   8 | 0x08 | `LDM32`  | I      |
|   9 | 0x09 | `STR8`   | I      |
|  10 | 0x0A | `STR16`  | I      |
|  11 | 0x0B | `STR32`  | I      |
|  12 | 0x0C | `ADD`    | R      |
|  13 | 0x0D | `SUB`    | R      |
|  14 | 0x0E | `MUL`    | R      |
|  15 | 0x0F | `AND`    | R      |
|  16 | 0x10 | `OR`     | R      |
|  17 | 0x11 | `XOR`    | R      |
|  18 | 0x12 | `NOT`    | R      |
|  19 | 0x13 | `SLL`    | R      |
|  20 | 0x14 | `SLR`    | R      |
|  21 | 0x15 | `SAR`    | R      |
|  22 | 0x16 | `BEQ`    | I      |
|  23 | 0x17 | `BNE`    | I      |
|  24 | 0x18 | `BLT`    | I      |
|  25 | 0x19 | `BGE`    | I      |
|  26 | 0x1A | `BLTU`   | I      |
|  27 | 0x1B | `BGEU`   | I      |
|  28 | 0x1C | `BAL`    | U      |
|  29 | 0x1D | `ECALL`  | N      |
|  30 | 0x1E | `ERET`   | N      |
|  31 | 0x1F | `MFCR`   | C      |
|  32 | 0x20 | `MTCR`   | C      |
|  33 | 0x21 | `BALR`   | R      |

Opcodes 34–63 are reserved.

## Pseudo-instructions (assembler-expanded)

Pseudo-instructions are expanded by the assembler into one or more real instructions.
Several pseudo-instructions use **r0 as a temporary** and will clobber it — do not hold
a live value in r0 across a `PUSH`, `POP`, `CALL`, `RET`, or `ADDI`.

| Pseudo-instruction  | Expands to                                                   | Clobbers |
|---------------------|--------------------------------------------------------------|----------|
| `LI rd, imm`        | `LDI32L rd, imm[15:0]` + `LDI32H rd, imm[31:16]`           | —        |
| `PUSH rs`           | `LDI16 r0, 4` · `SUB r13, r13, r0` · `STR32 rs, [r13]`    | r0       |
| `POP rd`            | `LDM32 rd, [r13]` · `LDI16 r0, 4` · `ADD r13, r13, r0`    | r0       |
| `CALL label`        | `PUSH r14` · `BAL r14, label`                               | r0       |
| `RET`               | `POP r14` · `BALR r0, r14`                                  | r0       |
| `JMP label`         | `BAL r0, label`                                             | r0       |
| `JMP rs`            | `BALR r0, rs`                                               | r0       |
| `ADDI rd, imm`      | `LDI16 r0, imm` · `ADD rd, rd, r0`  (imm must fit 16 bits) | r0       |

> `LI rd, imm` is already implemented. The rest are planned for Phase 1.

## Calling Convention

### Register roles

| Register | Role                          | Saved by  |
|----------|-------------------------------|-----------|
| r0       | Assembler scratch / return value | — (never saved; always clobbered freely) |
| r1–r3    | Argument registers            | Caller    |
| r4–r7    | Caller-saved temporaries      | Caller    |
| r8–r12   | Callee-saved                  | Callee    |
| r13      | Stack pointer (SP)            | Always preserved |
| r14      | Link register (LR)            | Saved by `CALL` |
| r15      | Program counter (PC)          | — (not writable) |

**Caller-saved** registers (r0–r7, r14) may be freely overwritten by any function. If the
caller needs their values after a call, it must save them to the stack first.

**Callee-saved** registers (r8–r12) must be preserved across a call: a function that uses
any of them must push them on entry and pop them before returning.

### Stack

- Full descending: SP points to the last pushed value, grows toward lower addresses.
- SP is initialised to `0x1FFC` at kernel boot.
- All pushes and pops are 4-byte aligned.

### Call sequence (using pseudo-instructions)

```asm
; Caller
call my_func      ; saves r14, jumps to my_func, r14 ← return address

; Callee prologue (if it uses callee-saved registers)
push r8
push r9

; ... function body ...

; Callee epilogue
pop r9
pop r8
ret               ; restores r14, jumps to it
```

## Syscall ABI

User programs trap into the kernel with `ECALL`. The convention:

- `r0` — syscall number (on entry to handler)
- `r1`, `r2`, `r3` — arguments
- `r0` — return value (set by handler before `ERET`)

### Cause codes

| `cause` | Meaning  |
|---------|----------|
| 0       | `ECALL` (syscall from user mode) |

### Syscall table

| r0 | Name    | r1         | r2          | Returns       |
|----|---------|------------|-------------|---------------|
| 1  | `write` | buf ptr    | byte count  | —             |
| 2  | `read`  | —          | —           | r0 = byte read (blocks until available) |

## Filesystem

**BlissFS** — a minimal custom filesystem for the Bliss storage device.

### Parameters

| Property          | Value                    |
|-------------------|--------------------------|
| Block size        | 512 bytes (= one sector) |
| Magic number      | `0xB2155F2D`             |
| Max inodes        | 32                       |
| Max filename      | 24 bytes (null-padded)   |
| Max file size     | 8 × 512 = 4096 bytes     |

### Disk layout

```
Block 0:      Superblock
Blocks 1–4:   Inode table  (32 inodes × 64 bytes = 2048 bytes)
Block 5:      Free-block bitmap
Block 6+:     Data blocks
```

### Superblock (block 0, first 20 bytes)

| Offset | Size | Field                |
|--------|------|----------------------|
| 0      | 4    | Magic (`0xB2155F2D`) |
| 4      | 4    | Block size (512)     |
| 8      | 4    | Inode count (32)     |
| 12     | 4    | Data start block (6) |
| 16     | 4    | Free block count     |

### Inode (64 bytes)

| Offset | Size | Field                                          |
|--------|------|------------------------------------------------|
| 0      | 4    | File size in bytes                             |
| 4      | 1    | Type: 0 = unused, 1 = file, 2 = directory      |
| 5      | 3    | Reserved                                       |
| 8      | 32   | 8 × 4-byte direct block pointers (0 = unused)  |
| 40     | 24   | Reserved                                       |

Inode 0 is reserved (null). Inode 1 is the root directory.

### Directory entry (28 bytes)

| Offset | Size | Field                         |
|--------|------|-------------------------------|
| 0      | 24   | Filename, null-padded         |
| 24     | 4    | Inode number (0 = empty slot) |

18 directory entries fit per block (18 × 28 = 504 bytes; 8 bytes unused).
