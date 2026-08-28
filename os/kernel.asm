; kernel.asm

kernel_start:
  li r0, trap_handler             ; install trap handler
  mtcr tvec, r0

  li r0, 0xFFFF0010               ; load program
  li r1, 0                        ; disk_sector = 0
  str32 r1, [r0]

  li r0, 0xFFFF0014               ; disk_buffer = 0x2000
  li r2, 0x2000
  str16 r2, [r0]

  li r0, 0xFFFF0018               ; trigger load sector
  li r1, 0
  str8 r1, [r0]

  mtcr epc, r2                    ; drop into user mode
  eret

trap_handler:                     ; assume write syscall for now since we only
  li r3, 0xFFFF0000               ; have one kind of trap
  ldi16 r4, 0                     ; also, for now we clobber registers r3-r6
  ldi16 r5, 1

  loop:
    ldm8 r6, [r1]                 ; load next byte
    str8 r6, [r3]                 ; write to TX
    add r1, r1, r5                ; advance pointer
    add r4, r4, r5                ; inc counter
    bne r4, r2, loop              ; loop if counter != byte count

  eret
