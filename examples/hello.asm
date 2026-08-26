; Hello World using .str directive and a print loop
; To run: cargo run -p asm hello.asm && cargo run hello.bin

  li r0, 0xFFFF0000   ; r0 = TX address
  li r1, msg          ; r1 = pointer to string
  li r2, 0            ; r2 = zero (null terminator sentinel)

loop:
  ldm8 r3, [r1]       ; load next byte
  beq r3, r2, done    ; if null, stop
  str8 r3, [r0]       ; write to TX
  ldi16 r4, 1
  add r1, r1, r4      ; advance pointer
  beq r2, r2, loop    ; unconditional branch (0 == 0 always)

done:
  hlt

msg:
  .str "Hello, World!\n"
