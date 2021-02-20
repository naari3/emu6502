mod cpu;
mod instruction;
mod ram;

use cpu::CPU;
use ram::RAM;

fn main() {
    let mut cpu = CPU::default();
    let mut ram = RAM::default();
    cpu.reset(&mut ram);
    ram[0x8000] = 0xA2; // LDX #$02
    ram[0x8001] = 0x2;
    ram[0x8002] = 0xB5; // LDA $40,x => should be $42
    ram[0x8003] = 0x40;
    ram[0x8004] = 0xEA; // NOP
    ram[0x8005] = 0xEA; // NOP
    ram[0x8006] = 0xEA; // NOP
    ram[0x8007] = 0x85; // STA $43
    ram[0x8008] = 0x43;
    ram[0x8009] = 0xAC; // LDY $FFFD
    ram[0x800A] = 0xFD;
    ram[0x800B] = 0xFF;

    ram[0xFFFC] = 0x00;
    ram[0xFFFD] = 0x80;

    ram[0x42] = 0x84;
    cpu.execute(21, &mut ram);
    println!("CPU: {:?}", cpu);
    println!("RAM: {:?}", ram[0x43]);
    println!("AAA: {:?}", 0x10_u8.overflowing_sub(0x11_u8));
}
