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

#[derive(Debug, PartialEq, Eq)]
pub struct StatusFlag {
    pub c: bool, // Carry Flag
    pub z: bool, // Zero Flag
    pub i: bool, // Interrupt Disable
    pub d: bool, // Decimal Mode
    pub b: bool, // Break Command
    pub r: bool, // Reserved (Unused, always true)
    pub v: bool, // Overflow Flag
    pub n: bool, // Negative Flag
}

impl Default for StatusFlag {
    fn default() -> Self {
        StatusFlag {
            c: false,
            z: false,
            i: false,
            d: false,
            b: false,
            r: true,
            v: false,
            n: false,
        }
    }
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

    pub fn set_zero_and_negative_flag(&mut self, byte: u8) {
        self.flags.z = byte == 0;
        self.flags.n = (byte >> 7 & 1) == 1;
    }

    pub fn set_accumulator(&mut self, byte: u8) {
        self.a = byte;
        self.set_zero_and_negative_flag(byte);
    }

    pub fn set_index_x(&mut self, byte: u8) {
        self.x = byte;
        self.set_zero_and_negative_flag(byte);
    }

    pub fn set_index_y(&mut self, byte: u8) {
        self.y = byte;
        self.set_zero_and_negative_flag(byte);
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

impl StatusFlag {
    pub fn get_as_u8(&mut self) -> u8 {
        let byte = self.c as u8
            + ((self.z as u8) << 1)
            + ((self.i as u8) << 2)
            + ((self.d as u8) << 3)
            + ((self.b as u8) << 4)
            + ((self.r as u8) << 5)
            + ((self.v as u8) << 6)
            + ((self.n as u8) << 7);
        byte
    }

    pub fn set_as_u8(&mut self, byte: u8) {
        self.c = (byte >> 0 & 1) == 1;
        self.z = (byte >> 1 & 1) == 1;
        self.i = (byte >> 2 & 1) == 1;
        self.d = (byte >> 3 & 1) == 1;
        self.b = (byte >> 4 & 1) == 1;
        self.r = (byte >> 5 & 1) == 1;
        self.v = (byte >> 6 & 1) == 1;
        self.n = (byte >> 7 & 1) == 1;
    }
}

#[cfg(test)]
mod test_status_flags {
    use super::*;

    #[test]
    fn test_get_as_u8() {
        let mut sf = StatusFlag {
            c: true,
            z: false,
            i: true,
            d: false,
            b: true,
            r: false,
            v: true,
            n: false,
        };
        assert_eq!(sf.get_as_u8(), 0b01010101);
    }

    #[test]
    fn test_set_as_u8() {
        let mut sf = StatusFlag::default();
        sf.set_as_u8(0b01010101);
        assert_eq!(
            sf,
            StatusFlag {
                c: true,
                z: false,
                i: true,
                d: false,
                b: true,
                r: false,
                v: true,
                n: false,
            }
        );
    }
}
