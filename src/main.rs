use std::ops::{Index, IndexMut};

mod instruction;

use instruction::OPCODES;

// http://www.obelisk.me.uk/6502/registers.html
#[derive(Debug, Default)]
pub struct CPU {
    pc: u16, // Program Counter
    sp: u8,  // Stack Pointer

    a: u8, // Accumulator
    x: u8, // Index Register X
    y: u8, // Index Register Y

    flags: StatusFlag, // Processor Status
}

#[derive(Debug, Default)]
struct StatusFlag {
    c: bool, // Carry Flag
    z: bool, // Zero Flag
    i: bool, // Interrupt Disable
    d: bool, // Decimal Mode
    b: bool, // Break Command
    v: bool, // Overflow Flag
    n: bool, // Negative Flag
}

trait Instruct {
    fn execute(&self, cpu: &mut CPU, cycles: isize, ram: &mut RAM);
}

impl CPU {
    pub fn reset(&mut self, ram: &mut RAM) {
        self.pc = 0xFFFC;
        self.sp = 0xFF;
        self.flags.c = false;
        self.flags.z = false;
        self.flags.i = false;
        self.flags.d = false;
        self.flags.b = false;
        self.flags.v = false;
        self.flags.n = false;
        self.a = 0;
        self.x = 0;
        self.y = 0;

        ram.initialize();
    }

    fn fetch_byte(&mut self, cycles: &mut isize, ram: &mut RAM) -> u8 {
        let byte = ram.read_byte(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        *cycles -= 1;
        byte
    }

    fn read_byte(&mut self, cycles: &mut isize, ram: &mut RAM, addr: usize) -> u8 {
        let byte = ram.read_byte(addr);
        *cycles -= 1;
        byte
    }

    fn write_byte(&mut self, cycles: &mut isize, ram: &mut RAM, addr: usize, byte: u8) {
        ram.write_byte(addr, byte);
        *cycles -= 1;
    }

    pub fn execute(&mut self, mut cycles: isize, ram: &mut RAM) {
        let addr_low = self.fetch_byte(&mut cycles, ram);
        let addr_high = self.fetch_byte(&mut cycles, ram);
        self.pc = ((addr_high as u16) << 8) + (addr_low as u16);
        while cycles > 0 {
            let op = self.fetch_byte(&mut cycles, ram) as usize;
            if let Some(op) = &OPCODES[op] {
                op.execute(self, &mut cycles, ram);
            } else {
                panic!("{:#01x} is not implemented!", op);
            }
        }
    }
}

const MAX_MEMORY: usize = 0x100 * 0x100;
pub struct RAM {
    inner: [u8; MAX_MEMORY],
}

impl Index<usize> for RAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl Default for RAM {
    fn default() -> Self {
        RAM {
            inner: [0; MAX_MEMORY],
        }
    }
}

impl RAM {
    fn initialize(&mut self) {
        self.inner = [0; MAX_MEMORY];
    }

    fn read_byte(&mut self, address: usize) -> u8 {
        self.inner[address]
    }

    fn write_byte(&mut self, address: usize, byte: u8) {
        self.inner[address] = byte;
    }
}

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
}
