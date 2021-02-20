use std::usize;

use crate::cpu::CPU;
use crate::ram::RAM;

#[derive(Debug)]
enum Instruction {
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

    NOP,
}

#[derive(Debug)]
enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    // Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

impl AddressingMode {
    fn fetch(&self, cpu: &mut CPU, cycles: &mut isize, ram: &mut RAM) -> Option<u8> {
        match self {
            Accumulator => Some(cpu.a),
            Immediate => Some(cpu.fetch_byte(cycles, ram)),
            ZeroPage => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            ZeroPageX => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            ZeroPageY => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            Absolute => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            AbsoluteX => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            AbsoluteY => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            IndexedIndirect => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            IndirectIndexed => {
                let addr = self.get_address(cpu, cycles, ram).unwrap();
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            Implied => panic!("You can't call fetch from {:?}!", self),
            Indirect => panic!("You can't call fetch from {:?}!", self),
        }
    }

    fn get_address(&self, cpu: &mut CPU, cycles: &mut isize, ram: &mut RAM) -> Option<u16> {
        match self {
            ZeroPage => Some(cpu.fetch_byte(cycles, ram).into()),
            ZeroPageX => {
                *cycles -= 1; // may be consumed by add x
                Some((cpu.fetch_byte(cycles, ram) + cpu.x).into())
            }
            ZeroPageY => {
                *cycles -= 1; // may be consumed by add y
                Some((cpu.fetch_byte(cycles, ram) + cpu.y).into())
            }
            Absolute => {
                let addr = cpu.fetch_byte(cycles, ram) as u16
                    + ((cpu.fetch_byte(cycles, ram) as u16) << 8);

                Some(addr)
            }
            AbsoluteX => {
                let addr = cpu.fetch_byte(cycles, ram) as u16
                    + ((cpu.fetch_byte(cycles, ram) as u16) << 8)
                    + cpu.x as u16;
                Some(addr)
            }
            AbsoluteY => {
                let addr = cpu.fetch_byte(cycles, ram) as u16
                    + ((cpu.fetch_byte(cycles, ram) as u16) << 8)
                    + cpu.y as u16;
                Some(addr)
            }
            Indirect => {
                let ind_addr = cpu.fetch_byte(cycles, ram) as u16
                    + ((cpu.fetch_byte(cycles, ram) as u16) << 8);
                let addr = cpu.read_byte(cycles, ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(cycles, ram, (ind_addr + 1) as usize) as u16) << 8);
                Some(addr)
            }
            IndexedIndirect => {
                let ind_addr = cpu.fetch_byte(cycles, ram) as u16 + cpu.x as u16;
                let addr = cpu.read_byte(cycles, ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(cycles, ram, (ind_addr + 1) as usize) as u16) << 8);
                Some(addr)
            }
            IndirectIndexed => {
                let ind_addr = cpu.fetch_byte(cycles, ram) as u16;
                let addr = cpu.read_byte(cycles, ram, ind_addr as usize) as u16
                    + ((cpu.read_byte(cycles, ram, (ind_addr + 1) as usize) as u16) << 8)
                    + cpu.y as u16;
                Some(addr)
            }
            Accumulator => panic!("You can't call get_address from {:?}!", self),
            Implied => panic!("You can't call get_address from {:?}!", self),
            Immediate => panic!("You can't call get_address from {:?}!", self),
        }
    }
}

pub struct OpCode(Instruction, AddressingMode);

