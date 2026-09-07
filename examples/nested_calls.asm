; nested_calls.asm

main:
  li sp, 0x1FFC         ; initialise stack pointer
  call foo
  hlt

foo:
  push lr

  li r1, in_foo
  call print

  call bar

  li r1, out_foo
  call print

  pop lr
  ret

bar:
  push lr

  li r1, msg
  call print

  pop lr
  ret

print:
  li r0, 0xFFFF0000     ; r0 = TX address
  ldi16 r2, 0           ; r2 = zero (null terminator sentinel)

  loop:
    ldm8 r3, [r1]       ; load next byte
    beq r3, r2, done    ; if null, stop
    str8 r3, [r0]       ; write to TX
    ldi16 r4, 1
    add r1, r1, r4      ; advance pointer
    beq r2, r2, loop    ; unconditional branch (0 == 0 always)

  done:
    ret

in_foo:
  .str "in foo\n"

out_foo:
  .str "out foo\n"

msg:
  .str "Hello, world!\n"
