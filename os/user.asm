; user.asm
.org 0x2000

start:
  ldi16 r0, 1                     ; r0 = 1 (write)
  li r1, msg                      ; r1 = pointer to string
  li r2, 22                       ; r2 = byte count
  ecall
  hlt

msg:
  .str "Hello from user mode!\n"
