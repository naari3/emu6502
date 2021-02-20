use crate::instruction::OPCODES;
use crate::ram::RAM;

// http://www.obelisk.me.uk/6502/registers.html
#[derive(Debug, Default)]
pub struct CPU {
    pub pc: u16, // Program Counter
    pub sp: u8,  // Stack Pointer, it uses as lower byte on "0x01XX".

    pub a: u8, // Accumulator
    pub x: u8, // Index Register X
    pub y: u8, // Index Register Y

    pub flags: StatusFlag, // Processor Status
}

#[derive(Debug, Default)]
pub struct StatusFlag {
    pub c: bool, // Carry Flag
    pub z: bool, // Zero Flag
    pub i: bool, // Interrupt Disable
    pub d: bool, // Decimal Mode
    pub b: bool, // Break Command
    pub v: bool, // Overflow Flag
    pub n: bool, // Negative Flag
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

    pub fn fetch_byte(&mut self, cycles: &mut isize, ram: &mut RAM) -> u8 {
        let byte = ram.read_byte(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        *cycles -= 1;
        byte
    }

    pub fn read_byte(&mut self, cycles: &mut isize, ram: &mut RAM, addr: usize) -> u8 {
        let byte = ram.read_byte(addr);
        *cycles -= 1;
        byte
    }

    pub fn write_byte(&mut self, cycles: &mut isize, ram: &mut RAM, addr: usize, byte: u8) {
        ram.write_byte(addr, byte);
        *cycles -= 1;
    }

    pub fn push_to_stack(&mut self, cycles: &mut isize, ram: &mut RAM, byte: u8) {
        self.write_byte(cycles, ram, (0x0100 + self.sp as u16) as usize, byte);
        self.sp -= 1;
        *cycles -= 1;
    }

    pub fn pull_from_stack(&mut self, cycles: &mut isize, ram: &mut RAM) -> u8 {
        self.sp += 1;
        let byte = self.read_byte(cycles, ram, (0x0100 + self.sp as u16) as usize);
        *cycles -= 1;
        byte
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
