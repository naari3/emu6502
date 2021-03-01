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

impl AddressingMode {
    fn fetch<T: MemIO>(&self, cpu: &mut CPU, ram: &mut T) -> Option<u8> {
        let byte = match self {
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
                let ind_addr = cpu.fetch_byte(ram) as u16;
                let addr = cpu.read_byte(ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(ram, (ind_addr + 1) as usize) as u16) << 8)
                    + cpu.y as u16;
                if (addr - cpu.y as u16) & 0xFF00 != addr & 0xFF00 {
                    cpu.remain_cycles += 1;
                }
                Some(cpu.read_byte(ram, addr as usize))
            }
            Implied => panic!("You can't call fetch from {:?}!", self),
            Relative => panic!("You can't call fetch from {:?}!", self),
            Indirect => panic!("You can't call fetch from {:?}!", self),
        };
        // if cpu.debug {
        //     match byte {
        //         Some(b) => {
        //             println!("fetch: 0x{:02X}", b);
        //         }
        //         None => {}
        //     }
        // }
        byte
    }

    fn get_address<T: MemIO>(&self, cpu: &mut CPU, ram: &mut T) -> Option<u16> {
        let addr = match self {
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
                    + ((cpu.read_byte(ram, (ind_addr + 1) as usize) as u16) << 8);
                Some(addr)
            }
            IndexedIndirect => {
                let ind_addr = cpu.fetch_byte(ram) as u16 + cpu.x as u16;
                let addr = cpu.read_byte(ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(ram, (ind_addr + 1) as usize) as u16) << 8);
                cpu.remain_cycles += 1;
                Some(addr)
            }
            IndirectIndexed => {
                let ind_addr = cpu.fetch_byte(ram) as u16;
                let addr = cpu.read_byte(ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(ram, (ind_addr + 1) as usize) as u16) << 8)
                    + cpu.y as u16;
                Some(addr)
            }
            Accumulator => panic!("You can't call get_address from {:?}!", self),
            Implied => panic!("You can't call get_address from {:?}!", self),
            Immediate => panic!("You can't call get_address from {:?}!", self),
        };
        // if cpu.debug {
        //     match addr {
        //         Some(a) => {
        //             println!("addr: 0x{:04X}", a);
        //         }
        //         None => {}
        //     }
        // }
        addr
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OpCode(pub Instruction, pub AddressingMode);

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
                cpu.push_to_stack(ram, byte);
            }
            PLP => {
                let byte = cpu.pull_from_stack(ram);
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
                cpu.push_to_stack(ram, (pc & 0xFF) as u8);
                cpu.push_to_stack(ram, (pc >> 8) as u8);
                cpu.remain_cycles -= 1;
                cpu.pc = addr;
            }
            RTS => {
                cpu.remain_cycles += 1;
                let pc = ((cpu.pull_from_stack(ram) as u16) << 8) + cpu.pull_from_stack(ram) as u16;
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
                cpu.push_to_stack(ram, (pc & 0xFF) as u8);
                cpu.push_to_stack(ram, (pc >> 8) as u8);
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
                    ((cpu.pull_from_stack(ram) as u16) << 8 + cpu.pull_from_stack(ram) as u16) + 1;
                cpu.remain_cycles -= 1;
            }
        }
    }

    #[cfg(not(feature = "logging"))]
    pub fn log<T: MemIO>(&self, cpu: &mut CPU, mem: &mut T) -> String {
        "".to_string()
    }

    #[cfg(feature = "logging")]
    pub fn log<T: MemIO>(&self, cpu: &mut CPU, mem: &mut T) -> String {
        let ins = self.0;
        let adr_mode = self.1;

        let ins_byte = OPCODES
            .iter()
            .position(|&o| match o {
                Some(ins2) => ins == ins2.0 && adr_mode == ins2.1,
                None => false,
            })
            .unwrap() as u8;
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
                format!("${:04X}", cpu.pc + 1 + bytes[0] as u16),
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
                Some(bytes[0] as u16 + ((bytes[1] as u16) << 8).wrapping_add(cpu.y as u16)),
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
                let addr = mem.read_byte(in_addr as usize) as u16
                    + ((mem.read_byte((in_addr.wrapping_add(1)) as usize) as u16) << 8)
                    + cpu.y as u16;
                (format!("(${:02X}),Y", bytes[0]), Some(addr))
            }
        };
        match ins {
            STA => {
                addr_str = format!(
                    "{:} = {:02X}",
                    addr_str,
                    mem.read_byte(addr.unwrap() as usize)
                )
            }
            STX => {
                addr_str = format!(
                    "{:} = {:02X}",
                    addr_str,
                    mem.read_byte(addr.unwrap() as usize)
                )
            }
            STY => {
                addr_str = format!(
                    "{:} = {:02X}",
                    addr_str,
                    mem.read_byte(addr.unwrap() as usize)
                )
            }
            BIT => {
                addr_str = format!(
                    "{:} = {:02X}",
                    addr_str,
                    mem.read_byte(addr.unwrap() as usize)
                )
            }
            _ => {}
        }

        let bytes_str = match need_byte_count {
            1 => format!("{:02X} {:02X}", ins_byte, bytes[0]),
            2 => format!("{:02X} {:02X} {:02X}", ins_byte, bytes[0], bytes[1]),
            _ => format!("{:02X}", ins_byte),
        };

        format!("{: <8}  {:?} {: <26} ", bytes_str, ins, addr_str)
    }
}

