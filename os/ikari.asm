; Ikari (ikari.asm)
; A shell for BlissFieldsOS
jmp start

.include "stdlib.asm"

start:
  li r1, welcome_msg
  call stdlib_println

  jmp loop

exit:
  ldi16 r0, 0                        ; sys_exit
  ecall

.define INPUT_BUFFER_SZ 256
input_buffer: .space INPUT_BUFFER_SZ

loop:
  li r1, prompt
  call stdlib_print

  li r1, input_buffer
  li r2, INPUT_BUFFER_SZ
  call stdlib_readline

  li r1, newline
  call stdlib_print

  jmp loop

welcome_msg: .str "Welcome to Ikari, a shell for BlissFieldsOS"
prompt: .str "ikari % "
newline: .str "\n"

