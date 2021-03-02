use std::usize;

use crate::cpu::CPU;
use crate::ram::MemIO;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    LDA,
    LDX,
    LDY,

    STA,
    STX,
    STY,

    TAX,
    TAY,
    TXA,
    TYA,
    TSX,
    TXS,

    PHA,
    PLA,
    PHP,
    PLP,

    AND,
    EOR,
    ORA,
    BIT,

    ADC,
    SBC,
    CMP,
    CPX,
    CPY,

    INC,
    INX,
    INY,
    DEC,
    DEX,
    DEY,

    ASL,
    LSR,
    ROL,
    ROR,

    JMP,
    JSR,
    RTS,

    BCC,
    BCS,
    BNE,
    BEQ,
    BPL,
    BMI,
    BVC,
    BVS,

    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,

    BRK,
    NOP,
    RTI,
    //
    // Unofficial
    // see also https://wiki.nesdev.com/w/index.php/Programming_with_unofficial_opcodes
    // Convined operations
    LAX,
    SAX,
    // RMW instructions
    DCP,
    // NOPs
    SKB,
    IGN,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

// has official instruction or not
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Officiality {
    Official,
    Unofficial,
}

impl std::fmt::Display for Officiality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Official => {
                write!(f, " ")
            }
            Unofficial => {
                write!(f, "*")
            }
        }
    }
}

