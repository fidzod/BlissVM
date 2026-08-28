; kernel.asm

kernel_start:
  li r0, trap_handler             ; install trap handler
  mtcr tvec, r0

  li r0, user_start               ; drop to user mode
  mtcr epc, r0
  eret

user_start:
  ldi16 r0, 1                     ; r0 = 1 (write)
  li r1, msg                      ; r1 = pointer to string
  li r2, 22                       ; r2 = byte count
  ecall
  hlt

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

msg:
  .str "Hello from user mode!\n"