impl OpCode {
    pub fn execute(&self, cpu: &mut CPU, cycles: &mut isize, ram: &mut RAM) {
        let ins = &self.0;
        let adr_mode = &self.1;
        println!("instruction: {:?}", ins);
        println!("adr_mode:    {:?}", adr_mode);
        match ins {
            LDA => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.set_accumulator(byte);
            }
            LDX => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.set_index_x(byte);
            }
            LDY => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.set_index_y(byte);
            }
            STA => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                cpu.write_byte(cycles, ram, addr as usize, cpu.a);
            }
            STX => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                cpu.write_byte(cycles, ram, addr as usize, cpu.x);
            }
            STY => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                cpu.write_byte(cycles, ram, addr as usize, cpu.y);
            }
            TAX => {
                cpu.set_index_x(cpu.a);
                *cycles -= 1;
            }
            TAY => {
                cpu.set_index_y(cpu.a);
                *cycles -= 1;
            }
            TXA => {
                cpu.set_accumulator(cpu.x);
                *cycles -= 1;
            }
            TYA => {
                cpu.set_accumulator(cpu.y);
                *cycles -= 1;
            }
            TSX => {
                cpu.set_index_x(cpu.sp);
                *cycles -= 1;
            }
            TXS => {
                cpu.sp = cpu.x;
                *cycles -= 1;
            }
            PHA => {
                cpu.push_to_stack(cycles, ram, cpu.a);
            }
            PLA => {
                let byte = cpu.pull_from_stack(cycles, ram);
                cpu.set_accumulator(byte);
                *cycles -= 1;
            }
            PHP => {
                let byte = cpu.flags.get_as_u8();
                cpu.push_to_stack(cycles, ram, byte);
            }
            PLP => {
                let byte = cpu.pull_from_stack(cycles, ram);
                cpu.flags.set_as_u8(byte);
                *cycles -= 1;
            }
            AND => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.set_accumulator(cpu.a & byte);
            }
            EOR => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.set_accumulator(cpu.a ^ byte);
            }
            ORA => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.set_accumulator(cpu.a | byte);
            }
            BIT => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.flags.z = cpu.a & byte == 0;
                cpu.flags.v = (byte >> 5 & 1) == 1;
                cpu.flags.n = (byte >> 6 & 1) == 1;
            }
            ADC => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                let (byte, overflowing1) = cpu.a.overflowing_add(byte);
                let (byte, overflowing2) = byte.overflowing_add(cpu.flags.c as u8);
                cpu.flags.c = overflowing1 || overflowing2;
                cpu.set_accumulator(byte);
            }
            SBC => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                let (byte, overflowing1) = cpu.a.overflowing_sub(byte);
                let (byte, overflowing2) = byte.overflowing_sub(!cpu.flags.c as u8);
                cpu.flags.c = !(overflowing1 || overflowing2);
                cpu.set_accumulator(byte);
            }
            CMP => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.flags.c = cpu.a >= byte;
                cpu.flags.z = cpu.a == byte;
                cpu.flags.n = cpu.a.wrapping_sub(byte) >> 7 & 1 == 1;
            }
            CPX => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.flags.c = cpu.x >= byte;
                cpu.flags.z = cpu.x == byte;
                cpu.flags.n = cpu.x.wrapping_sub(byte) >> 7 & 1 == 1;
            }
            CPY => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.flags.c = cpu.y >= byte;
                cpu.flags.z = cpu.y == byte;
                cpu.flags.n = cpu.y.wrapping_sub(byte) >> 7 & 1 == 1;
            }
            INC => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                let byte = cpu.read_byte(cycles, ram, addr as usize);
                let byte = byte.wrapping_add(1);
                cpu.set_zero_and_negative_flag(byte);
                cpu.write_byte(cycles, ram, addr as usize, byte);
            }
            INX => {
                let byte = cpu.x;
                let byte = byte.wrapping_add(1);
                cpu.set_index_x(byte);
            }
            INY => {
                let byte = cpu.y;
                let byte = byte.wrapping_add(1);
                cpu.set_index_y(byte);
            }
            DEC => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                let byte = cpu.read_byte(cycles, ram, addr as usize);
                let byte = byte.wrapping_sub(1);
                cpu.set_zero_and_negative_flag(byte);
                cpu.write_byte(cycles, ram, addr as usize, byte);
            }
            DEX => {
                let byte = cpu.x;
                let byte = byte.wrapping_sub(1);
                cpu.set_index_x(byte);
            }
            DEY => {
                let byte = cpu.y;
                let byte = byte.wrapping_sub(1);
                cpu.set_index_y(byte);
            }
            ASL => {
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = byte << 1;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                    let byte = cpu.read_byte(cycles, ram, addr as usize);
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = byte << 1;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(cycles, ram, addr as usize, byte);
                }
            }
            LSR => {
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = byte >> 1;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                    let byte = cpu.read_byte(cycles, ram, addr as usize);
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = byte >> 1;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(cycles, ram, addr as usize, byte);
                }
            }
            ROL => {
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                    let new_first_byte = cpu.flags.c as u8;
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = (byte << 1) | new_first_byte;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                    let byte = cpu.read_byte(cycles, ram, addr as usize);
                    let new_first_byte = cpu.flags.c as u8;
                    cpu.flags.c = byte >> 7 & 1 == 1; // old 7 bit
                    let byte = (byte << 1) | new_first_byte;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(cycles, ram, addr as usize, byte);
                }
            }
            ROR => {
                if let Accumulator = adr_mode {
                    let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                    let new_last_byte = (cpu.flags.c as u8) << 7;
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = (byte >> 1) | new_last_byte;
                    cpu.set_accumulator(byte);
                } else {
                    let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                    let byte = cpu.read_byte(cycles, ram, addr as usize);
                    let new_last_byte = (cpu.flags.c as u8) << 7;
                    cpu.flags.c = byte >> 0 & 1 == 1; // old 0 bit
                    let byte = (byte >> 1) | new_last_byte;
                    cpu.set_zero_and_negative_flag(byte);
                    cpu.write_byte(cycles, ram, addr as usize, byte);
                }
            }
            JMP => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                cpu.pc = addr;
            }
            JSR => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                let pc = cpu.pc - 1;
                cpu.push_to_stack(cycles, ram, (pc & 0xFF) as u8);
                cpu.push_to_stack(cycles, ram, (pc >> 8) as u8);
                cpu.pc = addr;
            }
            NOP => {
                *cycles -= 1;
                println!("nop");
            }
        }
    }
}

