; echo.asm
; echo program using read and write syscalls
jmp start

.include "stdlib.asm"

.define ECHOBUFSZ 200

start:
  li r8, echobuf

  read:
    mov r1, r8
    ldi16 r2, ECHOBUFSZ
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
  li r1, newline                  ; r1 = pointer to string
  ldi16 r0, 1                     ; r0 = 1 (write)
  ldi16 r2, 1                     ; r2 = byte count (1)
  ecall
  ret

newline: .str "\n"
echobuf: .space ECHOBUFSZ
