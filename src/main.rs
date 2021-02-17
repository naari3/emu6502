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
    o: bool, // Overflow Flag
    n: bool, // Negative Flag
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Instruction {
    LDA_IM = 0xA9,
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
        self.flags.o = false;
        self.flags.n = false;
        self.a = 0;
        self.x = 0;
        self.y = 0;

        ram.initialize();
    }

    fn fetch_byte(&mut self, cycles: &mut usize, ram: &mut RAM) -> u8 {
        let byte = ram.read_byte(self.pc as usize);
        self.pc += 1;
        *cycles -= 1;
        byte
    }

    pub fn execute(&mut self, mut cycles: usize, ram: &mut RAM) {
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
    ram[0xFFFC] = Instruction::LDA_IM.into();
    ram[0xFFFD] = 0x42;
    cpu.execute(2, &mut ram);
    println!("CPU: {:?}", cpu);
}
