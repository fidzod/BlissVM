; user.asm
; echo program using read and write syscalls
.org 0x2000
jmp start

.include "stdlib.asm"

start:
  ldi16 r8, 0x2800                ; echo_buffer at 0x2800 - 0x28c8 (200 bytes)

  read:
    mov r1, r8
    ldi16 r2, 200
    call stdlib_readline
    mov r9, r0                    ; r9 = bytes read
    jmp print

print:
  call print_newline

  ldi16 r0, 1                     ; r0 = 1 (write)
  mov r1, r8                      ; r1 = pointer to string (r8)
  mov r2, r9                      ; r2 = byte count (r9)

  ecall

  call print_newline

  jmp read

print_newline:
  ldi16 r0, 1                     ; r0 = 1 (write)
  li r1, newline                  ; r1 = pointer to string
  ldi16 r2, 1                     ; r2 = byte count (1)
  ecall
  ret

newline:
  .str "\n"

