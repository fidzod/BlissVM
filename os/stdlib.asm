; stdlib.asm
; readline

stdlib_readline:
  ; * stdlib_readline
  ; arguments:
  ;   r1 = pointer to a caller-supplied buffer
  ;   r2 = buffer capacity (max bytes to read)
  ; returns:
  ;   r0 = number of bytes read (not including newline)
  mov r3, r1                      ; r3 = pointer
  mov r4, r1                      ; r4 = counter
  mov r5, r2                      ; r5 = buffer capacity

  stdrl_read_char:
    ldi16 r0, 2                   ; r0 = 2 (read)
    ecall
    mov r6, r0

    ldi16 r0, 10
    beq r6, r0, stdrl_return

    str8 r6, [r4]

    ldi16 r0, 1                   ; r0 = 1 (write)
    mov r1, r4                    ; r1 = counter
    ldi16 r2, 1                   ; r2 = byte_count (1)
    ecall

    ldi16 r0, 1
    add r4, r4, r0                ; counter += 1

    mov r0, r4
    sub r0, r0, r3
    beq r0, r5, stdrl_return      ; bytes read == capacity -> return

    jmp stdrl_read_char

  stdrl_return:
    mov r0, r4                    ; calculate bytes read
    sub r0, r0, r3
    ret

stdlib_strcmp:
  hlt                             ; todo

stdlib_len:
  hlt                             ; todo

stdlib_int_to_str:
  hlt                             ; todo

stdlib_str_to_int:
  hlt                             ; todo