use AddressingMode::*;
use Instruction::*;
#[allow(dead_code)]
pub const OPCODES: [Option<OpCode>; 0x100] = [
    None,                               // $00    BRK	         Implied
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
    None,                               // $10    BPL $NN      Relative
    Some(OpCode(ORA, IndirectIndexed)), // $11    ORA ($NN),Y  IndirectIndexed
    None,                               // $12
    None,                               // $13
    None,                               // $14
    Some(OpCode(ORA, ZeroPageX)),       // $15    ORA $NN,X    ZeroPageX
    Some(OpCode(ASL, ZeroPageX)),       // $16    ASL $NN,X    ZeroPageX
    None,                               // $17
    None,                               // $18    CLC          Implied
    Some(OpCode(ORA, AbsoluteY)),       // $19    ORA $NNNN,Y  AbsoluteY
    None,                               // $1A
    None,                               // $1B
    None,                               // $1C
    Some(OpCode(ORA, AbsoluteX)),       // $1D    ORA $NNNN,X  AbsoluteX
    Some(OpCode(ASL, AbsoluteX)),       // $1E    ASL $NNNN,X  AbsoluteX
    None,                               // $1F
    Some(OpCode(JSR, IndexedIndirect)), // $20    JSR $NNNN    Absolute
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
    None,                               // $30    BMI $NN      Relative
    Some(OpCode(AND, IndirectIndexed)), // $31    AND ($NN),Y  IndirectIndexed
    None,                               // $32
    None,                               // $33
    None,                               // $34
    Some(OpCode(AND, ZeroPageX)),       // $35    AND $NN,X    ZeroPageX
    Some(OpCode(ROL, ZeroPageX)),       // $36    ROL $NN,X    ZeroPageX
    None,                               // $37
    None,                               // $38    SEC          Implied
    Some(OpCode(AND, AbsoluteY)),       // $39    AND $NNNN,Y  AbsoluteY
    None,                               // $3A
    None,                               // $3B
    None,                               // $3C
    Some(OpCode(AND, AbsoluteX)),       // $3D    AND $NNNN,X  AbsoluteX
    Some(OpCode(ROL, AbsoluteX)),       // $3E    ROL $NNNN,X  AbsoluteX
    None,                               // $3F
    None,                               // $40    RTI          Implied
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
    None,                               // $50    BVC $NN      Relative
    Some(OpCode(EOR, IndirectIndexed)), // $51    EOR ($NN),Y  IndirectIndexed
    None,                               // $52
    None,                               // $53
    None,                               // $54
    Some(OpCode(EOR, ZeroPageX)),       // $55    EOR $NN,X    ZeroPageX
    Some(OpCode(LSR, ZeroPageX)),       // $56    LSR $NN,X    ZeroPageX
    None,                               // $57
    None,                               // $58    CLI          Implied
    Some(OpCode(EOR, AbsoluteY)),       // $59    EOR $NNNN,Y  AbsoluteY
    None,                               // $5A
    None,                               // $5B
    None,                               // $5C
    Some(OpCode(EOR, AbsoluteX)),       // $5D    EOR $NNNN,X  AbsoluteX
    Some(OpCode(LSR, AbsoluteX)),       // $5E    LSR $NNNN,X  AbsoluteX
    None,                               // $5F
    None,                               // $60    RTS          Implied
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
    None,                               // $70    BVS $NN      Relative
    Some(OpCode(ADC, IndirectIndexed)), // $71    ADC ($NN),Y  IndirectIndexed
    None,                               // $72
    None,                               // $73
    None,                               // $74
    Some(OpCode(ADC, ZeroPageX)),       // $75    ADC $NN,X    ZeroPageX
    Some(OpCode(ROR, ZeroPageX)),       // $76    ROR $NN,X    ZeroPageX
    None,                               // $77
    None,                               // $78    SEI          Implied
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
    None,                               // $90    BCC $NN      Relative
    Some(OpCode(STA, IndirectIndexed)), // $91    STA ($NN),Y  IndirectIndexed
    None,                               // $92
    None,                               // $93
    Some(OpCode(STY, ZeroPageX)),       // $94    STY $NN,X    ZeroPageX
    Some(OpCode(STA, ZeroPageY)),       // $95    STA $NN,X    ZeroPageX
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
    None,                               // $B0    BCS $NN      Relative
    Some(OpCode(LDA, IndirectIndexed)), // $B1    LDA ($NN),Y  IndirectIndexed
    None,                               // $B2
    None,                               // $B3
    Some(OpCode(LDY, ZeroPageX)),       // $B4    LDY $NN,X    ZeroPageX
    Some(OpCode(LDA, ZeroPageX)),       // $B5    LDA $NN,X    ZeroPageX
    Some(OpCode(LDX, ZeroPageY)),       // $B6    LDX $NN,Y    ZeroPageY
    None,                               // $B7
    None,                               // $B8    CLV          Implied
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
    None,                               // $D0    BNE $NN      Relative
    Some(OpCode(CMP, IndirectIndexed)), // $D1    CMP ($NN),Y  IndirectIndexed
    None,                               // $D2
    None,                               // $D3
    None,                               // $D4
    Some(OpCode(CMP, ZeroPageX)),       // $D5    CMP $NN,X    ZeroPageX
    Some(OpCode(DEC, ZeroPageX)),       // $D6    DEC $NN,X    ZeroPageX
    None,                               // $D7
    None,                               // $D8    CLD          Implied
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
    None,                               // $F0    BEQ $NN      Relative
    Some(OpCode(SBC, IndirectIndexed)), // $F1    SBC ($NN),Y  IndirectIndexed
    None,                               // $F2
    None,                               // $F3
    None,                               // $F4
    Some(OpCode(SBC, ZeroPageX)),       // $F5    SBC $NN,X    ZeroPageX
    Some(OpCode(INC, ZeroPageX)),       // $F6    INC $NN,X    ZeroPageX
    None,                               // $F7
    None,                               // $F8    SED          Implied
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
    use super::*;

    #[test]
    fn test_accumulator() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0x42;
        let byte = AddressingMode::Accumulator.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));
    }

    #[test]
    fn test_immediate() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0x42;
        let byte = AddressingMode::Immediate.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));
    }

    #[test]
    fn test_zero_page() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x10] = 0x42;
        ram[0x8000] = 0x10;
        let byte = AddressingMode::ZeroPage.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::ZeroPage.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(addr, Some(0x10));
    }

    #[test]
    fn test_zero_page_x() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.x = 2;
        ram[0x12] = 0x42;
        ram[0x8000] = 0x10;
        let byte = AddressingMode::ZeroPageX.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::ZeroPageX.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(addr, Some(0x12));
    }

    #[test]
    fn test_zero_page_y() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.y = 2;
        ram[0x12] = 0x42;
        ram[0x8000] = 0x10;
        let byte = AddressingMode::ZeroPageY.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::ZeroPageY.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(addr, Some(0x12));
    }

    #[test]
    fn test_absolute() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x8001] = 0x01;
        ram[0x0100] = 0x42;
        let byte = AddressingMode::Absolute.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::Absolute.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(addr, Some(0x0100));
    }

    #[test]
    fn test_absolute_x() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x00;
        ram[0x8001] = 0x01;
        ram[0x0101] = 0x42;
        let byte = AddressingMode::AbsoluteX.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::AbsoluteX.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(addr, Some(0x0101));
    }

    #[test]
    fn test_absolute_y() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.y = 1;
        ram[0x8000] = 0x00;
        ram[0x8001] = 0x01;
        ram[0x0101] = 0x42;
        let byte = AddressingMode::AbsoluteY.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));

        cpu.pc = 0x8000;
        let addr = AddressingMode::AbsoluteY.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(addr, Some(0x0101));
    }

    #[test]
    fn test_indirect() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0x02;
        ram[0x8001] = 0x01;
        ram[0x0102] = 0x04;
        ram[0x0103] = 0x03;
        let byte = AddressingMode::Indirect.get_address(&mut cpu, &mut cycles, &mut ram);

        assert_eq!(byte, Some(0x0304));
    }

    #[test]
    fn test_indexed_indirect() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x00;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        let byte = AddressingMode::IndexedIndirect.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x0304));

        cpu.pc = 0x8000;
        cpu.x = 1;
        ram[0x8000] = 0x00;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        ram[0x0304] = 0x42;
        let byte = AddressingMode::IndexedIndirect.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));
    }

    #[test]
    fn test_indirect_indexed() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.y = 1;
        ram[0x8000] = 0x01;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        let byte = AddressingMode::IndirectIndexed.get_address(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x0305));

        cpu.pc = 0x8000;
        cpu.y = 1;
        ram[0x8000] = 0x01;
        ram[0x01] = 0x04;
        ram[0x02] = 0x03;
        ram[0x0305] = 0x42;
        let byte = AddressingMode::IndirectIndexed.fetch(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(byte, Some(0x42));
    }
}

