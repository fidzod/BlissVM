; stdlib.asm
; divmod, memcpy, print, println, readline, strcmp, strlen, str_from_int, str_to_int

stdlib_divmod:
  ; * stdlib_divmod
  ; arguments:
  ;   r1 = dividend
  ;   r2 = divisor
  ;   r3 = output pointer (two adjacent words)
  ; returns:
  ;   [r3] = quotient, [r3 + 4] = remainder
  ;   r0 = 0 on success, 1 for divide by zero
  ldi16 r4, 0                        ; r4 = counter

  beq r2, r4, stdlib_divmod_eret     ; check for div_by_zero

stdlib_divmod_loop:
  blt r1, r2, stdlib_divmod_ret      ; stop if if r1 < r2

  sub r1, r1, r2                     ; r1 -= r2 

  ldi16 r0, 1                        ; counter++
  add r4, r4, r0

  jmp stdlib_divmod_loop

stdlib_divmod_eret:
  ldi16 r0, 1
  ret

stdlib_divmod_ret:
  str32 r4, [r3]                     ; [r3] = quotient

  ldi16 r0, 4                        ; [r3 + 4] = remainder
  add r3, r3, r0
  str32 r1, [r3]

  ldi16 r0, 0                        ; ret 0
  ret

stdlib_memcpy:
  ; * stdlib_memcpy
  ; arguments:
  ;   r1 = source pointer
  ;   r2 = destination pointer
  ;   r3 = byte count
  ; returns:
  ;   (none)
  ldi16 r5, 0                        ; r5 = zero sentinel

stdlib_memcpy_loop:
  beq r3, r5, stdlib_memcpy_ret      ; stop when count == 0

  ldm8 r4, [r1]                      ; r4 = *src
  str8 r4, [r2]                      ; *dst = r4

  ldi16 r0, 1
  add r1, r1, r0                     ; src++
  add r2, r2, r0                     ; dst++
  sub r3, r3, r0                     ; count--

  jmp stdlib_memcpy_loop

stdlib_memcpy_ret:
  ret

stdlib_print:
  ; * stdlib_print
  ; arguments:
  ;   r1 = &str (null-terminated)
  ; returns:
  ;   r0 = number of bytes written
  push lr
  push r8

  mov r8, r1                         ; save str pointer (strlen clobbers r1)
  call stdlib_strlen                 ; r0 = strlen(str)

  mov r2, r0                         ; r2 = byte count
  mov r1, r8                         ; r1 = str pointer
  ldi16 r0, 1                        ; sys_write
  ecall

  pop r8
  pop lr
  ret

.define STDLIB_PRINTLN_BUFF_SZ 128
stdlib_println:
  ; * stdlib_println
  ; arguments:
  ;  r1 = &str (max len: 128 bytes)
  ; returns:
  ;  r0 = 0: success, 1: err(exceeds_max_len)
  push lr
  push r8
  push r9

  mov r8, r1                         ; save r1 (gets clobbered by strlen)

  call stdlib_strlen                 ; r0 = strlen(*r1)
  mov r9, r0                         ; save in r9

  mov r1, r9                         ; check for err(exceeds_max_len)
  ldi16 r0, 1
  add r1, r1, r0
  ldi16 r0, STDLIB_PRINTLN_BUFF_SZ
  blt r0, r1, stdlib_println_err_1   ; if len + 1 > 128 err

  mov r1, r8                         ; r1 = r8 (src pointer)
  li r2, stdlib_println_buff         ; r2 = &stdlib_println_buff (dest pointer)
  mov r3, r9                         ; r3 = strlen(*r1)
  call stdlib_memcpy                 ; copy string into buffer

  li r1, stdlib_println_buff
  mov r0, r9
  add r1, r1, r0
  ldi16 r0, 10                       ; '\n'
  str8 r0, [r1]                      ; buff[strlen(*r1)] = '\n'

  li r1, stdlib_println_buff
  ldi16 r0, 1                        ; sys_write
  mov r2, r9
  add r2, r2, r0                     ; add 1 to len to account for '\n'
  ecall

  pop r9
  pop r8
  pop lr
  ldi16 r0, 0
  ret

stdlib_println_err_1:
  pop r9
  pop r8
  pop lr
  ldi16 r0, 1
  ret

stdlib_println_buff: .space 128

stdlib_readline:
  ; * stdlib_readline
  ; arguments:
  ;   r1 = pointer to a caller-supplied buffer
  ;   r2 = buffer capacity (max bytes to read)
  ; returns:
  ;   r0 = number of bytes read (not including newline)
  mov r3, r1                         ; r3 = pointer
  mov r4, r1                         ; r4 = counter
  mov r5, r2                         ; r5 = buffer capacity

stdlib_readline_getch:
  ldi16 r0, 2                        ; r0 = 2 (read)
  ecall
  mov r6, r0

  ldi16 r0, 10
  beq r6, r0, stdlib_readline_ret

  str8 r6, [r4]

  ldi16 r0, 1                        ; r0 = 1 (write)
  mov r1, r4                         ; r1 = counter
  ldi16 r2, 1                        ; r2 = byte_count (1)
  ecall

  ldi16 r0, 1
  add r4, r4, r0                     ; counter += 1

  mov r0, r4
  sub r0, r0, r3
  beq r0, r5, stdlib_readline_ret    ; bytes read == capacity -> return

  jmp stdlib_readline_getch