// LDA #$01
// LDA $01 => $0001
// LDA $0101

use AddressingMode::*;
use Instruction::*;
pub const OPCODES: [Option<OpCode>; 0x100] = [
    Some(OpCode(BRK, Implied)),         // $00    BRK	       Implied
    Some(OpCode(ORA, IndexedIndirect)), // $01    ORA ($NN,X)  IndexedIndirect
    None,                               // $02
    None,                               // $03
    None,                               // $04
    Some(OpCode(ORA, ZeroPage)),        // $05    ORA $NN      ZeroPage
    Some(OpCode(ASL, ZeroPage)),        // $06    ASL $NN      ZeroPage
    None,                               // $07
    Some(OpCode(PHP, Implied)),         // $08    PHP          Implied
    Some(OpCode(ORA, Immediate)),       // $09    ORA #$NN     Immediate
    Some(OpCode(ASL, Accumulator)),     // $0A    ASL A        Accumulator
    None,                               // $0B
    None,                               // $0C
    Some(OpCode(ORA, Absolute)),        // $0D    ORA $NNNN    Absolute
    Some(OpCode(ASL, Absolute)),        // $0E    ASL $NNNN    Absolute
    None,                               // $0F
    Some(OpCode(BPL, Relative)),        // $10    BPL $NN      Relative
    Some(OpCode(ORA, IndirectIndexed)), // $11    ORA ($NN),Y  IndirectIndexed
    None,                               // $12
    None,                               // $13
    None,                               // $14
    Some(OpCode(ORA, ZeroPageX)),       // $15    ORA $NN,X    ZeroPageX
    Some(OpCode(ASL, ZeroPageX)),       // $16    ASL $NN,X    ZeroPageX
    None,                               // $17
    Some(OpCode(CLC, Implied)),         // $18    CLC          Implied
    Some(OpCode(ORA, AbsoluteY)),       // $19    ORA $NNNN,Y  AbsoluteY
    None,                               // $1A
    None,                               // $1B
    None,                               // $1C
    Some(OpCode(ORA, AbsoluteX)),       // $1D    ORA $NNNN,X  AbsoluteX
    Some(OpCode(ASL, AbsoluteX)),       // $1E    ASL $NNNN,X  AbsoluteX
    None,                               // $1F
    Some(OpCode(JSR, Absolute)),        // $20    JSR $NNNN    Absolute
    Some(OpCode(AND, IndexedIndirect)), // $21    AND ($NN,X)  IndexedIndirect
    None,                               // $22
    None,                               // $23
    Some(OpCode(BIT, ZeroPage)),        // $24    BIT $NN      ZeroPage
    Some(OpCode(AND, ZeroPage)),        // $25    AND $NN      ZeroPage
    Some(OpCode(ROL, ZeroPage)),        // $26    ROL $NN      ZeroPage
    None,                               // $27
    Some(OpCode(PLP, Implied)),         // $28    PLP          Implied
    Some(OpCode(AND, Immediate)),       // $29    AND #$NN     Immediate
    Some(OpCode(ROL, Accumulator)),     // $2A    ROL A        Accumulator
    None,                               // $2B
    Some(OpCode(BIT, Absolute)),        // $2C    BIT $NNNN    Absolute
    Some(OpCode(AND, Absolute)),        // $2D    AND $NNNN    Absolute
    Some(OpCode(ROL, Absolute)),        // $2E    ROL $NNNN    Absolute
    None,                               // $2F
    Some(OpCode(BMI, Relative)),        // $30    BMI $NN      Relative
    Some(OpCode(AND, IndirectIndexed)), // $31    AND ($NN),Y  IndirectIndexed
    None,                               // $32
    None,                               // $33
    None,                               // $34
    Some(OpCode(AND, ZeroPageX)),       // $35    AND $NN,X    ZeroPageX
    Some(OpCode(ROL, ZeroPageX)),       // $36    ROL $NN,X    ZeroPageX
    None,                               // $37
    Some(OpCode(SEC, Implied)),         // $38    SEC          Implied
    Some(OpCode(AND, AbsoluteY)),       // $39    AND $NNNN,Y  AbsoluteY
    None,                               // $3A
    None,                               // $3B
    None,                               // $3C
    Some(OpCode(AND, AbsoluteX)),       // $3D    AND $NNNN,X  AbsoluteX
    Some(OpCode(ROL, AbsoluteX)),       // $3E    ROL $NNNN,X  AbsoluteX
    None,                               // $3F
    Some(OpCode(RTI, Implied)),         // $40    RTI          Implied
    Some(OpCode(EOR, IndexedIndirect)), // $41    EOR ($NN,X)  IndexedIndirect
    None,                               // $42
    None,                               // $43
    None,                               // $44
    Some(OpCode(EOR, ZeroPage)),        // $45    EOR $NN      ZeroPage
    Some(OpCode(LSR, ZeroPage)),        // $46    LSR $NN      ZeroPage
    None,                               // $47
    Some(OpCode(PHA, Implied)),         // $48    PHA          Implied
    Some(OpCode(EOR, Immediate)),       // $49    EOR #$NN     Immediate
    Some(OpCode(LSR, Accumulator)),     // $4A    LSR A        Accumulator
    None,                               // $4B
    Some(OpCode(JMP, Absolute)),        // $4C    JMP $NNNN    Absolute
    Some(OpCode(EOR, Absolute)),        // $4D    EOR $NNNN    Absolute
    Some(OpCode(LSR, Absolute)),        // $4E    LSR $NNNN    Absolute
    None,                               // $4F
    Some(OpCode(BVC, Relative)),        // $50    BVC $NN      Relative
    Some(OpCode(EOR, IndirectIndexed)), // $51    EOR ($NN),Y  IndirectIndexed
    None,                               // $52
    None,                               // $53
    None,                               // $54
    Some(OpCode(EOR, ZeroPageX)),       // $55    EOR $NN,X    ZeroPageX
    Some(OpCode(LSR, ZeroPageX)),       // $56    LSR $NN,X    ZeroPageX
    None,                               // $57
    Some(OpCode(CLI, Implied)),         // $58    CLI          Implied
    Some(OpCode(EOR, AbsoluteY)),       // $59    EOR $NNNN,Y  AbsoluteY
    None,                               // $5A
    None,                               // $5B
    None,                               // $5C
    Some(OpCode(EOR, AbsoluteX)),       // $5D    EOR $NNNN,X  AbsoluteX
    Some(OpCode(LSR, AbsoluteX)),       // $5E    LSR $NNNN,X  AbsoluteX
    None,                               // $5F
    Some(OpCode(RTS, Implied)),         // $60    RTS          Implied
    Some(OpCode(ADC, IndexedIndirect)), // $61    ADC ($NN,X)  IndexedIndirect
    None,                               // $62
    None,                               // $63
    None,                               // $64
    Some(OpCode(ADC, ZeroPage)),        // $65    ADC $NN      ZeroPage
    Some(OpCode(ROR, ZeroPage)),        // $66    ROR $NN      ZeroPage
    None,                               // $67
    Some(OpCode(PLA, Implied)),         // $68    PLA          Implied
    Some(OpCode(ADC, Immediate)),       // $69    ADC #$NN     Immediate
    Some(OpCode(ROR, Accumulator)),     // $6A    ROR A        Accumulator
    None,                               // $6B
    Some(OpCode(JMP, Indirect)),        // $6C    JMP $NN      Indirect
    Some(OpCode(ADC, Absolute)),        // $6D    ADC $NNNN    Absolute
    Some(OpCode(ROR, AbsoluteX)),       // $6E    ROR $NNNN,X  AbsoluteX
    None,                               // $6F
    Some(OpCode(BVS, Relative)),        // $70    BVS $NN      Relative
    Some(OpCode(ADC, IndirectIndexed)), // $71    ADC ($NN),Y  IndirectIndexed
    None,                               // $72
    None,                               // $73
    None,                               // $74
    Some(OpCode(ADC, ZeroPageX)),       // $75    ADC $NN,X    ZeroPageX
    Some(OpCode(ROR, ZeroPageX)),       // $76    ROR $NN,X    ZeroPageX
    None,                               // $77
    Some(OpCode(SEI, Implied)),         // $78    SEI          Implied
    Some(OpCode(ADC, AbsoluteY)),       // $79    ADC $NNNN,Y  AbsoluteY
    None,                               // $7A
    None,                               // $7B
    None,                               // $7C
    Some(OpCode(ADC, AbsoluteX)),       // $7D    ADC $NNNN,X  AbsoluteX
    Some(OpCode(ROR, Absolute)),        // $7E    ROR $NNNN    Absolute
    None,                               // $7F
    None,                               // $80
    Some(OpCode(STA, IndexedIndirect)), // $81    STA ($NN,X)  IndexedIndirect
    None,                               // $82
    None,                               // $83
    Some(OpCode(STY, ZeroPage)),        // $84    STY $NN      ZeroPage
    Some(OpCode(STA, ZeroPage)),        // $85    STA $NN      ZeroPage
    Some(OpCode(STX, ZeroPage)),        // $86    STX $NN      ZeroPage
    None,                               // $87
    Some(OpCode(DEY, Implied)),         // $88    DEY          Implied
    None,                               // $89
    Some(OpCode(TXA, Implied)),         // $8A    TXA          Implied
    None,                               // $8B
    Some(OpCode(STY, Absolute)),        // $8C    STY $NNNN    Absolute
    Some(OpCode(STA, Absolute)),        // $8D    STA $NNNN    Absolute
    Some(OpCode(STX, Absolute)),        // $8E    STX $NNNN    Absolute
    None,                               // $8F
    Some(OpCode(BCC, Relative)),        // $90    BCC $NN      Relative
    Some(OpCode(STA, IndirectIndexed)), // $91    STA ($NN),Y  IndirectIndexed
    None,                               // $92
    None,                               // $93
    Some(OpCode(STY, ZeroPageX)),       // $94    STY $NN,X    ZeroPageX
    Some(OpCode(STA, ZeroPageX)),       // $95    STA $NN,X    ZeroPageX
    Some(OpCode(STX, ZeroPageY)),       // $96    STX $NN,Y    ZeroPageY
    None,                               // $97
    Some(OpCode(TYA, Implied)),         // $98    TYA          Implied
    Some(OpCode(STA, AbsoluteY)),       // $99    STA $NNNN,Y  AbsoluteY
    Some(OpCode(TXS, Implied)),         // $9A    TXS          Implied
    None,                               // $9B
    None,                               // $9C
    Some(OpCode(STA, AbsoluteX)),       // $9D    STA $NNNN,X  AbsoluteX
    None,                               // $9E
    None,                               // $9F
    Some(OpCode(LDY, Immediate)),       // $A0    LDY #$NN     Immediate
    Some(OpCode(LDA, IndexedIndirect)), // $A1    LDA ($NN,X)  IndexedIndirect
    Some(OpCode(LDX, Immediate)),       // $A2    LDX #$NN     Immediate
    None,                               // $A3
    Some(OpCode(LDY, ZeroPage)),        // $A4    LDY $NN      ZeroPage
    Some(OpCode(LDA, ZeroPage)),        // $A5    LDA $NN      ZeroPage
    Some(OpCode(LDX, ZeroPage)),        // $A6    LDX $NN      ZeroPage
    None,                               // $A7
    Some(OpCode(TAY, Implied)),         // $A8    TAY          Implied
    Some(OpCode(LDA, Immediate)),       // $A9    LDA #$NN     Immediate
    Some(OpCode(TAX, Implied)),         // $AA    TAX          Implied
    None,                               // $AB
    Some(OpCode(LDY, Absolute)),        // $AC    LDY $NNNN    Absolute
    Some(OpCode(LDA, Absolute)),        // $AD    LDA $NNNN    Absolute
    Some(OpCode(LDX, Absolute)),        // $AE    LDX $NNNN    Absolute
    None,                               // $AF
    Some(OpCode(BCS, Relative)),        // $B0    BCS $NN      Relative
    Some(OpCode(LDA, IndirectIndexed)), // $B1    LDA ($NN),Y  IndirectIndexed
    None,                               // $B2
    None,                               // $B3
    Some(OpCode(LDY, ZeroPageX)),       // $B4    LDY $NN,X    ZeroPageX
    Some(OpCode(LDA, ZeroPageX)),       // $B5    LDA $NN,X    ZeroPageX
    Some(OpCode(LDX, ZeroPageY)),       // $B6    LDX $NN,Y    ZeroPageY
    None,                               // $B7
    Some(OpCode(CLV, Implied)),         // $B8    CLV          Implied
    Some(OpCode(LDA, AbsoluteY)),       // $B9    LDA $NNNN,Y  AbsoluteY
    Some(OpCode(TSX, Implied)),         // $BA    TSX          Implied
    None,                               // $BB
    Some(OpCode(LDY, AbsoluteX)),       // $BC    LDY $NNNN,X  AbsoluteX
    Some(OpCode(LDA, AbsoluteX)),       // $BD    LDA $NNNN,X  AbsoluteX
    Some(OpCode(LDX, AbsoluteY)),       // $BE    LDX $NNNN,Y  AbsoluteY
    None,                               // $BF
    Some(OpCode(CPY, Immediate)),       // $C0    CPY #$NN     Immediate
    Some(OpCode(CMP, IndexedIndirect)), // $C1    CMP ($NN,X)  IndexedIndirect
    None,                               // $C2
    None,                               // $C3
    Some(OpCode(CPY, ZeroPage)),        // $C4    CPY $NN      ZeroPage
    Some(OpCode(CMP, ZeroPage)),        // $C5    CMP $NN      ZeroPage
    Some(OpCode(DEC, ZeroPage)),        // $C6    DEC $NN      ZeroPage
    None,                               // $C7
    Some(OpCode(INY, Implied)),         // $C8    INY          Implied
    Some(OpCode(CMP, Immediate)),       // $C9    CMP #$NN     Immediate
    Some(OpCode(DEX, Implied)),         // $CA    DEX          Implied
    None,                               // $CB
    Some(OpCode(CPY, Absolute)),        // $CC    CPY $NNNN    Absolute
    Some(OpCode(CMP, Absolute)),        // $CD    CMP $NNNN    Absolute
    Some(OpCode(DEC, Absolute)),        // $CE    DEC $NNNN    Absolute
    None,                               // $CF
    Some(OpCode(BNE, Relative)),        // $D0    BNE $NN      Relative
    Some(OpCode(CMP, IndirectIndexed)), // $D1    CMP ($NN),Y  IndirectIndexed
    None,                               // $D2
    None,                               // $D3
    None,                               // $D4
    Some(OpCode(CMP, ZeroPageX)),       // $D5    CMP $NN,X    ZeroPageX
    Some(OpCode(DEC, ZeroPageX)),       // $D6    DEC $NN,X    ZeroPageX
    None,                               // $D7
    Some(OpCode(CLD, Implied)),         // $D8    CLD          Implied
    Some(OpCode(CMP, AbsoluteY)),       // $D9    CMP $NNNN,Y  AbsoluteY
    None,                               // $DA
    None,                               // $DB
    None,                               // $DC
    Some(OpCode(CMP, AbsoluteX)),       // $DD    CMP $NNNN,X  AbsoluteX
    Some(OpCode(DEC, AbsoluteX)),       // $DE    DEC $NNNN,X  AbsoluteX
    None,                               // $DF
    Some(OpCode(CPX, Immediate)),       // $E0    CPX #$NN     Immediate
    Some(OpCode(SBC, IndexedIndirect)), // $E1    SBC ($NN,X)  IndexedIndirect
    None,                               // $E2
    None,                               // $E3
    Some(OpCode(CPX, ZeroPage)),        // $E4    CPX $NN      ZeroPage
    Some(OpCode(SBC, ZeroPage)),        // $E5    SBC $NN      ZeroPage
    Some(OpCode(INC, ZeroPage)),        // $E6    INC $NN      ZeroPage
    None,                               // $E7
    Some(OpCode(INX, Implied)),         // $E8    INX          Implied
    Some(OpCode(SBC, Immediate)),       // $E9    SBC #$NN     Immediate
    Some(OpCode(NOP, Implied)),         // $EA    NOP          Implied
    None,                               // $EB
    Some(OpCode(CPX, Absolute)),        // $EC    CPX $NNNN    Absolute
    Some(OpCode(SBC, Absolute)),        // $ED    SBC $NNNN    Absolute
    Some(OpCode(INC, Absolute)),        // $EE    INC $NNNN    Absolute
    None,                               // $EF
    Some(OpCode(BEQ, Relative)),        // $F0    BEQ $NN      Relative
    Some(OpCode(SBC, IndirectIndexed)), // $F1    SBC ($NN),Y  IndirectIndexed
    None,                               // $F2
    None,                               // $F3
    None,                               // $F4
    Some(OpCode(SBC, ZeroPageX)),       // $F5    SBC $NN,X    ZeroPageX
    Some(OpCode(INC, ZeroPageX)),       // $F6    INC $NN,X    ZeroPageX
    None,                               // $F7
    Some(OpCode(SED, Implied)),         // $F8    SED          Implied
    Some(OpCode(SBC, AbsoluteY)),       // $F9    SBC $NNNN,Y  AbsoluteY
    None,                               // $FA
    None,                               // $FB
    None,                               // $FC
    Some(OpCode(SBC, AbsoluteX)),       // $FD    SBC $NNNN,X  AbsoluteX
    Some(OpCode(INC, AbsoluteX)),       // $FE    INC $NNNN,X  AbsoluteX
    None,                               // $FF
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
        OpCode(Instruction::LDA, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDA, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDA, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::LDX, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDX, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDX, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::LDY, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDY, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDY, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::STA, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_stx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.x = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STX, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_sty() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.y = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STY, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x42;
        cpu.x = 0;
        OpCode(Instruction::TAX, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_tay() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0x42;
        cpu.y = 0;
        OpCode(Instruction::TAY, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0x42);
    }

    #[test]
    fn test_txa() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0x42;
        cpu.a = 0;
        OpCode(Instruction::TXA, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tya() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.y = 0x42;
        cpu.a = 0;
        OpCode(Instruction::TYA, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tsx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0x42;
        cpu.x = 0;
        OpCode(Instruction::TSX, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_txs() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0x42;
        cpu.sp = 0;
        OpCode(Instruction::TXS, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.sp, 0x42);
    }

    #[test]
    fn test_pha() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0xFF;
        cpu.pc = 0x8000;
        cpu.a = 0x42;
        OpCode(Instruction::PHA, AddressingMode::Implied).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::PLA, AddressingMode::Implied).execute(&mut cpu, &mut ram);
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

        OpCode(Instruction::PHP, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(ram[0x1FF], 0b00100001);
    }

    #[test]
    fn test_plp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.sp = 0xFE;
        cpu.flags.c = false;
        cpu.flags.r = false;
        ram[0x1FF] = 0b00100001;

        OpCode(Instruction::PLP, AddressingMode::Implied).execute(&mut cpu, &mut ram);
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

        OpCode(Instruction::AND, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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

        OpCode(Instruction::EOR, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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

        OpCode(Instruction::ORA, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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

        OpCode(Instruction::BIT, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.v, false);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        cpu.a = 0;
        ram[0x1] = 0b11000000;
        ram[0x8000] = 0x1;

        OpCode(Instruction::BIT, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::ADC, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x30);
        assert_eq!(cpu.flags.c, false);

        cpu.a = 0xFF;
        cpu.pc = 0x8000;
        cpu.flags.c = true;
        ram[0x8000] = 1;
        OpCode(Instruction::ADC, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::SBC, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0x20);
        assert_eq!(cpu.flags.c, true);

        cpu.a = 0x00;
        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 1;
        OpCode(Instruction::SBC, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::CMP, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.a = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CMP, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::CPX, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.x = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CPX, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::CPY, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.y = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CPY, AddressingMode::Immediate).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::INC, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0xFF;
        OpCode(Instruction::INC, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_inx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0xFE;
        OpCode(Instruction::INX, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.x = 0xFF;
        OpCode(Instruction::INX, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.y = 0xFE;
        OpCode(Instruction::INY, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.y = 0xFF;
        OpCode(Instruction::INY, AddressingMode::Implied).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::DEC, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0x00;
        OpCode(Instruction::DEC, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x00], 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_dex() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.x = 0x01;
        OpCode(Instruction::DEX, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0x00);

        cpu.x = 0x00;
        OpCode(Instruction::DEX, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.x, 0xFF);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.y = 0x01;
        OpCode(Instruction::DEY, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0x00);

        cpu.y = 0x00;
        OpCode(Instruction::DEY, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.y, 0xFF);
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b10111111;
        OpCode(Instruction::ASL, AddressingMode::Accumulator).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b01111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b01000000;
        OpCode(Instruction::ASL, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01], 0b10000000);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_lsr() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b11111101;
        OpCode(Instruction::LSR, AddressingMode::Accumulator).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b01111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b00000010;
        OpCode(Instruction::LSR, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01], 0b00000001);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b10111111;
        cpu.flags.c = true;
        OpCode(Instruction::ROL, AddressingMode::Accumulator).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b01111111);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b01000000;
        OpCode(Instruction::ROL, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01], 0b10000000);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_ror() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.a = 0b11111101;
        cpu.flags.c = true;
        OpCode(Instruction::ROR, AddressingMode::Accumulator).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.a, 0b11111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b00000010;
        OpCode(Instruction::ROR, AddressingMode::ZeroPage).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::JMP, AddressingMode::Absolute).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x0102);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x02;
        ram[0x8001] = 0x01;
        ram[0x0102] = 0x04;
        ram[0x0103] = 0x03;
        OpCode(Instruction::JMP, AddressingMode::Indirect).execute(&mut cpu, &mut ram);
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
        OpCode(Instruction::JSR, AddressingMode::Absolute).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(ram[0x01FF], 0x02);
        assert_eq!(ram[0x01FE], 0x80);
    }

    #[test]
    fn test_rts() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.sp = 0xFD;
        ram[0x01FE] = 0x01;
        ram[0x01FF] = 0x02;
        OpCode(Instruction::RTS, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x0103);
    }

    #[test]
    fn test_bcc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.c = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCC, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.c = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCC, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bcs() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.c = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCS, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.c = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BCS, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bne() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.z = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BNE, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.z = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BNE, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_beq() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.z = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BEQ, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.z = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BEQ, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bpl() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.n = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BPL, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.n = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BPL, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bmi() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.n = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BMI, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.n = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BMI, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bvc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.v = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVC, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.v = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVC, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_bvs() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8001;
        cpu.flags.v = true;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVS, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8004);

        cpu.pc = 0x8001;
        cpu.flags.v = false;
        ram[0x8001] = 0x02_i8 as u8;
        OpCode(Instruction::BVS, AddressingMode::Relative).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.pc, 0x8002);
    }

    #[test]
    fn test_clc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.c = true;
        OpCode(Instruction::CLC, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_cld() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.d = true;
        OpCode(Instruction::CLD, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.d, false);
    }

    #[test]
    fn test_cli() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.i = true;
        OpCode(Instruction::CLI, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.i, false);
    }

    #[test]
    fn test_clv() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.v = true;
        OpCode(Instruction::CLV, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.v, false);
    }

    #[test]
    fn test_sec() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.c = false;
        OpCode(Instruction::SEC, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
    }

    #[test]
    fn test_sed() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.d = false;
        OpCode(Instruction::SED, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.d, true);
    }

    #[test]
    fn test_sei() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.flags.i = false;
        OpCode(Instruction::SEI, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.i, true);
    }

    #[test]
    fn test_brk() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        cpu.pc = 0x8000;
        cpu.sp = 0xFF;
        OpCode(Instruction::BRK, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(ram[0x01FF], 0x00);
        assert_eq!(ram[0x01FE], 0x80);
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
        ram[0x01FE] = 0x90;
        ram[0x01FF] = 0x00;
        OpCode(Instruction::RTI, AddressingMode::Implied).execute(&mut cpu, &mut ram);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.i, false);
        assert_eq!(cpu.flags.b, false);
        assert_eq!(cpu.pc, 0x9001);
    }

    #[test]
    fn test_nop() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();

        OpCode(Instruction::NOP, AddressingMode::Implied).execute(&mut cpu, &mut ram);
    }
}