#[cfg(test)]
mod test_instructions {
    use super::*;

    #[test]
    fn test_lda() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0b10000010;
        OpCode(Instruction::LDA, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDA, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDA, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_ldx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0b10000010;
        OpCode(Instruction::LDX, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.x, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDX, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDX, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.x, 1);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_ldy() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0b10000010;
        OpCode(Instruction::LDY, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.y, 0b10000010);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0;
        OpCode(Instruction::LDY, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 1;
        OpCode(Instruction::LDY, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.y, 1);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.a = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STA, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_stx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.x = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STX, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_sty() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.y = 0x42;
        ram[0x8000] = 0x0;
        OpCode(Instruction::STY, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x0], 0x42);
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0x42;
        cpu.x = 0;
        OpCode(Instruction::TAX, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_tay() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0x42;
        cpu.y = 0;
        OpCode(Instruction::TAY, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.y, 0x42);
    }

    #[test]
    fn test_txa() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.x = 0x42;
        cpu.a = 0;
        OpCode(Instruction::TXA, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tya() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.y = 0x42;
        cpu.a = 0;
        OpCode(Instruction::TYA, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_tsx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.sp = 0x42;
        cpu.x = 0;
        OpCode(Instruction::TSX, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.x, 0x42);
    }

    #[test]
    fn test_txs() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.x = 0x42;
        cpu.sp = 0;
        OpCode(Instruction::TXS, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.sp, 0x42);
    }

    #[test]
    fn test_pha() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.sp = 0xFF;
        cpu.pc = 0x8000;
        cpu.a = 0x42;
        OpCode(Instruction::PHA, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(ram[0x1FF], 0x42);
    }

    #[test]
    fn test_pla() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.sp = 0xFE;
        cpu.pc = 0x8000;
        cpu.a = 0;
        ram[0x1FF] = 0x42;
        OpCode(Instruction::PLA, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_php() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.sp = 0xFF;
        cpu.flags.c = true;
        cpu.flags.r = true;

        OpCode(Instruction::PHP, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(ram[0x1FF], 0b00100001);
    }

    #[test]
    fn test_plp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.sp = 0xFE;
        cpu.flags.c = false;
        cpu.flags.r = false;
        ram[0x1FF] = 0b00100001;

        OpCode(Instruction::PLP, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.r, true);
    }

    #[test]
    fn test_and() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.a = 0b00011000;
        ram[0x8000] = 0b00001111;

        OpCode(Instruction::AND, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b00001000);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_eor() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.a = 0b00001111;
        ram[0x8000] = 0b00001000;

        OpCode(Instruction::EOR, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b00000111);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.a = 0b00001111;
        ram[0x8000] = 0b11110000;

        OpCode(Instruction::ORA, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b11111111);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_bit() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        cpu.a = 0;
        ram[0x1] = 0;
        ram[0x8000] = 0x1;

        OpCode(Instruction::BIT, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.v, false);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        cpu.a = 0;
        ram[0x1] = 0b01100000;
        ram[0x8000] = 0x1;

        OpCode(Instruction::BIT, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.v, true);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_adc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0x20;
        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x10;
        OpCode(Instruction::ADC, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0x30);
        assert_eq!(cpu.flags.c, false);

        cpu.a = 0xFF;
        cpu.pc = 0x8000;
        cpu.flags.c = true;
        ram[0x8000] = 1;
        OpCode(Instruction::ADC, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 1);
        assert_eq!(cpu.flags.c, true);
    }

