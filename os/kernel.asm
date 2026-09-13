; kernel.asm

.define SPINIT 0x1FFC

.define MAGIC 0xB2155F2D

.define TXADDR   0xFFFF0000
.define RXADDR   0xFFFF0004
.define DISKSECT 0xFFFF0010
.define DISKBUF  0xFFFF0014
.define DISKCMD  0xFFFF0018

.define KERNELBUF 0x1000

.define PTABLE 0x1200
.define SLOTSZ 0x100

.define SLOT_STATUS 0
.define SLOT_PC     4
.define SLOT_SP     8
.define SLOT_REGS   12

.define PROC_UNUSED   0
.define PROC_RUNNABLE 1
.define PROC_RUNNING  2

.define CURPID 0x1600

.define USERSTART 0x2000
.define PROCBASE  0x2000              ; base of process 0 region (origin for pid arithmetic)
.define PROCSZ    0x2000              ; size of each process region

kernel_start:
  li r0, trap_handler                 ; install trap handler
  mtcr tvec, r0

  li sp, SPINIT                       ; initialise stack pointer

load_superblock:
  li r0, DISKSECT                     ; load superblock
  li r1, 0                            ; sector 0 (superblock)
  str32 r1, [r0]

  li r0, DISKBUF                      ; disk buffer = KERNELBUF
  li r2, KERNELBUF
  str16 r2, [r0]

  li r0, DISKCMD                      ; trigger load
  li r1, 0
  str8 r1, [r0]

  ldm32 r1, [r2]                      ; r1 = magic number

  li r2, MAGIC
  bne r1, r2, err_invalid_superblock

load_root_inode:
  li r0, DISKSECT                     ; load inode table
  li r1, 1                            ; sector 1
  str32 r1, [r0]

  li r0, DISKBUF                      ; disk buffer = KERNELBUF
  li r2, KERNELBUF
  str16 r2, [r0]

  li r0, DISKCMD                      ; trigger load
  li r1, 0
  str8 r1, [r0]

  ldm32 r1, [r2 + 72]                 ; r1 = block_pointers[0] of inode 1

find_file:
  li r0, DISKSECT                     ; load root dir data block
  str32 r1, [r0]

  li r0, DISKBUF                      ; disk buffer = KERNELBUF
  li r2, KERNELBUF
  str16 r2, [r0]

  li r0, DISKCMD                      ; trigger load
  li r1, 0
  str8 r1, [r0]

  li r0, KERNELBUF                    ; r0 = entry_ptr
  li r1, 18                           ; r1 = remaining
  ldi16 r2, 0                         ; r2 = const 0

  scan_loop:
    beq r1, r2, err_file_not_found    ; if remaining == 0 -> file not found

    ldm32 r3, [r0 + 24]               ; load inode number (entry_ptr + 24)
    beq r3, r2, err_file_not_found    ; if inode == 0 -> file not found (empty slot)

    mov r4, r0                        ; r4 = pointer into direntry
    li r5, target                     ; r5 = pointer into target

    cmp_loop:
      ldm8 r6, [r4]                   ; r6 = direntry[r4]
      ldm8 r7, [r5]                   ; r7 = target[r5]
      bne r6, r7, no_match            ; if r6 != r7 -> no_match
      beq r6, r2, found               ; if both equal and null -> found

      li r2, 1
      add r4, r4, r2                  ; r4 += 1
      add r5, r5, r2                  ; r5 += 1
      li r2, 0

      bal r14, cmp_loop

    no_match:
      ldi16 r2, 28                    ; entry_ptr += 28
      add r0, r0, r2

      ldi16 r2, 1                     ; remaining -= 1
      sub r1, r1, r2
      ldi16 r2, 0

      bal r14, scan_loop              ; jump back to scan_loop

    found:
      bal r14, load_file_inode        ; found a match, proceed (r3=inode)

target:
  .str "init.bin"

load_file_inode:
  li r0, DISKSECT                     ; load inode table
  mov r1, r3                          ; r1 = inode_block = (1 + N / 8)
  ldi16  r2, 3
  slr  r1, r1, r2
  ldi16 r2, 1
  add r1, r1, r2
  str32 r1, [r0]

  li r0, DISKBUF                      ; disk buffer = KERNELBUF
  li r2, KERNELBUF
  str16 r2, [r0]

  li r0, DISKCMD                      ; trigger load
  li r1, 0
  str8 r1, [r0]

  mov r1, r3                          ; r1 = byte_offset = (N & 7) * 64
  li r0, 7
  and r1, r1, r0
  li r0, 64
  mul r1, r1, r0
  add r2, r2, r1                      ; r2 = pointer to inode
  li r0, 8
  add r3, r2, r0                      ; r3 = pointer to block pointers

