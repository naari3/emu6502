use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;
use std::ops::{Index, IndexMut};

// http://www.obelisk.me.uk/6502/registers.html
#[derive(Debug, Default)]
struct CPU {
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

#[allow(non_camel_case_types)]
#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Instruction {
    LDA_IM = 0xA9,
    LDA_ZP = 0xA5,
    LDA_ZPX = 0xB5,

    LDX_IM = 0xA2,
    LDX_ZP = 0xA6,
    LDX_ZPY = 0xB6,

    LDY_IM = 0xA0,
    LDY_ZP = 0xA4,
    LDY_ZPX = 0xB4,

    NOP = 0xEA,
}
#[warn(non_camel_case_types)]

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

    pub fn execute(&mut self, mut cycles: isize, ram: &mut RAM) {
        while cycles > 0 {
            let ins = self.fetch_byte(&mut cycles, ram);

            use Instruction::*;
            match Instruction::try_from(ins) {
                Ok(LDA_IM) => {
                    let byte = self.fetch_byte(&mut cycles, ram);
                    self.a = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDA_ZP) => {
                    let addr = self.fetch_byte(&mut cycles, ram);
                    let byte = self.read_byte(&mut cycles, ram, addr as usize);
                    self.a = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDA_ZPX) => {
                    let addr = self.fetch_byte(&mut cycles, ram);
                    let byte = self.read_byte(&mut cycles, ram, (addr + self.x) as usize);
                    cycles -= 1; // may be consumed by add x
                    self.a = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDX_IM) => {
                    let byte = self.fetch_byte(&mut cycles, ram);
                    self.x = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDX_ZP) => {
                    let addr = self.fetch_byte(&mut cycles, ram);
                    let byte = self.read_byte(&mut cycles, ram, addr as usize);
                    self.x = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDX_ZPY) => {
                    let addr = self.fetch_byte(&mut cycles, ram);
                    let byte = self.read_byte(&mut cycles, ram, (addr + self.y) as usize);
                    cycles -= 1; // may be consumed by add y
                    self.x = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDY_IM) => {
                    let byte = self.fetch_byte(&mut cycles, ram);
                    self.y = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDY_ZP) => {
                    let addr = self.fetch_byte(&mut cycles, ram);
                    let byte = self.read_byte(&mut cycles, ram, addr as usize);
                    self.y = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(LDY_ZPX) => {
                    let addr = self.fetch_byte(&mut cycles, ram);
                    let byte = self.read_byte(&mut cycles, ram, (addr + self.x) as usize);
                    cycles -= 1; // may be consumed by add x
                    self.y = byte;
                    self.flags.z = byte == 0;
                    self.flags.n = byte >> 6 & 1 == 1
                }
                Ok(NOP) => {
                    cycles -= 1;
                    println!("nop")
                }
                Err(_) => println!("does not match!"),
            }
        }
    }
}

const MAX_MEMORY: usize = 0x100 * 0x100;
struct RAM {
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
}

fn main() {
    let mut cpu = CPU::default();
    let mut ram = RAM::default();
    cpu.reset(&mut ram);
    ram[0xFFFC] = Instruction::LDX_IM.into();
    ram[0xFFFD] = 0x2;
    ram[0xFFFE] = Instruction::LDA_ZPX.into();
    ram[0xFFFF] = 0x40;
    ram[0x42] = 0x84;
    cpu.execute(7, &mut ram);
    println!("CPU: {:?}", cpu);
}