    #[test]
    fn test_sbc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0x30;
        cpu.pc = 0x8000;
        cpu.flags.c = true;
        ram[0x8000] = 0x10;
        OpCode(Instruction::SBC, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0x20);
        assert_eq!(cpu.flags.c, true);

        cpu.a = 0x00;
        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 1;
        OpCode(Instruction::SBC, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_cmp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x10;
        OpCode(Instruction::CMP, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.a = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CMP, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.flags.c, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_cpx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.x = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x10;
        OpCode(Instruction::CPX, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.x = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CPX, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.flags.c, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_cpy() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.y = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x10;
        OpCode(Instruction::CPY, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.flags.c, true);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.y = 0x10;
        cpu.pc = 0x8000;
        ram[0x8000] = 0x20;
        OpCode(Instruction::CPY, AddressingMode::Immediate).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.flags.c, false);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0xFE;
        OpCode(Instruction::INC, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x00], 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0xFF;
        OpCode(Instruction::INC, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x00], 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_inx() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.x = 0xFE;
        OpCode(Instruction::INX, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.x, 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.x = 0xFF;
        OpCode(Instruction::INX, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.y = 0xFE;
        OpCode(Instruction::INY, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.y, 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);

        cpu.y = 0xFF;
        OpCode(Instruction::INY, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);
    }

    #[test]
    fn test_dec() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0x01;
        OpCode(Instruction::DEC, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x00], 0x00);
        assert_eq!(cpu.flags.z, true);
        assert_eq!(cpu.flags.n, false);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x00;
        ram[0x00] = 0x00;
        OpCode(Instruction::DEC, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x00], 0xFF);
        assert_eq!(cpu.flags.z, false);
        assert_eq!(cpu.flags.n, true);
    }