load_file_data:
  li r4, USERSTART                    ; r4 = current load address
  ldi16 r5, 0                         ; r5 = block counter
  ldi16 r6, 8                         ; r6 = max blocks
  ldi16 r7, 0                         ; r7 = const 0

  block_loop:
    beq r5, r6, jump_to_user          ; all 8 slots checked
    ldm32 r1, [r3]                    ; r1 = block number
    beq r1, r7, jump_to_user          ; null pointer = end of file

    li r0, DISKSECT                   ; disk_sector = r1
    str32 r1, [r0]

    li r0, DISKBUF                    ; disk_buffer = r4
    str16 r4, [r0]

    li r0, DISKCMD                    ; trigger load
    str8 r7, [r0]

    ldi16 r2, 4
    add r3, r3, r2                    ; advance to next block pointer slot
    ldi16 r2, 512
    add r4, r4, r2                    ; advance load address by one block
    ldi16 r2, 1
    add r5, r5, r2                    ; increment block counter

    bal r14, block_loop

jump_to_user:
  li r1, PTABLE                       ; set up process table for pid 0

  ldi16 r0, 0
  str8 r0, [r1 + SLOT_STATUS]         ; SLOT_STATUS = 0

  ldi16 r0, USERSTART
  str32 r0, [r1 + SLOT_PC]            ; SLOT_PC = USERSTART (0x2000)

  li r0, PROCBASE
  li r2, PROCSZ
  add r0, r0, r2
  ldi16 r2, 4
  sub r0, r0, r2
  str32 r0, [r1 + SLOT_SP]            ; SLOT_SP = PROCBASE + PROCSZ - 4 (0x3FFC)
  mov sp, r0                          ; SP = 0x3FFC

  li r1, CURPID
  ldi16 r0, 0
  str32 r0, [r1]                      ; CURPID = 0
  
  li r0, USERSTART
  mtcr epc, r0
  eret

scheduler:
  ; find the next runnable slot
  li r0, CURPID
  ldm32 r3, [r0]                      ; r3 = current pid
  li r2, 0                            ; counter

  sc_loop:
    li r1, 1
    add r3, r3, r1
    li r1, 3
    and r3, r3, r1                    ; (CURPID + 1) % 4
    li r1, 8
    sll r0, r3, r1
    li r1, PTABLE
    add r0, r0, r1

    ldm32 r1, [r0 + SLOT_STATUS]
    li r4, PROC_RUNNABLE
    beq r1, r4, sc_run

    li r0, 1
    add r2, r2, r0                    ; r2 += 1
    li r0, 3
    beq r2, r0, sc_hlt                ; r2 == 3 -> sc_hlt
    jmp sc_loop

  sc_run:
    ; slot base address in r0 
    ; pid in r3
    li r1, CURPID
    str32 r3, [r1]

    li r1, PROC_RUNNING
    str32 r1, [r0 + SLOT_STATUS]

    ldm32 r1, [r0 + SLOT_PC]
    mtcr epc, r1

    ldm32 sp, [r0 + SLOT_SP]

    li r1, SLOT_REGS
    add r0, r0, r1

    ldm32  r1, [r0 +  4]
    ldm32  r2, [r0 +  8]
    ldm32  r3, [r0 + 12]
    ldm32  r4, [r0 + 16]
    ldm32  r5, [r0 + 20]
    ldm32  r6, [r0 + 24]
    ldm32  r7, [r0 + 28]
    ldm32  r8, [r0 + 32]
    ldm32  r9, [r0 + 36]
    ldm32 r10, [r0 + 40]
    ldm32 r11, [r0 + 44]
    ldm32 r12, [r0 + 48]
    ldm32 r14, [r0 + 52]
    ldm32  r0, [r0 +  0]

    eret

  sc_hlt:
    hlt

err_invalid_superblock:
  li r0, 1                            ; r0 = 1 (err: invalid superblock)
  hlt

err_file_not_found:
  li r0, 2                            ; r0 = 2 (err: file not found)
  hlt

err_unknown_trap:
  li r0, 3                            ; r0 = 3 (err: unknown trap)
  hlt