impl AddressingMode {
    fn fetch<T: MemIO>(&self, cpu: &mut CPU, ram: &mut T) -> Option<u8> {
        match self {
            Accumulator => Some(cpu.a),
            Immediate => Some(cpu.fetch_byte(ram)),
            ZeroPage => {
                let addr = self.get_address(cpu, ram).unwrap();
                Some(cpu.read_byte(ram, addr as usize))
            }
            ZeroPageX => {
                let addr = self.get_address(cpu, ram).unwrap();
                Some(cpu.read_byte(ram, addr as usize))
            }
            ZeroPageY => {
                let addr = self.get_address(cpu, ram).unwrap();
                Some(cpu.read_byte(ram, addr as usize))
            }
            Absolute => {
                let addr = self.get_address(cpu, ram).unwrap();
                Some(cpu.read_byte(ram, addr as usize))
            }
            AbsoluteX => {
                let before_pc = cpu.pc;
                let addr = self.get_address(cpu, ram).unwrap();
                if before_pc & 0xFF00 != addr & 0xFF00 {
                    cpu.remain_cycles += 1;
                }
                Some(cpu.read_byte(ram, addr as usize))
            }
            AbsoluteY => {
                let before_pc = cpu.pc;
                let addr = self.get_address(cpu, ram).unwrap();
                if before_pc & 0xFF00 != addr & 0xFF00 {
                    cpu.remain_cycles += 1;
                }
                Some(cpu.read_byte(ram, addr as usize))
            }
            IndexedIndirect => {
                let addr = self.get_address(cpu, ram).unwrap();
                Some(cpu.read_byte(ram, addr as usize))
            }
            IndirectIndexed => {
                let ind_addr = cpu.fetch_byte(ram);
                let addr = (cpu.read_byte(ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(ram, (ind_addr.wrapping_add(1)) as usize) as u16) << 8))
                    .wrapping_add(cpu.y as u16);
                if addr.wrapping_sub(cpu.y as u16) & 0xFF00 != addr & 0xFF00 {
                    cpu.remain_cycles += 1;
                }
                Some(cpu.read_byte(ram, addr as usize))
            }
            Implied | Relative | Indirect => panic!("You can't call fetch from {:?}!", self),
        }
    }

    fn get_address<T: MemIO>(&self, cpu: &mut CPU, ram: &mut T) -> Option<u16> {
        match self {
            ZeroPage => Some(cpu.fetch_byte(ram).into()),
            ZeroPageX => {
                cpu.remain_cycles += 1; // may be consumed by add x
                Some((cpu.fetch_byte(ram).wrapping_add(cpu.x)).into())
            }
            ZeroPageY => {
                cpu.remain_cycles += 1; // may be consumed by add y
                Some((cpu.fetch_byte(ram).wrapping_add(cpu.y)).into())
            }
            Relative => Some((((cpu.fetch_byte(ram) as i8) as i32) + cpu.pc as i32) as u16),
            Absolute => {
                let addr = cpu.fetch_byte(ram) as u16 + ((cpu.fetch_byte(ram) as u16) << 8);

                Some(addr)
            }
            AbsoluteX => {
                let addr = (cpu.fetch_byte(ram) as u16 + ((cpu.fetch_byte(ram) as u16) << 8))
                    .wrapping_add(cpu.x as u16);
                Some(addr)
            }
            AbsoluteY => {
                let addr = (cpu.fetch_byte(ram) as u16 + ((cpu.fetch_byte(ram) as u16) << 8))
                    .wrapping_add(cpu.y as u16);
                Some(addr)
            }
            Indirect => {
                let ind_addr = cpu.fetch_byte(ram) as u16 + ((cpu.fetch_byte(ram) as u16) << 8);
                let addr = cpu.read_byte(ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(
                        ram,
                        // http://www.obelisk.me.uk/6502/reference.html#JMP
                        // An original 6502 has does not correctly fetch the target address if the indirect
                        // vector falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
                        // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
                        // This is fixed in some later chips like the 65SC02 so for compatibility always ensure
                        // the indirect vector is not at the end of the page.
                        ((ind_addr & 0xFF00) + ((ind_addr as u8).wrapping_add(1)) as u16) as usize,
                    ) as u16)
                        << 8);
                Some(addr)
            }
            IndexedIndirect => {
                let ind_addr = cpu.fetch_byte(ram).wrapping_add(cpu.x);
                let addr = cpu.read_byte(ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(ram, (ind_addr.wrapping_add(1)) as usize) as u16) << 8);
                cpu.remain_cycles += 1;
                Some(addr)
            }
            IndirectIndexed => {
                let ind_addr = cpu.fetch_byte(ram);
                let addr = (cpu.read_byte(ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(ram, (ind_addr.wrapping_add(1)) as usize) as u16) << 8))
                    .wrapping_add(cpu.y as u16);
                Some(addr)
            }
            Accumulator | Implied | Immediate => {
                panic!("You can't call get_address from {:?}!", self)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OpCode(pub Instruction, pub AddressingMode, Officiality);

impl OpCode {
    pub fn execute<T: MemIO>(&self, cpu: &mut CPU, ram: &mut T) {
        let ins = &self.0;
        let adr_mode = &self.1;
        match ins {
            LDA => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.set_accumulator(byte);
            }
            LDX => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.set_index_x(byte);
            }
            LDY => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.set_index_y(byte);
            }
            STA => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                cpu.write_byte(ram, addr as usize, cpu.a);
            }
            STX => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                cpu.write_byte(ram, addr as usize, cpu.x);
            }
            STY => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                cpu.write_byte(ram, addr as usize, cpu.y);
            }
            TAX => {
                cpu.set_index_x(cpu.a);
                cpu.remain_cycles += 1;
            }
            TAY => {
                cpu.set_index_y(cpu.a);
                cpu.remain_cycles += 1;
            }
            TXA => {
                cpu.set_accumulator(cpu.x);
                cpu.remain_cycles += 1;
            }
            TYA => {
                cpu.set_accumulator(cpu.y);
                cpu.remain_cycles += 1;
            }
            TSX => {
                cpu.set_index_x(cpu.sp);
                cpu.remain_cycles += 1;
            }
            TXS => {
                cpu.sp = cpu.x;
                cpu.remain_cycles += 1;
            }
            PHA => {
                cpu.push_to_stack(ram, cpu.a);
            }
            PLA => {
                let byte = cpu.pull_from_stack(ram);
                cpu.set_accumulator(byte);
                cpu.remain_cycles += 1;
            }
            PHP => {
                let byte = cpu.flags.get_as_u8();
                // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
                let byte = byte | 0b00110000;
                cpu.push_to_stack(ram, byte);
            }
            PLP => {
                let byte = cpu.pull_from_stack(ram);
                // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
                let byte = byte & 0b11001111;
                cpu.flags.set_as_u8(byte);
                cpu.remain_cycles += 1;
            }
            AND => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.set_accumulator(cpu.a & byte);
            }
            EOR => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.set_accumulator(cpu.a ^ byte);
            }
            ORA => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.set_accumulator(cpu.a | byte);
            }
            BIT => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.flags.z = cpu.a & byte == 0;
                cpu.flags.v = (byte >> 6 & 1) == 1;
                cpu.flags.n = (byte >> 7 & 1) == 1;
            }
            ADC => {
                let before_byte = adr_mode.fetch(cpu, ram).unwrap();
                let (byte, overflowing1) = cpu.a.overflowing_add(before_byte);
                let (byte, overflowing2) = byte.overflowing_add(cpu.flags.c as u8);
                cpu.flags.c = overflowing1 || overflowing2;
                cpu.flags.v =
                    (((cpu.a ^ byte) & 0x80) != 0) && (((before_byte ^ byte) & 0x80) != 0);
                cpu.set_accumulator(byte);
            }
            SBC => {
                let before_byte = adr_mode.fetch(cpu, ram).unwrap();
                let (byte, overflowing1) = cpu.a.overflowing_sub(before_byte);
                let (byte, overflowing2) = byte.overflowing_sub(!cpu.flags.c as u8);
                cpu.flags.c = !(overflowing1 || overflowing2);
                cpu.flags.v =
                    (((cpu.a ^ before_byte) & 0x80) != 0) && (((cpu.a ^ byte) & 0x80) != 0);
                cpu.set_accumulator(byte);
            }
            CMP => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.flags.c = cpu.a >= byte;
                cpu.flags.z = cpu.a == byte;
                cpu.flags.n = cpu.a.wrapping_sub(byte) >> 7 & 1 == 1;
            }
            CPX => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.flags.c = cpu.x >= byte;
                cpu.flags.z = cpu.x == byte;
                cpu.flags.n = cpu.x.wrapping_sub(byte) >> 7 & 1 == 1;
            }
            CPY => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.flags.c = cpu.y >= byte;
                cpu.flags.z = cpu.y == byte;
                cpu.flags.n = cpu.y.wrapping_sub(byte) >> 7 & 1 == 1;
            }
            INC => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                let byte = cpu.read_byte(ram, addr as usize);
                let byte = byte.wrapping_add(1);
                cpu.remain_cycles += 1;
                cpu.set_zero_and_negative_flag(byte);
                cpu.write_byte(ram, addr as usize, byte);
            }
            INX => {
                let byte = cpu.x;
                let byte = byte.wrapping_add(1);
                cpu.remain_cycles += 1;
                cpu.set_index_x(byte);
            }
            INY => {
                let byte = cpu.y;
                let byte = byte.wrapping_add(1);
                cpu.remain_cycles += 1;
                cpu.set_index_y(byte);
            }
            DEC => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                let byte = cpu.read_byte(ram, addr as usize);
                let byte = byte.wrapping_sub(1);
                cpu.remain_cycles += 1;
                cpu.set_zero_and_negative_flag(byte);
                cpu.write_byte(ram, addr as usize, byte);
            }
            DEX => {
                let byte = cpu.x;
                let byte = byte.wrapping_sub(1);
                cpu.remain_cycles += 1;
                cpu.set_index_x(byte);
            }
            DEY => {
                let byte = cpu.y;
                let byte = byte.wrapping_sub(1);
                cpu.remain_cycles += 1;
                cpu.set_index_y(byte);
            }
            ASL => {
                cpu.remain_cycles += 1;
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, ram).unwrap();
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = byte << 1;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, ram).unwrap();
                    let byte = cpu.read_byte(ram, addr as usize);
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = byte << 1;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(ram, addr as usize, byte);
                }
            }
            LSR => {
                cpu.remain_cycles += 1;
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, ram).unwrap();
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = byte >> 1;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, ram).unwrap();
                    let byte = cpu.read_byte(ram, addr as usize);
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = byte >> 1;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(ram, addr as usize, byte);
                }
            }
            ROL => {
                cpu.remain_cycles += 1;
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, ram).unwrap();
                    let new_first_byte = cpu.flags.c as u8;
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = (byte << 1) | new_first_byte;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, ram).unwrap();
                    let byte = cpu.read_byte(ram, addr as usize);
                    let new_first_byte = cpu.flags.c as u8;
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = (byte << 1) | new_first_byte;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(ram, addr as usize, byte);
                }
            }
            ROR => {
                cpu.remain_cycles += 1;
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, ram).unwrap();
                    let new_last_byte = (cpu.flags.c as u8) << 7;
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = (byte >> 1) | new_last_byte;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, ram).unwrap();
                    let byte = cpu.read_byte(ram, addr as usize);
                    let new_last_byte = (cpu.flags.c as u8) << 7;
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = (byte >> 1) | new_last_byte;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(ram, addr as usize, byte);
                }
            }
            JMP => {
                cpu.remain_cycles += 1;
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                cpu.pc = addr;
            }
            JSR => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                let pc = cpu.pc - 1;
                cpu.push_to_stack(ram, (pc >> 8) as u8);
                cpu.push_to_stack(ram, (pc & 0xFF) as u8);
                cpu.remain_cycles -= 1;
                cpu.pc = addr;
            }
            RTS => {
                cpu.remain_cycles += 1;
                let pc =
                    (cpu.pull_from_stack(ram) as u16) + ((cpu.pull_from_stack(ram) as u16) << 8);
                cpu.pc = pc + 1;
            }
            BCC => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.c == false {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            BCS => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.c == true {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            BNE => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.z == false {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            BEQ => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.z == true {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            BPL => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.n == false {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            BMI => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.n == true {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            BVC => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.v == false {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            BVS => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                if cpu.flags.v == true {
                    cpu.remain_cycles += 1;
                    if cpu.pc & 0xFF00 != addr & 0xFF00 {
                        cpu.remain_cycles += 2;
                    }
                    cpu.pc = addr;
                }
            }
            CLC => {
                cpu.remain_cycles += 1;
                cpu.flags.c = false;
            }
            CLD => {
                cpu.remain_cycles += 1;
                cpu.flags.d = false;
            }
            CLI => {
                cpu.remain_cycles += 1;
                cpu.flags.i = false;
            }
            CLV => {
                cpu.remain_cycles += 1;
                cpu.flags.v = false;
            }
            SEC => {
                cpu.remain_cycles += 1;
                cpu.flags.c = true;
            }
            SED => {
                cpu.remain_cycles += 1;
                cpu.flags.d = true;
            }
            SEI => {
                cpu.remain_cycles += 1;
                cpu.flags.i = true;
            }
            BRK => {
                let pc = cpu.pc;
                cpu.push_to_stack(ram, (pc >> 8) as u8);
                cpu.push_to_stack(ram, (pc & 0xFF) as u8);
                cpu.flags.b = true;
                let flags = cpu.flags.get_as_u8();
                cpu.push_to_stack(ram, flags);
                cpu.flags.i = true;
                cpu.pc = ram.read_byte(0xFFFE) as u16 + (ram.read_byte(0xFFFF) as u16) << 8;
            }
            NOP => {
                cpu.remain_cycles += 1;
            }
            RTI => {
                let flags = cpu.pull_from_stack(ram);
                cpu.flags.set_as_u8(flags);
                cpu.flags.b = false;
                cpu.pc =
                    (cpu.pull_from_stack(ram) as u16) + ((cpu.pull_from_stack(ram) as u16) << 8);
                cpu.remain_cycles -= 1;
            }
            LAX => {
                let byte = adr_mode.fetch(cpu, ram).unwrap();
                cpu.set_accumulator(byte);
                cpu.set_index_x(byte);
            }
            SAX => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                let byte = cpu.a & cpu.x;
                cpu.write_byte(ram, addr as usize, byte);
            }
            DCP => {
                let addr = adr_mode.get_address(cpu, ram).unwrap();
                let byte = cpu.read_byte(ram, addr as usize);
                let byte = byte.wrapping_sub(1);
                cpu.write_byte(ram, addr as usize, byte);

                cpu.flags.c = cpu.a >= byte;
                cpu.flags.z = cpu.a == byte;
                cpu.flags.n = cpu.a.wrapping_sub(byte) >> 7 & 1 == 1;
                cpu.remain_cycles += 2;
            }
            SKB => {
                adr_mode.fetch(cpu, ram).unwrap();
            }
            IGN => {
                adr_mode.fetch(cpu, ram).unwrap();
            }
        }
    }

    #[cfg(not(feature = "logging"))]
    #[allow(dead_code)]
    pub fn log<T: MemIO>(&self, _cpu: &mut CPU, _mem: &mut T) -> String {
        "".to_string()
    }

    #[cfg(feature = "logging")]
    pub fn log<T: MemIO>(&self, cpu: &mut CPU, mem: &mut T) -> String {
        let ins_byte = mem.read_byte((cpu.pc - 1) as usize);
        let op = &OPCODES[ins_byte as usize].unwrap();

        let ins = op.0;
        let adr_mode = op.1;
        let ofc = op.2;

        let ins_name = match ins {
            SKB | IGN => "NOP".to_string(),
            _ => {
                format!("{:?}", ins)
            }
        };

        let need_byte_count = match adr_mode {
            Implied => 0,
            Accumulator => 0,
            Immediate => 1,
            ZeroPage => 1,
            ZeroPageX => 1,
            ZeroPageY => 1,
            Relative => 1,
            Absolute => 2,
            AbsoluteX => 2,
            AbsoluteY => 2,
            Indirect => 2,
            IndexedIndirect => 1,
            IndirectIndexed => 1,
        };
        let mut bytes = vec![];
        for i in 0..need_byte_count {
            bytes.push(mem.read_byte((cpu.pc + i) as usize));
        }

        let (mut addr_str, addr) = match adr_mode {
            Implied => ("".to_string(), None),
            Accumulator => ("A".to_string(), None),
            Immediate => (format!("#${:02X}", bytes[0]), Some(bytes[0] as u16)),
            ZeroPage => (format!("${:02X}", bytes[0]), Some(bytes[0] as u16)),
            ZeroPageX => (
                format!("${:02X},X", bytes[0]),
                Some((bytes[0].wrapping_add(cpu.x)) as u16),
            ),
            ZeroPageY => (
                format!("${:02X},Y", bytes[0]),
                Some((bytes[0].wrapping_add(cpu.y)) as u16),
            ),
            Relative => (
                format!(
                    "${:04X}",
                    (((cpu.pc + 1) as i32) + (bytes[0] as i8) as i32) as u16
                ),
                Some(cpu.pc + 1 + bytes[0] as u16),
            ),
            Absolute => (
                format!("${:04X}", bytes[0] as u16 + ((bytes[1] as u16) << 8)),
                Some(bytes[0] as u16 + ((bytes[1] as u16) << 8)),
            ),
            AbsoluteX => (
                format!("${:04X},X", bytes[0] as u16 + ((bytes[1] as u16) << 8)),
                Some(bytes[0] as u16 + ((bytes[1] as u16) << 8).wrapping_add(cpu.x as u16)),
            ),
            AbsoluteY => (
                format!("${:04X},Y", bytes[0] as u16 + ((bytes[1] as u16) << 8)),
                Some(
                    (bytes[0] as u16)
                        .wrapping_add(((bytes[1] as u16) << 8).wrapping_add(cpu.y as u16)),
                ),
            ),
            Indirect => {
                let in_addr = bytes[0] as u16 + ((bytes[1] as u16) << 8);
                let addr = mem.read_byte(in_addr as usize) as u16
                    + ((mem.read_byte((in_addr.wrapping_add(1)) as usize) as u16) << 8);
                (
                    format!("(${:04X})", bytes[0] as u16 + ((bytes[1] as u16) << 8)),
                    Some(addr),
                )
            }
            IndexedIndirect => {
                let in_addr = bytes[0].wrapping_add(cpu.x);
                let addr = mem.read_byte(in_addr as usize) as u16
                    + ((mem.read_byte((in_addr.wrapping_add(1)) as usize) as u16) << 8);
                (format!("(${:02X},X)", bytes[0]), Some(addr))
            }
            IndirectIndexed => {
                let in_addr = bytes[0];
                let addr = (mem.read_byte(in_addr as usize) as u16
                    + ((mem.read_byte((in_addr.wrapping_add(1)) as usize) as u16) << 8))
                    .wrapping_add(cpu.y as u16);
                (format!("(${:02X}),Y", bytes[0]), Some(addr))
            }
        };
        match ins {
            LDA | LDX | LDY | STA | STX | STY | BIT | ORA | AND | EOR | ADC | SBC | CMP | CPX
            | CPY | LSR | ASL | ROR | ROL | INC | DEC | LAX | SAX | SKB | IGN | DCP => {
                match adr_mode {
                    Implied | Accumulator | Immediate => {}
                    ZeroPageX => {
                        addr_str =
                            format!("{:} @ {:02X}", addr_str, (bytes[0]).wrapping_add(cpu.x));
                        addr_str = format!(
                            "{:} = {:02X}",
                            addr_str,
                            mem.read_byte(addr.unwrap() as usize)
                        )
                    }
                    ZeroPageY => {
                        addr_str =
                            format!("{:} @ {:02X}", addr_str, (bytes[0]).wrapping_add(cpu.y));
                        addr_str = format!(
                            "{:} = {:02X}",
                            addr_str,
                            mem.read_byte(addr.unwrap() as usize)
                        )
                    }
                    AbsoluteX => {
                        addr_str = format!(
                            "{:} @ {:04X}",
                            addr_str,
                            (bytes[0] as u16)
                                .wrapping_add(((bytes[1] as u16) << 8).wrapping_add(cpu.x as u16))
                        );
                        addr_str = format!(
                            "{:} = {:02X}",
                            addr_str,
                            mem.read_byte(addr.unwrap() as usize)
                        )
                    }
                    AbsoluteY => {
                        addr_str = format!(
                            "{:} @ {:04X}",
                            addr_str,
                            (bytes[0] as u16)
                                .wrapping_add(((bytes[1] as u16) << 8).wrapping_add(cpu.y as u16))
                        );
                        addr_str = format!(
                            "{:} = {:02X}",
                            addr_str,
                            mem.read_byte(addr.unwrap() as usize)
                        )
                    }
                    IndexedIndirect => {
                        let in_addr = bytes[0].wrapping_add(cpu.x);
                        addr_str = format!("{:} @ {:02X}", addr_str, in_addr);
                        let indexed_addr = mem.read_byte(in_addr as usize) as u16
                            + ((mem.read_byte((in_addr.wrapping_add(1)) as usize) as u16) << 8);
                        addr_str = format!("{:} = {:04X}", addr_str, indexed_addr);
                        addr_str = format!(
                            "{:} = {:02X}",
                            addr_str,
                            mem.read_byte(addr.unwrap() as usize)
                        )
                    }
                    IndirectIndexed => {
                        let in_addr = bytes[0];
                        let indirected_addr = mem.read_byte(in_addr as usize) as u16
                            + ((mem.read_byte((in_addr.wrapping_add(1)) as usize) as u16) << 8);
                        addr_str = format!("{:} = {:04X}", addr_str, indirected_addr);
                        addr_str = format!(
                            "{:} @ {:04X}",
                            addr_str,
                            indirected_addr.wrapping_add(cpu.y as u16)
                        );
                        addr_str = format!(
                            "{:} = {:02X}",
                            addr_str,
                            mem.read_byte(addr.unwrap() as usize)
                        )
                    }
                    _ => {
                        addr_str = format!(
                            "{:} = {:02X}",
                            addr_str,
                            mem.read_byte(addr.unwrap() as usize)
                        )
                    }
                }
            }
            JMP => {
                if adr_mode == Indirect {
                    addr_str = format!("{:} = {:04X}", addr_str, addr.unwrap());
                }
            }
            _ => {}
        }

        let bytes_str = match need_byte_count {
            1 => format!("{:02X} {:02X}", ins_byte, bytes[0]),
            2 => format!("{:02X} {:02X} {:02X}", ins_byte, bytes[0], bytes[1]),
            _ => format!("{:02X}", ins_byte),
        };

        format!("{: <8} {}{} {: <26} ", bytes_str, ofc, ins_name, addr_str)
    }
}

// LDA #$01
// LDA $01 => $0001
// LDA $0101

use AddressingMode::*;
use Instruction::*;
use Officiality::*;
pub const OPCODES: [Option<OpCode>; 0x100] = [
    /* 0x00 */ Some(OpCode(BRK, Implied, Official)),
    /* 0x01 */ Some(OpCode(ORA, IndexedIndirect, Official)),
    /* 0x02 */ None,
    /* 0x03 */ None,
    /* 0x04 */ Some(OpCode(IGN, ZeroPage, Unofficial)),
    /* 0x05 */ Some(OpCode(ORA, ZeroPage, Official)),
    /* 0x06 */ Some(OpCode(ASL, ZeroPage, Official)),
    /* 0x07 */ None,
    /* 0x08 */ Some(OpCode(PHP, Implied, Official)),
    /* 0x09 */ Some(OpCode(ORA, Immediate, Official)),
    /* 0x0A */ Some(OpCode(ASL, Accumulator, Official)),
    /* 0x0B */ None,
    /* 0x0C */ Some(OpCode(IGN, Absolute, Unofficial)),
    /* 0x0D */ Some(OpCode(ORA, Absolute, Official)),
    /* 0x0E */ Some(OpCode(ASL, Absolute, Official)),
    /* 0x0F */ None,
    /* 0x10 */ Some(OpCode(BPL, Relative, Official)),
    /* 0x11 */ Some(OpCode(ORA, IndirectIndexed, Official)),
    /* 0x12 */ None,
    /* 0x13 */ None,
    /* 0x14 */ Some(OpCode(IGN, ZeroPageX, Unofficial)),
    /* 0x15 */ Some(OpCode(ORA, ZeroPageX, Official)),
    /* 0x16 */ Some(OpCode(ASL, ZeroPageX, Official)),
    /* 0x17 */ None,
    /* 0x18 */ Some(OpCode(CLC, Implied, Official)),
    /* 0x19 */ Some(OpCode(ORA, AbsoluteY, Official)),
    /* 0x1A */ Some(OpCode(NOP, Implied, Unofficial)),
    /* 0x1B */ None,
    /* 0x1C */ Some(OpCode(IGN, AbsoluteX, Unofficial)),
    /* 0x1D */ Some(OpCode(ORA, AbsoluteX, Official)),
    /* 0x1E */ Some(OpCode(ASL, AbsoluteX, Official)),
    /* 0x1F */ None,
    /* 0x20 */ Some(OpCode(JSR, Absolute, Official)),
    /* 0x21 */ Some(OpCode(AND, IndexedIndirect, Official)),
    /* 0x22 */ None,
    /* 0x23 */ None,
    /* 0x24 */ Some(OpCode(BIT, ZeroPage, Official)),
    /* 0x25 */ Some(OpCode(AND, ZeroPage, Official)),
    /* 0x26 */ Some(OpCode(ROL, ZeroPage, Official)),
    /* 0x27 */ None,
    /* 0x28 */ Some(OpCode(PLP, Implied, Official)),
    /* 0x29 */ Some(OpCode(AND, Immediate, Official)),
    /* 0x2A */ Some(OpCode(ROL, Accumulator, Official)),
    /* 0x2B */ None,
    /* 0x2C */ Some(OpCode(BIT, Absolute, Official)),
    /* 0x2D */ Some(OpCode(AND, Absolute, Official)),
    /* 0x2E */ Some(OpCode(ROL, Absolute, Official)),
    /* 0x2F */ None,
    /* 0x30 */ Some(OpCode(BMI, Relative, Official)),
    /* 0x31 */ Some(OpCode(AND, IndirectIndexed, Official)),
    /* 0x32 */ None,
    /* 0x33 */ None,
    /* 0x34 */ Some(OpCode(IGN, ZeroPageX, Unofficial)),
    /* 0x35 */ Some(OpCode(AND, ZeroPageX, Official)),
    /* 0x36 */ Some(OpCode(ROL, ZeroPageX, Official)),
    /* 0x37 */ None,
    /* 0x38 */ Some(OpCode(SEC, Implied, Official)),
    /* 0x39 */ Some(OpCode(AND, AbsoluteY, Official)),
    /* 0x3A */ Some(OpCode(NOP, Implied, Unofficial)),
    /* 0x3B */ None,
    /* 0x3C */ Some(OpCode(IGN, AbsoluteX, Unofficial)),
    /* 0x3D */ Some(OpCode(AND, AbsoluteX, Official)),
    /* 0x3E */ Some(OpCode(ROL, AbsoluteX, Official)),
    /* 0x3F */ None,
    /* 0x40 */ Some(OpCode(RTI, Implied, Official)),
    /* 0x41 */ Some(OpCode(EOR, IndexedIndirect, Official)),
    /* 0x42 */ None,
    /* 0x43 */ None,
    /* 0x44 */ Some(OpCode(IGN, ZeroPage, Unofficial)),
    /* 0x45 */ Some(OpCode(EOR, ZeroPage, Official)),
    /* 0x46 */ Some(OpCode(LSR, ZeroPage, Official)),
    /* 0x47 */ None,
    /* 0x48 */ Some(OpCode(PHA, Implied, Official)),
    /* 0x49 */ Some(OpCode(EOR, Immediate, Official)),
    /* 0x4A */ Some(OpCode(LSR, Accumulator, Official)),
    /* 0x4B */ None,
    /* 0x4C */ Some(OpCode(JMP, Absolute, Official)),
    /* 0x4D */ Some(OpCode(EOR, Absolute, Official)),
    /* 0x4E */ Some(OpCode(LSR, Absolute, Official)),
    /* 0x4F */ None,
    /* 0x50 */ Some(OpCode(BVC, Relative, Official)),
    /* 0x51 */ Some(OpCode(EOR, IndirectIndexed, Official)),
    /* 0x52 */ None,
    /* 0x53 */ None,
    /* 0x54 */ Some(OpCode(IGN, ZeroPageX, Unofficial)),
    /* 0x55 */ Some(OpCode(EOR, ZeroPageX, Official)),
    /* 0x56 */ Some(OpCode(LSR, ZeroPageX, Official)),
    /* 0x57 */ None,
    /* 0x58 */ Some(OpCode(CLI, Implied, Official)),
    /* 0x59 */ Some(OpCode(EOR, AbsoluteY, Official)),
    /* 0x5A */ Some(OpCode(NOP, Implied, Unofficial)),
    /* 0x5B */ None,
    /* 0x5C */ Some(OpCode(IGN, AbsoluteX, Unofficial)),
    /* 0x5D */ Some(OpCode(EOR, AbsoluteX, Official)),
    /* 0x5E */ Some(OpCode(LSR, AbsoluteX, Official)),
    /* 0x5F */ None,
    /* 0x60 */ Some(OpCode(RTS, Implied, Official)),
    /* 0x61 */ Some(OpCode(ADC, IndexedIndirect, Official)),
    /* 0x62 */ None,
    /* 0x63 */ None,
    /* 0x64 */ Some(OpCode(IGN, ZeroPage, Unofficial)),
    /* 0x65 */ Some(OpCode(ADC, ZeroPage, Official)),
    /* 0x66 */ Some(OpCode(ROR, ZeroPage, Official)),
    /* 0x67 */ None,
    /* 0x68 */ Some(OpCode(PLA, Implied, Official)),
    /* 0x69 */ Some(OpCode(ADC, Immediate, Official)),
    /* 0x6A */ Some(OpCode(ROR, Accumulator, Official)),
    /* 0x6B */ None,
    /* 0x6C */ Some(OpCode(JMP, Indirect, Official)),
    /* 0x6D */ Some(OpCode(ADC, Absolute, Official)),
    /* 0x6E */ Some(OpCode(ROR, Absolute, Official)),
    /* 0x6F */ None,
    /* 0x70 */ Some(OpCode(BVS, Relative, Official)),
    /* 0x71 */ Some(OpCode(ADC, IndirectIndexed, Official)),
    /* 0x72 */ None,
    /* 0x73 */ None,
    /* 0x74 */ Some(OpCode(IGN, ZeroPageX, Unofficial)),
    /* 0x75 */ Some(OpCode(ADC, ZeroPageX, Official)),
    /* 0x76 */ Some(OpCode(ROR, ZeroPageX, Official)),
    /* 0x77 */ None,
    /* 0x78 */ Some(OpCode(SEI, Implied, Official)),
    /* 0x79 */ Some(OpCode(ADC, AbsoluteY, Official)),
    /* 0x7A */ Some(OpCode(NOP, Implied, Unofficial)),
    /* 0x7B */ None,
    /* 0x7C */ Some(OpCode(IGN, AbsoluteX, Unofficial)),
    /* 0x7D */ Some(OpCode(ADC, AbsoluteX, Official)),
    /* 0x7E */ Some(OpCode(ROR, AbsoluteX, Official)),
    /* 0x7F */ None,
    /* 0x80 */ Some(OpCode(SKB, Immediate, Unofficial)),
    /* 0x81 */ Some(OpCode(STA, IndexedIndirect, Official)),
    /* 0x82 */ Some(OpCode(SKB, Immediate, Unofficial)),
    /* 0x83 */ Some(OpCode(SAX, IndexedIndirect, Unofficial)),
    /* 0x84 */ Some(OpCode(STY, ZeroPage, Official)),
    /* 0x85 */ Some(OpCode(STA, ZeroPage, Official)),
    /* 0x86 */ Some(OpCode(STX, ZeroPage, Official)),
    /* 0x87 */ Some(OpCode(SAX, ZeroPage, Unofficial)),
    /* 0x88 */ Some(OpCode(DEY, Implied, Official)),
    /* 0x89 */ Some(OpCode(SKB, Immediate, Unofficial)),
    /* 0x8A */ Some(OpCode(TXA, Implied, Official)),
    /* 0x8B */ None,
    /* 0x8C */ Some(OpCode(STY, Absolute, Official)),
    /* 0x8D */ Some(OpCode(STA, Absolute, Official)),
    /* 0x8E */ Some(OpCode(STX, Absolute, Official)),
    /* 0x8F */ Some(OpCode(SAX, Absolute, Unofficial)),
    /* 0x90 */ Some(OpCode(BCC, Relative, Official)),
    /* 0x91 */ Some(OpCode(STA, IndirectIndexed, Official)),
    /* 0x92 */ None,
    /* 0x93 */ None,
    /* 0x94 */ Some(OpCode(STY, ZeroPageX, Official)),
    /* 0x95 */ Some(OpCode(STA, ZeroPageX, Official)),
    /* 0x96 */ Some(OpCode(STX, ZeroPageY, Official)),
    /* 0x97 */ Some(OpCode(SAX, ZeroPageY, Unofficial)),
    /* 0x98 */ Some(OpCode(TYA, Implied, Official)),
    /* 0x99 */ Some(OpCode(STA, AbsoluteY, Official)),
    /* 0x9A */ Some(OpCode(TXS, Implied, Official)),
    /* 0x9B */ None,
    /* 0x9C */ None,
    /* 0x9D */ Some(OpCode(STA, AbsoluteX, Official)),
    /* 0x9E */ None,
    /* 0x9F */ None,
    /* 0xA0 */ Some(OpCode(LDY, Immediate, Official)),
    /* 0xA1 */ Some(OpCode(LDA, IndexedIndirect, Official)),
    /* 0xA2 */ Some(OpCode(LDX, Immediate, Official)),
    /* 0xA3 */ Some(OpCode(LAX, IndexedIndirect, Unofficial)),
    /* 0xA4 */ Some(OpCode(LDY, ZeroPage, Official)),
    /* 0xA5 */ Some(OpCode(LDA, ZeroPage, Official)),
    /* 0xA6 */ Some(OpCode(LDX, ZeroPage, Official)),
    /* 0xA7 */ Some(OpCode(LAX, ZeroPage, Unofficial)),
    /* 0xA8 */ Some(OpCode(TAY, Implied, Official)),
    /* 0xA9 */ Some(OpCode(LDA, Immediate, Official)),
    /* 0xAA */ Some(OpCode(TAX, Implied, Official)),
    /* 0xAB */ None,
    /* 0xAC */ Some(OpCode(LDY, Absolute, Official)),
    /* 0xAD */ Some(OpCode(LDA, Absolute, Official)),
    /* 0xAE */ Some(OpCode(LDX, Absolute, Official)),
    /* 0xAF */ Some(OpCode(LAX, Absolute, Unofficial)),
    /* 0xB0 */ Some(OpCode(BCS, Relative, Official)),
    /* 0xB1 */ Some(OpCode(LDA, IndirectIndexed, Official)),
    /* 0xB2 */ None,
    /* 0xB3 */ Some(OpCode(LAX, IndirectIndexed, Unofficial)),
    /* 0xB4 */ Some(OpCode(LDY, ZeroPageX, Official)),
    /* 0xB5 */ Some(OpCode(LDA, ZeroPageX, Official)),
    /* 0xB6 */ Some(OpCode(LDX, ZeroPageY, Official)),
    /* 0xB7 */ Some(OpCode(LAX, ZeroPageY, Unofficial)),
    /* 0xB8 */ Some(OpCode(CLV, Implied, Official)),
    /* 0xB9 */ Some(OpCode(LDA, AbsoluteY, Official)),
    /* 0xBA */ Some(OpCode(TSX, Implied, Official)),
    /* 0xBB */ None,
    /* 0xBC */ Some(OpCode(LDY, AbsoluteX, Official)),
    /* 0xBD */ Some(OpCode(LDA, AbsoluteX, Official)),
    /* 0xBE */ Some(OpCode(LDX, AbsoluteY, Official)),
    /* 0xBF */ Some(OpCode(LAX, AbsoluteY, Unofficial)),
    /* 0xC0 */ Some(OpCode(CPY, Immediate, Official)),
    /* 0xC1 */ Some(OpCode(CMP, IndexedIndirect, Official)),
    /* 0xC2 */ Some(OpCode(SKB, Immediate, Unofficial)),
    /* 0xC3 */ Some(OpCode(DCP, IndexedIndirect, Unofficial)),
    /* 0xC4 */ Some(OpCode(CPY, ZeroPage, Official)),
    /* 0xC5 */ Some(OpCode(CMP, ZeroPage, Official)),
    /* 0xC6 */ Some(OpCode(DEC, ZeroPage, Official)),
    /* 0xC7 */ Some(OpCode(DCP, ZeroPage, Unofficial)),
    /* 0xC8 */ Some(OpCode(INY, Implied, Official)),
    /* 0xC9 */ Some(OpCode(CMP, Immediate, Official)),
    /* 0xCA */ Some(OpCode(DEX, Implied, Official)),
    /* 0xCB */ None,
    /* 0xCC */ Some(OpCode(CPY, Absolute, Official)),
    /* 0xCD */ Some(OpCode(CMP, Absolute, Official)),
    /* 0xCE */ Some(OpCode(DEC, Absolute, Official)),
    /* 0xCF */ Some(OpCode(DCP, Absolute, Unofficial)),
    /* 0xD0 */ Some(OpCode(BNE, Relative, Official)),
    /* 0xD1 */ Some(OpCode(CMP, IndirectIndexed, Official)),
    /* 0xD2 */ None,
    /* 0xD3 */ Some(OpCode(DCP, IndirectIndexed, Unofficial)),
    /* 0xD4 */ Some(OpCode(IGN, ZeroPageX, Unofficial)),
    /* 0xD5 */ Some(OpCode(CMP, ZeroPageX, Official)),
    /* 0xD6 */ Some(OpCode(DEC, ZeroPageX, Official)),
    /* 0xD7 */ Some(OpCode(DCP, ZeroPageX, Unofficial)),
    /* 0xD8 */ Some(OpCode(CLD, Implied, Official)),
    /* 0xD9 */ Some(OpCode(CMP, AbsoluteY, Official)),
    /* 0xDA */ Some(OpCode(NOP, Implied, Unofficial)),
    /* 0xDB */ Some(OpCode(DCP, AbsoluteY, Unofficial)),
    /* 0xDC */ Some(OpCode(IGN, AbsoluteX, Unofficial)),
    /* 0xDD */ Some(OpCode(CMP, AbsoluteX, Official)),
    /* 0xDE */ Some(OpCode(DEC, AbsoluteX, Official)),
    /* 0xDF */ Some(OpCode(DCP, AbsoluteX, Unofficial)),
    /* 0xE0 */ Some(OpCode(CPX, Immediate, Official)),
    /* 0xE1 */ Some(OpCode(SBC, IndexedIndirect, Official)),
    /* 0xE2 */ Some(OpCode(SKB, Immediate, Unofficial)),
    /* 0xE3 */ None,
    /* 0xE4 */ Some(OpCode(CPX, ZeroPage, Official)),
    /* 0xE5 */ Some(OpCode(SBC, ZeroPage, Official)),
    /* 0xE6 */ Some(OpCode(INC, ZeroPage, Official)),
    /* 0xE7 */ None,
    /* 0xE8 */ Some(OpCode(INX, Implied, Official)),
    /* 0xE9 */ Some(OpCode(SBC, Immediate, Official)),
    /* 0xEA */ Some(OpCode(NOP, Implied, Official)),
    /* 0xEB */ Some(OpCode(SBC, Immediate, Unofficial)),
    /* 0xEC */ Some(OpCode(CPX, Absolute, Official)),
    /* 0xED */ Some(OpCode(SBC, Absolute, Official)),
    /* 0xEE */ Some(OpCode(INC, Absolute, Official)),
    /* 0xEF */ None,
    /* 0xF0 */ Some(OpCode(BEQ, Relative, Official)),
    /* 0xF1 */ Some(OpCode(SBC, IndirectIndexed, Official)),
    /* 0xF2 */ None,
    /* 0xF3 */ None,
    /* 0xF4 */ Some(OpCode(IGN, ZeroPageX, Unofficial)),
    /* 0xF5 */ Some(OpCode(SBC, ZeroPageX, Official)),
    /* 0xF6 */ Some(OpCode(INC, ZeroPageX, Official)),
    /* 0xF7 */ None,
    /* 0xF8 */ Some(OpCode(SED, Implied, Official)),
    /* 0xF9 */ Some(OpCode(SBC, AbsoluteY, Official)),
    /* 0xFA */ Some(OpCode(NOP, Implied, Unofficial)),
    /* 0xFB */ None,
    /* 0xFC */ Some(OpCode(IGN, AbsoluteX, Unofficial)),
    /* 0xFD */ Some(OpCode(SBC, AbsoluteX, Official)),
    /* 0xFE */ Some(OpCode(INC, AbsoluteX, Official)),
    /* 0xFF */ None,
];

#[cfg(test)]
mod test_addressing_modes {
    use super::super::ram::RAM;
    use super::*;

    #[test]
    fn test_accumulator() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x42;
        let byte = AddressingMode::Accumulator.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
    }

    #[test]
    fn test_immediate() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x42;
        let byte = AddressingMode::Immediate.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 1);
    }

    #[test]
    fn test_zero_page() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x10] = 0x42;
        ram[0x8000] = 0x10;
        let byte = AddressingMode::ZeroPage.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 2);

        cpu.pc = 0x8000;
        let addr = AddressingMode::ZeroPage.get_address(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x10));
    }

    #[test]
    fn test_zero_page_x() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.x = 2;
        ram[0x12] = 0x42;
        ram[0x8000] = 0x10;
        let byte = AddressingMode::ZeroPageX.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 3);

        cpu.pc = 0x8000;
        let addr = AddressingMode::ZeroPageX.get_address(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x12));
    }

    #[test]
    fn test_zero_page_y() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.y = 2;
        ram[0x12] = 0x42;
        ram[0x8000] = 0x10;
        let byte = AddressingMode::ZeroPageY.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 3);

        cpu.pc = 0x8000;
        let addr = AddressingMode::ZeroPageY.get_address(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x12));
    }

    #[test]
    fn test_relative() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        ram[0x8001] = 0x02;
        let addr = AddressingMode::Relative.get_address(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x8004));
    }

    #[test]
    fn test_absolute() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x8001] = 0x01;
        ram[0x0100] = 0x42;
        let byte = AddressingMode::Absolute.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 3);

        cpu.pc = 0x8000;
        let addr = AddressingMode::Absolute.get_address(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x0100));
    }

    #[test]
    fn test_absolute_x() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x00;
        ram[0x8001] = 0x01;
        ram[0x0101] = 0x42;
        let byte = AddressingMode::AbsoluteX.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::AbsoluteX.get_address(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x0101));

        cpu.remain_cycles = 0;
        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x50;
        ram[0x8001] = 0x80;
        ram[0x8051] = 0x42;
        let addr = AddressingMode::AbsoluteX.fetch(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x42));
        assert_eq!(cpu.remain_cycles, 3);

        cpu.remain_cycles = 0;
        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x50;
        ram[0x8001] = 0x81;
        ram[0x8151] = 0x42;
        let addr = AddressingMode::AbsoluteX.fetch(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x42));
        assert_eq!(cpu.remain_cycles, 4);
    }

    #[test]
    fn test_absolute_y() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.y = 1;
        ram[0x8000] = 0x00;
        ram[0x8001] = 0x01;
        ram[0x0101] = 0x42;
        let byte = AddressingMode::AbsoluteY.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::AbsoluteY.get_address(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x0101));

        cpu.remain_cycles = 0;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x50;
        ram[0x8001] = 0x80;
        ram[0x8051] = 0x42;
        let addr = AddressingMode::AbsoluteY.fetch(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x42));
        assert_eq!(cpu.remain_cycles, 3);

        cpu.remain_cycles = 0;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x50;
        ram[0x8001] = 0x81;
        ram[0x8151] = 0x42;
        let addr = AddressingMode::AbsoluteY.fetch(&mut cpu, &mut ram);
        assert_eq!(addr, Some(0x42));
        assert_eq!(cpu.remain_cycles, 4);
    }

    #[test]
    fn test_indirect() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x02;
        ram[0x8001] = 0x01;
        ram[0x0102] = 0x04;
        ram[0x0103] = 0x03;
        let byte = AddressingMode::Indirect.get_address(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x0304));
    }

    #[test]
    fn test_indexed_indirect() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x00;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        let byte = AddressingMode::IndexedIndirect.get_address(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x0304));

        cpu.remain_cycles = 0;
        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x00;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        ram[0x0304] = 0x42;
        let byte = AddressingMode::IndexedIndirect.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 5);
    }

    #[test]
    fn test_indirect_indexed() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.y = 1;
        ram[0x8000] = 0x01;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        let byte = AddressingMode::IndirectIndexed.get_address(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x0305));

        cpu.pc = 0x8000;
        cpu.y = 1;
        ram[0x8000] = 0x01;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        ram[0x0305] = 0x42;
        let byte = AddressingMode::IndirectIndexed.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.remain_cycles = 0;
        cpu.pc = 0x8000;
        cpu.y = 1;
        ram[0x8000] = 0x01;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        ram[0x0305] = 0x42;
        let byte = AddressingMode::IndirectIndexed.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 4);

        cpu.remain_cycles = 0;
        cpu.pc = 0x8000;
        cpu.y = 0x10;
        ram[0x8000] = 0x01;
        ram[0x01] = 0xF4;
        ram[0x02] = 0x02;
        ram[0x0304] = 0x42;
        let byte = AddressingMode::IndirectIndexed.fetch(&mut cpu, &mut ram);
        assert_eq!(byte, Some(0x42));
        assert_eq!(cpu.remain_cycles, 5);
    }
}

