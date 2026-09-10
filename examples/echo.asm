; Echo using RX and TX

start:
  li r7, 0xFFFF0004               ; r7 = RX address
  li r8, 0xFFFF0000               ; r8 = TX address
  li r9, 0x2800                   ; pointer to echo_buffer
  li r10, 0x2800
  ldi16 r1, 10                    ; r1 = \n

read_char:
  ldm8 r0, [r7]                   ; read 1 byte from RX into r0
  beq r0, r1, print               ; r0 == \n -> stop

  str8 r0, [r8]                   ; write r0 to TX

  str8 r0, [r9]                   ; write r0 into echo_buffer
  ldi16 r0, 1
  add r9, r9, r0                  ; r9 += 1

  jmp read_char

print:
  str8 r1, [r8]                   ; print \n

  loop:
    ldm8 r0, [r10]                ; load from echo buffer
    str8 r0, [r8]                 ; print char
    ldi16 r0, 1
    add r10, r10, r0              ; inc pointer to echo buffer
    bne r10, r9, loop             ; r10 != r9 -> print next char

  str8 r1, [r8]                   ; print \n

  jmp read_char
