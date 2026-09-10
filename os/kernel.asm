; kernel.asm

kernel_start:
  li r0, trap_handler               ; install trap handler
  mtcr tvec, r0

  li sp, 0x1FFC                     ; initialise stack pointer

load_superblock:
  li r0, 0xFFFF0010                 ; load superblock
  li r1, 0                          ; sector 0 (superblock)
  str32 r1, [r0]

  li r0, 0xFFFF0014                 ; disk buffer = 0x1000
  li r2, 0x1000
  str16 r2, [r0]

  li r0, 0xFFFF0018                 ; trigger load
  li r1, 0
  str8 r1, [r0]

  ldm32 r1, [r2]                    ; r1 = magic number

  li r2, 0xB2155F2D
  bne r1, r2, err_invalid_superblock

load_root_inode:
  li r0, 0xFFFF0010                 ; load inode table
  li r1, 1                          ; sector 1
  str32 r1, [r0]

  li r0, 0xFFFF0014                 ; disk buffer = 0x1000
  li r2, 0x1000
  str16 r2, [r0]

  li r0, 0xFFFF0018                 ; trigger load
  li r1, 0
  str8 r1, [r0]

  ldm32 r1, [r2 + 72]               ; r1 = block_pointers[0] of inode 1

find_file:
  li r0, 0xFFFF0010                 ; load root dir data block
  str32 r1, [r0]

  li r0, 0xFFFF0014                 ; disk buffer = 0x1000
  li r2, 0x1000
  str16 r2, [r0]

  li r0, 0xFFFF0018                 ; trigger load
  li r1, 0
  str8 r1, [r0]

  li r0, 0x1000                     ; r0 = entry_ptr
  li r1, 18                         ; r1 = remaining
  ldi16 r2, 0                       ; r2 = const 0

  scan_loop:
    beq r1, r2, err_file_not_found  ; if remaining == 0 -> file not found

    ldm32 r3, [r0 + 24]             ; load inode number (entry_ptr + 24)
    beq r3, r2, err_file_not_found  ; if inode == 0 -> file not found (empty slot)

    mov r4, r0                      ; r4 = pointer into direntry
    li r5, target                   ; r5 = pointer into target

    cmp_loop:
      ldm8 r6, [r4]                 ; r6 = direntry[r4]
      ldm8 r7, [r5]                 ; r7 = target[r5]
      bne r6, r7, no_match          ; if r6 != r7 -> no_match
      beq r6, r2, found             ; if both equal and null -> found

      li r2, 1
      add r4, r4, r2                 ; r4 += 1
      add r5, r5, r2                 ; r5 += 1
      li r2, 0

      bal r14, cmp_loop

    no_match:
      ldi16 r2, 28                  ; entry_ptr += 28
      add r0, r0, r2

      ldi16 r2, 1                   ; remaining -= 1
      sub r1, r1, r2
      ldi16 r2, 0

      bal r14, scan_loop            ; jump back to scan_loop

    found:
      bal r14, load_file_inode      ; found a match, proceed (r3=inode)

target:
  .str "user.bin"

load_file_inode:
  li r0, 0xFFFF0010                 ; load inode table
  mov r1, r3                        ; r1 = inode_block = (1 + N / 8)
  ldi16  r2, 3
  slr  r1, r1, r2
  ldi16 r2, 1
  add r1, r1, r2
  str32 r1, [r0]

  li r0, 0xFFFF0014                 ; disk buffer = 0x1000
  li r2, 0x1000
  str16 r2, [r0]

  li r0, 0xFFFF0018                 ; trigger load
  li r1, 0
  str8 r1, [r0]

  mov r1, r3                        ; r1 = byte_offset = (N & 7) * 64
  li r0, 7
  and r1, r1, r0
  li r0, 64
  mul r1, r1, r0
  add r2, r2, r1                    ; r2 = pointer to inode
  li r0, 8
  add r3, r2, r0                    ; r3 = pointer to block pointers

load_file_data:
  li r4, 0x2000                     ; r4 = current load address
  ldi16 r5, 0                       ; r5 = block counter
  ldi16 r6, 8                       ; r6 = max blocks
  ldi16 r7, 0                       ; r7 = const 0

  block_loop:
    beq r5, r6, jump_to_user        ; all 8 slots checked
    ldm32 r1, [r3]                  ; r1 = block number
    beq r1, r7, jump_to_user        ; null pointer = end of file

    li r0, 0xFFFF0010               ; disk_sector = r1
    str32 r1, [r0]

    li r0, 0xFFFF0014               ; disk_buffer = r4
    str16 r4, [r0]

    li r0, 0xFFFF0018               ; trigger load
    str8 r7, [r0]

    ldi16 r2, 4
    add r3, r3, r2                  ; advance to next block pointer slot
    ldi16 r2, 512
    add r4, r4, r2                  ; advance load address by one block
    ldi16 r2, 1
    add r5, r5, r2                  ; increment block counter

    bal r14, block_loop

jump_to_user:
  li r0, 0x2000
  mtcr epc, r0
  eret

err_invalid_superblock:
  li r0, 1                          ; r0 = 1 (err: invalid superblock)
  hlt

err_file_not_found:
  li r0, 2                          ; r0 = 2 (err: file not found)
  hlt

err_unknown_trap:
  li r0, 3                          ; r0 = 3 (err: unknown trap)
  hlt

trap_handler:
  mfcr r3, cause
  li r4, 0
  beq r3, r4, syscall_handler
  jmp err_unknown_trap

syscall_handler:
  li r3, 1
  beq r0, r3, sys_write

  li r3, 2
  beq r0, r3, sys_read

  jmp sys_unknown

sys_unknown:
  ldi16 r0, -1                      ; return -1
  eret

sys_write:
  li r3, 0xFFFF0000
  ldi16 r4, 0
  ldi16 r5, 1

  sys_write_loop:
    ldm8 r6, [r1]                   ; load next byte
    str8 r6, [r3]                   ; write to TX
    add r1, r1, r5                  ; advance pointer
    add r4, r4, r5                  ; inc counter
    bne r4, r2, sys_write_loop      ; loop if counter != byte count

  mov r0, r4                        ; return bytes written
  eret

sys_read:
  li r1, 0xFFFF0004
  ldm8 r0, [r1]                     ; return byte from RX
  eret
