# BlissVM Spec

## Architecture

- 32-bit register-based VM
- 16 general-purpose 32-bit registers (r0–r15)
  - r15 = PC, r14 = LR (link register), r13 = SP — reserved by convention
- Fixed-width 32-bit instructions
- Load/store: arithmetic on registers only; memory accessed via explicit loads/stores
- Memory-mapped I/O; no special I/O instructions
- Flat 32-bit address space

## Instruction Set

### Data Movement
- `MOV`              — register to register
- `LDI16`            — load 16-bit immediate (zero-extended)
- `LDI32L`           — load immediate into lower 16 bits of register
- `LDI32H`           — load immediate into upper 16 bits of register
- `LDM8/16/32`       — load from memory into register (zero-extended)
- `STR8/16/32`       — store register to memory

### Arithmetic
- `ADD`, `SUB`, `MUL`
- `AND`, `OR`, `XOR`, `NOT`
- `SLL`, `SLR`       — logical left/right shift
- `SAR`              — arithmetic right shift

### Control Flow
- `BEQ`, `BNE`       — branch if equal/not equal
- `BLT`, `BGE`       — branch if less/greater-or-equal (signed)
- `BLTU`, `BGEU`     — branch if less/greater-or-equal (unsigned)
- `BAL`              — branch and link: jump and save return address to a register
- `HLT`              — halt
- `NOP`              — no operation

## Instruction Formats

All instructions are 32 bits wide. The opcode is always 6 bits [31:26].

**R — register operations** `| 6: opcode | 4: rd | 4: rs1 | 4: rs2 | 14: — |`
ADD, SUB, MUL, AND, OR, XOR, SLL, SLR, SAR, MOV, NOT

**I — two registers + immediate** `| 6: opcode | 4: r1 | 4: r2 | 18: imm |`
Loads (r1=dst, r2=base, imm=offset), stores (r1=src, r2=base, imm=offset),
branches (r1=rs1, r2=rs2, imm=signed PC-relative offset, ±512KB reach)

**U — one register + large immediate** `| 6: opcode | 4: rd | 22: imm |`
LDI16/LDI32L/LDI32H (16-bit imm, upper bits unused), BAL (22-bit signed PC-relative offset, ±8MB reach)

**N — no operands** `| 6: opcode | 26: — |`
HLT, NOP

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

Opcodes 29–63 are reserved.

## Pseudo-instructions (assembler-expanded)

- `CALL`             — push link register to stack, BAL to target
- `RET`              — pop return address from stack into PC
- `JMP`              — unconditional jump to label