stdlib_readline_ret:
  mov r0, r4                         ; calculate bytes read
  sub r0, r0, r3
  ret

stdlib_strcmp:
  ; * stdlib_strcmp
  ; arguments:
  ;   r1, r2 = string pointers 
  ; returns:
  ;   r0 = 1 equal, 0 not equal
  ldm8 r3, [r1]
  ldm8 r4, [r2]

  bne r3, r4, stdlib_strcmp_ne
  ldi16 r0, 0
  beq r3, r0, stdlib_strcmp_eq

  ldi16 r0, 1
  add r1, r1, r0
  add r2, r2, r0
  jmp stdlib_strcmp

stdlib_strcmp_eq:
  ldi16 r0, 1                        ; return true
  ret

stdlib_strcmp_ne:
  ldi16 r0, 0                        ; return false
  ret

stdlib_strlen:
  ; * stdlib_strlen
  ; arguments:
  ;   r1 = string pointer
  ; returns:
  ;   r0 = string length
  ldi16 r3, 0                        ; r3 = null sentinel
  ldi16 r4, 0                        ; r4 = counter

stdlib_strlen_loop:
  ldm8 r2, [r1]                      ; r2 = *r1
  beq r2, r3, stdlib_strlen_ret
  ldi16 r0, 1
  add r4, r4, r0                     ; counter++
  add r1, r1, r0                     ; r1++
  jmp stdlib_strlen_loop

stdlib_strlen_ret:
  mov r0, r4
  ret

stdlib_str_from_int:
  ; * stdlib_str_from_int
  ; arguments:
  ;   r1 = integer
  ;   r2 = output buffer pointer
  ; returns:
  ;   [r2...] = str(r1)
  ;   r0 = string length
  push lr
  push r8
  mov r5, r1                         ; r5 = n
  mov r6, r2                         ; r6 = buffer pointer
  ldi16 r7, 0                        ; r7 = counter

  ldi16 r0, 0
  beq r5, r0, stdlib_str_from_int_zro

stdlib_str_from_int_parse:
  ldi16 r0, 0
  beq r5, r0, stdlib_str_from_int_wr

  mov r1, r5
  ldi16 r2, 10
  li r3, stdlib_str_from_int_buf
  call stdlib_divmod                 ; divmod(n, 10)

  ldm32 r5, [r3]                     ; r5 = remainder (r3 = buf+4 after divmod)
  ldi16 r0, 48
  add r5, r5, r0                     ; r5 = remainder + '0' (digit char)
  push r5

  ldi16 r0, 4
  sub r3, r3, r0                     ; r3 = buf
  ldm32 r5, [r3]                     ; r5 = quotient

  ldi16 r0, 1                        ; counter++
  add r7, r7, r0

  jmp stdlib_str_from_int_parse

stdlib_str_from_int_wr:
  mov r8, r7

stdlib_str_from_int_loop:
  pop r5
  str8 r5, [r6]
  ldi16 r0, 1
  add r6, r6, r0                     ; buffer_pointer++

  ldi16 r0, 1
  sub r8, r8, r0                     ; counter--

  ldi16 r0, 0
  beq r8, r0, stdlib_str_from_int_end

  jmp stdlib_str_from_int_loop

stdlib_str_from_int_end:
  ldi16 r0, 0
  str8 r0, [r6]
  mov r0, r7
  beq r0, r0, stdlib_str_from_int_ret

stdlib_str_from_int_zro:
  ldi16 r0, 48
  str8 r0, [r6]
  ldi16 r0, 0
  str8 r0, [r6 + 1]
  ldi16 r7, 1                        ; r7 = 1 (length), survives the pops below

stdlib_str_from_int_ret:
  pop r8
  pop lr
  mov r0, r7                         ; restore return value after pops clobber r0
  ret

stdlib_str_from_int_buf: .space 8    ; reserve 2 words for divmod

stdlib_str_to_int:
  ; * stdlib_str_to_int
  ; arguments:
  ;   r1 = string pointer
  ; returns:
  ;   r0 = int (stops at first non-digit; ie. '42abc' = 42)
  ldi16 r3, 0                        ; r3 = accumulator

stdlib_str_to_int_loop:
  ldm8 r2, [r1]                      ; r2 = *r1

  ldi16 r0, 0
  beq r0, r2, stdlib_str_to_int_ret  ; stop if r2 == '\0'

  ldi16 r0, 48
  sub r2, r2, r0                     ; r2 -= '0' (48)
  ldi16 r0, 9
  blt r0, r2, stdlib_str_to_int_ret  ; stop if r2 > 9 (not a digit)

  ldi16 r0, 10                       ; r3 (accumulator) = r3 * 10 + r2 (digit)
  mul r3, r3, r0
  add r3, r3, r2

  ldi16 r0, 1                        ; r1++
  add r1, r1, r0

  jmp stdlib_str_to_int_loop

stdlib_str_to_int_ret:
  mov r0, r3                         ; r0 = r3 (accumulator)
  ret

