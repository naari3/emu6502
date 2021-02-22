# emu6502

```rust
fn main() {
    let mut cpu = CPU::default();
    let mut ram = RAM::default();
    cpu.reset(&mut ram);
    ram.write_rom(
        0x8000,
        &[
            // https://gist.github.com/pedrofranceschi/1285964
            0xA2, 0x01, //     LDX #$01; x = 1
            0x86, 0x00, //     STX $00; stores x
            //
            0x38, //           SEC; clean carry;
            0xA0, 0x07, //     LDY #$07; calculates 7th fibonacci number (13 = D in hex)
            0x98, //           TYA; transfer y register to accumulator
            0xE9, 0x03, //     SBC #$03; handles the algorithm iteration counting
            0xA8, //           TAY; transfer the accumulator to the y register
            //
            0x18, //           CLC; clean carry
            0xA9, 0x02, //     LDA #$02; a = 2
            0x85, 0x01, //     STA $01; stores a
            //
            //             loop:
            0xA6, 0x01, //     LDX $01; x = a
            0x65, 0x00, //     ADC $00; a += x
            0x85, 0x01, //     STA $01; stores a
            0x86, 0x00, //     STX $00; stores x
            0x88, //           DEY; y -= 1
            0xD0, -11_i8 as u8, //  BNE loop; jumps back to loop if Z bit != 0
        ],
    );

    ram[0xFFFC] = 0x00;
    ram[0xFFFD] = 0x80;

    let cycles = 82;
    cpu.execute(cycles, &mut ram);
    println("cpu.y: {}", cpu.y); // #=> should be 13
}
```