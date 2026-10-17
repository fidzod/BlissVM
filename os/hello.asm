; hello.asm
; minimal exec test: write Hello! then exit

start:
  li r1, msg         ; PIC: r1 = &msg (uses r0 as scratch)
  ldi16 r0, 1        ; sys_write — set AFTER li so r0 is not overwritten
  ldi16 r2, 7
  ecall

  ldi16 r0, 0        ; sys_exit
  ecall

msg: .str "Hello!\n"