    #[test]
    fn test_dex() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.x = 0x01;
        OpCode(Instruction::DEX, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.x, 0x00);

        cpu.x = 0x00;
        OpCode(Instruction::DEX, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.x, 0xFF);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.y = 0x01;
        OpCode(Instruction::DEY, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.y, 0x00);

        cpu.y = 0x00;
        OpCode(Instruction::DEY, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.y, 0xFF);
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0b10111111;
        OpCode(Instruction::ASL, AddressingMode::Accumulator).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b01111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b01000000;
        OpCode(Instruction::ASL, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x01], 0b10000000);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_lsr() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0b11111101;
        OpCode(Instruction::LSR, AddressingMode::Accumulator).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b01111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b00000010;
        OpCode(Instruction::LSR, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x01], 0b00000001);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0b10111111;
        cpu.flags.c = true;
        OpCode(Instruction::ROL, AddressingMode::Accumulator).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b01111111);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b01000000;
        OpCode(Instruction::ROL, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x01], 0b10000000);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_ror() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.a = 0b11111101;
        cpu.flags.c = true;
        OpCode(Instruction::ROR, AddressingMode::Accumulator).execute(
            &mut cpu,
            &mut cycles,
            &mut ram,
        );
        assert_eq!(cpu.a, 0b11111110);
        assert_eq!(cpu.flags.c, true);

        cpu.pc = 0x8000;
        cpu.flags.c = false;
        ram[0x8000] = 0x01;
        ram[0x01] = 0b00000010;
        OpCode(Instruction::ROR, AddressingMode::ZeroPage).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(ram[0x01], 0b00000001);
        assert_eq!(cpu.flags.c, false);
    }

    #[test]
    fn test_jmp() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8000;
        ram[0x8000] = 0x02;
        ram[0x8001] = 0x01;
        OpCode(Instruction::JMP, AddressingMode::Absolute).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.pc, 0x0102);

        cpu.pc = 0x8000;
        ram[0x8000] = 0x02;
        ram[0x8001] = 0x01;
        ram[0x0102] = 0x04;
        ram[0x0103] = 0x03;
        OpCode(Instruction::JMP, AddressingMode::Indirect).execute(&mut cpu, &mut cycles, &mut ram);
        assert_eq!(cpu.pc, 0x0304);
    }

    #[test]
    fn test_jsr() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        cpu.pc = 0x8001;
        cpu.sp = 0xFF;
        ram[0x8001] = 0x02;
        ram[0x8002] = 0x01;
        OpCode(Instruction::JSR, AddressingMode::Absolute).execute(&mut cpu, &mut cycles, &mut ram);
        println!("ram[0x01FF]: {}", ram[0x01FF]);
        println!("ram[0x01FE]: {}", ram[0x01FE]);
        assert_eq!(cpu.pc, 0x0102);
        assert_eq!(ram[0x01FF], 0x02);
        assert_eq!(ram[0x01FE], 0x80);
    }

    #[test]
    fn test_nop() {
        let mut cpu = CPU::default();
        let mut ram = RAM::default();
        let mut cycles = 999;

        OpCode(Instruction::NOP, AddressingMode::Implied).execute(&mut cpu, &mut cycles, &mut ram);
    }
}
