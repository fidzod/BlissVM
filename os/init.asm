; init.asm
; start ikari shell
jmp start

.include "stdlib.asm"

start:
  li r1, init_msg                                ; stdlib_println(init_msg)
  call stdlib_println

  li r1, shell_path                              ; exec("ikari.bin")
  ldi16 r0, 3
  ecall

end:
  hlt

init_msg: .str "Init: Running ikari.bin"
shell_path: .str "ikari.bin"
