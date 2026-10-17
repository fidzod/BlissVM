; init.asm
; exec echo

start:
  li r1, init_msg
  li r2, 23
  ldi16 r0, 1
  ecall

  li r1, echo
  ldi16 r0, 3
  ecall

end:
  hlt

init_msg: .str "Init: Running echo.bin\n"
echo: .str "echo.bin"
