use crate::instruction::{OpCode, OPCODES};
use crate::ram::MemIO;
use crate::reset::Reset;

// http://www.obelisk.me.uk/6502/registers.html
#[derive(Debug, Default, Clone, Copy)]
pub struct CPU {
    pub pc: u16, // Program Counter
    pub sp: u8,  // Stack Pointer, it uses as lower byte on "0x01XX".

    pub a: u8, // Accumulator
    pub x: u8, // Index Register X
    pub y: u8, // Index Register Y

    pub flags: StatusFlag, // Processor Status

    pub remain_cycles: usize,
    pub total_cycles: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Interrupt {
    NMI,
    Reset,
    IRQ,
    BRK,
}

impl CPU {
    pub fn reset<T: Reset + MemIO>(&mut self, ram: &mut T) {
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

        let addr_low = self.fetch_byte(ram);
        let addr_high = self.fetch_byte(ram);
        self.pc = ((addr_high as u16) << 8) + (addr_low as u16);

        ram.reset();
    }

    pub fn interrupt<T: MemIO>(&mut self, ram: &mut T, kind: Interrupt) {
        if Interrupt::IRQ == kind && self.flags.i {
            return;
        }
        if Interrupt::Reset != kind {
            if Interrupt::BRK != kind {
                self.flags.b = false;
            }
            self.flags.r = true;
            self.push_to_stack(ram, (self.pc >> 8) as u8);
            self.push_to_stack(ram, (self.pc & 0xFF) as u8);
            let flag_status = self.flags.get_as_u8();
            self.push_to_stack(ram, flag_status);
            self.flags.i = true;
        }

        self.pc = match kind {
            Interrupt::NMI => 0xFFFA,
            Interrupt::Reset => 0xFFFC,
            Interrupt::IRQ => 0xFFFE,
            Interrupt::BRK => 0xFFFE,
        };

        let addr_low = self.fetch_byte(ram);
        let addr_high = self.fetch_byte(ram);
        self.pc = ((addr_high as u16) << 8) + (addr_low as u16);
    }

    pub fn fetch_byte<T: MemIO>(&mut self, ram: &mut T) -> u8 {
        let byte = ram.read_byte(self.pc as usize);
        self.pc = self.pc.wrapping_add(1);
        self.remain_cycles += 1;
        byte
    }

    pub fn read_byte<T: MemIO>(&mut self, ram: &mut T, addr: usize) -> u8 {
        let byte = ram.read_byte(addr);
        self.remain_cycles += 1;
        byte
    }

    pub fn write_byte<T: MemIO>(&mut self, ram: &mut T, addr: usize, byte: u8) {
        ram.write_byte(addr, byte);
        self.remain_cycles += 1;
    }

    pub fn push_to_stack<T: MemIO>(&mut self, ram: &mut T, byte: u8) {
        self.write_byte(ram, (0x0100 + self.sp as u16) as usize, byte);
        self.sp = self.sp.wrapping_sub(1);
        self.remain_cycles += 1;
    }

    pub fn pull_from_stack<T: MemIO>(&mut self, ram: &mut T) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let byte = self.read_byte(ram, (0x0100 + self.sp as u16) as usize);
        self.remain_cycles += 1;
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

    pub fn execute<T: Reset + MemIO>(&mut self, mut cycles: isize, ram: &mut T) {
        self.reset(ram);
        cycles -= 2;
        while cycles > 0 {
            self.step(ram);
            cycles -= 1;
        }
    }

    pub fn step<T: MemIO>(&mut self, ram: &mut T) {
        if !self.is_waiting_for_cycles() {
            let op = self.fetch_byte(ram) as usize;
            if let Some(op) = &OPCODES[op] {
                if cfg!(feature = "logging") {
                    println!("{}", self.log(op, ram));
                }
                op.execute(self, ram);
                self.total_cycles += self.remain_cycles;
            } else {
                panic!("{:#01X} is not implemented!", op);
            }
        }
        self.remain_cycles -= 1;
    }

    fn is_waiting_for_cycles(&self) -> bool {
        self.remain_cycles > 0
    }

    #[cfg(not(feature = "logging"))]
    fn log<T: MemIO>(&mut self, _op: &OpCode, _ram: &mut T) -> String {
        "".to_string()
    }

    #[cfg(feature = "logging")]
    fn log<T: MemIO>(&mut self, op: &OpCode, ram: &mut T) -> String {
        format!(
            "{:04X}  {} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.pc - 1,
            op.log(self, ram),
            self.a,
            self.x,
            self.y,
            self.flags.get_as_u8(),
            self.sp
        )
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
        // self.r = (byte >> 5 & 1) == 1;
        self.r = true; // always true ?
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
                r: true,
                v: true,
                n: false,
            }
        );
    }
}