#[cfg(test)]
mod test_instructions {
    use super::super::ram::RAM;
    use super::*;

    #[test]
    fn test_lda() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0b10000010;
        OpCode(Instruction::LDA, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDA, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDA, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_ldx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0b10000010;
        OpCode(Instruction::LDX, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDX, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDX, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 1);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_ldy() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0b10000010;
        OpCode(Instruction::LDY, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDY, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDY, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 1);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.a = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STA, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_stx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.x = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STX, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_sty() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.y = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STY, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x42;
        cpu.x = 0;
        OpCode(Instruction::TAX, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_tay() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x42;
        cpu.y = 0;
        OpCode(Instruction::TAY, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0x42);
    }

    #[test]
    fn test_txa() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0x42;
        cpu.a = 0;
        OpCode(Instruction::TXA, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tya() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.y = 0x42;
        cpu.a = 0;
        OpCode(Instruction::TYA, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tsx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0x42;
        cpu.x = 0;
        OpCode(Instruction::TSX, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_txs() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0x42;
        cpu.sp = 0;
        OpCode(Instruction::TXS, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.sp, 0x42);
    }

    #[test]
    fn test_pha() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0xFF;
        cpu.pc = 0x8000;
        cpu.a = 0x42;
        OpCode(Instruction::PHA, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(ram[0x1FF], 0x42);
    }

    #[test]
    fn test_pla() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0xFE;
        cpu.pc = 0x8000;
        cpu.a = 0;
        ram[0x1FF] = 0x42;
        OpCode(Instruction::PLA, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_php() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0xFF;
        cpu.flags.c = true;
        cpu.flags.r = true;

        OpCode(Instruction::PHP, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(ram[0x1FF], 0b00110001);
    }

    #[test]
    fn test_plp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0xFE;
        cpu.flags.c = false;
        cpu.flags.r = false;
        ram[0x1FF] = 0b00100001;

        OpCode(Instruction::PLP, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.r, true);
    }

    #[test]
    fn test_and() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.a = 0b00011000;
        ram[0x8000] = 0b00001111;

        OpCode(Instruction::AND, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b00001000);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_eor() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.a = 0b00001111;
        ram[0x8000] = 0b00001000;

        OpCode(Instruction::EOR, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b00000111);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.a = 0b00001111;
        ram[0x8000] = 0b11110000;

        OpCode(Instruction::ORA, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b11111111);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_bit() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.a = 0;
        ram[0x1] = 0;
        ram[0x8000] = 0x1;

        OpCode(Instruction::BIT, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.v, false);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        cpu.a = 0;
        ram[0x1] = 0b11000000;
        ram[0x8000] = 0x1;

        OpCode(Instruction::BIT, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.v, true);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_adc() {
        // TODO: implement test for v flag
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x20;
        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x10;
        OpCode(Instruction::ADC, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x30);
        assert_eq!(cpu.flags.c, false);

        cpu.a = 0xFF;
        cpu.pc = 0x8000;
        cpu.flags.c = true;
        ram[0x8000] = 1;
        OpCode(Instruction::ADC, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.flags.c, true);
    }

    #[test]
    fn test_sbc() {
        // TODO: implement test for v flag
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x30;
        cpu.pc = 0x8000;
        cpu.flags.c = true;
        ram[0x8000] = 0x10;
        OpCode(Instruction::SBC, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x20);
        assert_eq!(cpu.flags.c, true);

        cpu.a = 0x00;
        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 1;
        OpCode(Instruction::SBC, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_cmp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x10;
        OpCode(Instruction::CMP, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.a = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CMP, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_cpx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x10;
        OpCode(Instruction::CPX, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.x = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CPX, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_cpy() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.y = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x10;
        OpCode(Instruction::CPY, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.y = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CPY, AddressingMode::Immediate, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0xFE;
        OpCode(Instruction::INC, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0xFF;
        OpCode(Instruction::INC, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_inx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0xFE;
        OpCode(Instruction::INX, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.x = 0xFF;
        OpCode(Instruction::INX, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.y = 0xFE;
        OpCode(Instruction::INY, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.y = 0xFF;
        OpCode(Instruction::INY, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_dec() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0x01;
        OpCode(Instruction::DEC, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0x00;
        OpCode(Instruction::DEC, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_dex() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0x01;
        OpCode(Instruction::DEX, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x00);

        cpu.x = 0x00;
        OpCode(Instruction::DEX, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0xFF);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.y = 0x01;
        OpCode(Instruction::DEY, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0x00);

        cpu.y = 0x00;
        OpCode(Instruction::DEY, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0xFF);
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b10111111;
        OpCode(Instruction::ASL, AddressingMode::Accumulator, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b01111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b01000000;
        OpCode(Instruction::ASL, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01], 0b10000000);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_lsr() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b11111101;
        OpCode(Instruction::LSR, AddressingMode::Accumulator, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b01111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b00000010;
        OpCode(Instruction::LSR, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01], 0b00000001);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b10111111;
        cpu.flags.c = true;
        OpCode(Instruction::ROL, AddressingMode::Accumulator, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b01111111);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b01000000;
        OpCode(Instruction::ROL, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01], 0b10000000);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_ror() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b11111101;
        cpu.flags.c = true;
        OpCode(Instruction::ROR, AddressingMode::Accumulator, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b11111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b00000010;
        OpCode(Instruction::ROR, AddressingMode::ZeroPage, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01], 0b00000001);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x02;
        ram[0x8001] = 0x01;
        OpCode(Instruction::JMP, AddressingMode::Absolute, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x0102);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x02;
        ram[0x8001] = 0x01;
        ram[0x0102] = 0x04;
        ram[0x0103] = 0x03;
        OpCode(Instruction::JMP, AddressingMode::Indirect, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x0304);
    }

    #[test]
    fn test_jsr() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.sp = 0xFF;
        ram[0x8001] = 0x02;
        ram[0x8002] = 0x01;
        OpCode(Instruction::JSR, AddressingMode::Absolute, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(ram[0x01FF], 0x80);
        assert_eq!(ram[0x01FE], 0x02);
    }

    #[test]
    fn test_rts() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.sp = 0xFD;
        ram[0x01FE] = 0x02;
        ram[0x01FF] = 0x01;
        OpCode(Instruction::RTS, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x0103);
    }

    #[test]
    fn test_bcc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.c = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCC, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.c = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCC, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bcs() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.c = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCS, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.c = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCS, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bne() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.z = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BNE, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.z = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BNE, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_beq() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.z = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BEQ, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.z = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BEQ, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bpl() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.n = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BPL, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.n = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BPL, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bmi() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.n = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BMI, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.n = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BMI, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bvc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.v = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVC, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.v = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVC, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bvs() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.v = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVS, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.v = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVS, AddressingMode::Relative, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_clc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.c = true;
        OpCode(Instruction::CLC, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_cld() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.d = true;
        OpCode(Instruction::CLD, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.d, false);
    }

    #[test]
    fn test_cli() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.i = true;
        OpCode(Instruction::CLI, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.i, false);
    }

    #[test]
    fn test_clv() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.v = true;
        OpCode(Instruction::CLV, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.v, false);
    }

    #[test]
    fn test_sec() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.c = false;
        OpCode(Instruction::SEC, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
    }

    #[test]
    fn test_sed() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.d = false;
        OpCode(Instruction::SED, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.d, true);
    }

    #[test]
    fn test_sei() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.i = false;
        OpCode(Instruction::SEI, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.i, true);
    }

    #[test]
    fn test_brk() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.sp = 0xFF;
        OpCode(Instruction::BRK, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01FE], 0x00);
        assert_eq!(ram[0x01FF], 0x80);
        assert_eq!(ram[0x01FD], 0b00110000);
        assert_eq!(cpu.flags.i, true);
    }

    #[test]
    fn test_rti() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.sp = 0xFC;
        cpu.flags.i = true;
        ram[0x01FD] = 0b00110001;
        ram[0x01FE] = 0x00;
        ram[0x01FF] = 0x90;
        OpCode(Instruction::RTI, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.i, false);
        assert_eq!(cpu.flags.b, false);
        assert_eq!(cpu.pc, 0x9000);
    }

    #[test]
    fn test_nop() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        OpCode(Instruction::NOP, AddressingMode::Implied, Official).execute(&mut cpu, &mut ram);
    }

    #[test]
    fn test_lax() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x21;
        ram[0x21] = 0b10000010;
        OpCode(Instruction::LAX, AddressingMode::ZeroPage, Unofficial).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b10000010);
        assert_eq!(cpu.x, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_sax() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x42;
        cpu.a = 0b00011111;
        cpu.x = 0b11110000;
        OpCode(Instruction::SAX, AddressingMode::ZeroPage, Unofficial).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x42], 0b00010000);
    }

    #[test]
    fn test_dcp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x10;
        ram[0x10] = 0x11;
        OpCode(Instruction::DCP, AddressingMode::ZeroPage, Unofficial).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_skb() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;

        OpCode(Instruction::SKB, AddressingMode::Immediate, Unofficial).execute(&mut cpu, &mut ram);

        assert_eq!(cpu.remain_cycles, 1);
    }

    #[test]
    fn test_ign() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        ram[0x8001] = 0x10;

        OpCode(Instruction::IGN, AddressingMode::Absolute, Unofficial).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.remain_cycles, 3);
    }
}
