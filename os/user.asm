; user.asm
; echo program using read and write syscalls
.org 0x2000

start:
  ldi16 r8, 0x2800                ; echo_buffer at 0x2800 - 0x28c8 (200 bytes)
  ldi16 r9, 10                    ; r9 = \n

  ldi16 r0, 0x28c8
  str8 r9, [r0]                   ; memory[0x28c8] = \n

  read_line:
    mov r10, r8                   ; r10 = count (pointer into echo_buffer)

    read_char:
      ldi16 r0, 2                 ; r0 = 2 (read, result in r0)
      ecall
      mov r11, r0                 ; r0 -> r11

      beq r11, r9, print          ; r11 == \n -> print

      str8 r11, [r10]             ; write r11 into echo_buffer

      ldi16 r0, 1                 ; r0 = 1 (write)
      mov r1, r10                 ; r1 = r10 (pointer to current char)
      ldi16 r2, 1                    ; r2 = byte count (1)
      ecall

      ldi16 r0, 1
      add r10, r10, r0            ; r10 += 1

      jmp read_char

print:
  call print_newline

  ldi16 r0, 1                     ; r0 = 1 (write)
  mov r1, r8                      ; r1 = r8 (pointer to string)

  mov r2, r10                     ; r2 = bytecount (r10 - r8)
  mov r3, r8
  sub r2, r2, r3

  ecall

  call print_newline

  jmp read_line

print_newline:
  ldi16 r0, 1                     ; r0 = 1 (write)
  ldi16 r1, 0x28c8                ; print a newline (assume newline at 0x28c8)
  ldi16 r2, 1
  ecall
  ret