trap_handler:
  mtcr scratch, r0                    ; save r0
  ldi16 r0, 56                        ; frame is 14 regs * 4 bytes
  sub sp, sp, r0                      ; grow stack
  
  str32  r1, [sp +  4]                ; save r1-14
  str32  r2, [sp +  8]
  str32  r3, [sp + 12]
  str32  r4, [sp + 16]
  str32  r5, [sp + 20]
  str32  r6, [sp + 24]
  str32  r7, [sp + 28]
  str32  r8, [sp + 32]
  str32  r9, [sp + 36]
  str32 r10, [sp + 40]
  str32 r11, [sp + 44]
  str32 r12, [sp + 48]
  str32 r14, [sp + 52]

  mfcr r0, scratch                    ; recover r0 (syscall number)
  str32 r0, [sp]

  mfcr r3, cause
  li r4, 0
  beq r3, r4, syscall_handler
  jmp err_unknown_trap

trap_exit:
  str32 r0, [sp + 0]                  ; write return value into frame
  ldi16 r0, 56
  add r0, sp, r0                      ; r0 = future SP (current SP + 56)
  mtcr scratch, r0                    ; stash

  ldm32  r1, [sp +  4]                ; restore r1-r14
  ldm32  r2, [sp +  8]
  ldm32  r3, [sp + 12]
  ldm32  r4, [sp + 16]
  ldm32  r5, [sp + 20]
  ldm32  r6, [sp + 24]
  ldm32  r7, [sp + 28]
  ldm32  r8, [sp + 32]
  ldm32  r9, [sp + 36]
  ldm32 r10, [sp + 40]
  ldm32 r11, [sp + 44]
  ldm32 r12, [sp + 48]
  ldm32 r14, [sp + 52]

  ldm32 r0, [sp]                      ; restore return value into r0
  mfcr sp, scratch

  eret

syscall_handler:
  li r3, 0
  beq r0, r3, sys_exit

  li r3, 1
  beq r0, r3, sys_write

  li r3, 2
  beq r0, r3, sys_read

  li r3, 4
  beq r0, r3, sys_yield

  jmp sys_unknown

sys_unknown:
  li r0, -1                        ; return -1
  bal lr, trap_exit

sys_exit:
  li r0, CURPID
  ldm32 r0, [r0]
  ldi16 r1, 8
  sll r0, r0, r1
  li r1, PTABLE
  add r0, r0, r1                   ; R0 = SLOT PTR = PTABLE + CURPID << 8

  ldi16 r1, PROC_UNUSED
  str32 r1, [r0 + SLOT_STATUS]

  jmp scheduler

sys_write:
  li r3, TXADDR
  ldi16 r4, 0
  ldi16 r5, 1
  beq r4, r2, sys_write_done

  sys_write_loop:
    ldm8 r6, [r1]                     ; load next byte
    str8 r6, [r3]                     ; write to TX
    add r1, r1, r5                    ; advance pointer
    add r4, r4, r5                    ; inc counter
    bne r4, r2, sys_write_loop        ; loop if counter != byte count

  sys_write_done:
    mov r0, r4                        ; return bytes written
    bal lr, trap_exit

sys_read:
  li r1, RXADDR
  ldm8 r0, [r1]                       ; return byte from RX
  bal lr, trap_exit

sys_yield:
  li r0, CURPID
  ldm32 r0, [r0]
  ldi16 r1, 8
  sll r0, r0, r1
  li r1, PTABLE
  add r0, r0, r1                      ; R0 = SLOT PTR = PTABLE + CURPID << 8

  ldi16 r1, PROC_RUNNABLE
  str32 r1, [r0 + SLOT_STATUS]

  mfcr r1, epc
  str32 r1, [r0 + SLOT_PC]

  li r1, 56
  add r1, r1, sp
  str32 r1, [r0 + SLOT_SP]

  li r1, SLOT_REGS
  add r0, r0, r1

  li r1, 0                            ; 0 return code
  str32 r1, [r0 +  0]
  ldm32 r1, [sp +  4]
  str32 r1, [r0 +  4]
  ldm32 r1, [sp +  8]
  str32 r1, [r0 +  8]
  ldm32 r1, [sp + 12]
  str32 r1, [r0 + 12]
  ldm32 r1, [sp + 16]
  str32 r1, [r0 + 16]
  ldm32 r1, [sp + 20]
  str32 r1, [r0 + 20]
  ldm32 r1, [sp + 24]
  str32 r1, [r0 + 24]
  ldm32 r1, [sp + 28]
  str32 r1, [r0 + 28]
  ldm32 r1, [sp + 32]
  str32 r1, [r0 + 32]
  ldm32 r1, [sp + 36]
  str32 r1, [r0 + 36]
  ldm32 r1, [sp + 40]
  str32 r1, [r0 + 40]
  ldm32 r1, [sp + 44]
  str32 r1, [r0 + 44]
  ldm32 r1, [sp + 48]
  str32 r1, [r0 + 48]
  ldm32 r1, [sp + 52]
  str32 r1, [r0 + 52]

  jmp scheduler
