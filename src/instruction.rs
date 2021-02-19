use crate::{CPU, RAM};
#[derive(Debug)]
enum Instruction {
    LDA,
    LDX,
    LDY,
    STA,

    NOP,
}

#[derive(Debug)]
enum AdressingMode {
    Implied,
    // Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    // Relative,
    Absolute,
    // AbsoluteX,
    // AbsoluteY,
    // Indirect,
    // IndexedIndirect,
    // IndirectIndexed,
}

impl AdressingMode {
    fn fetch(&self, cpu: &mut CPU, cycles: &mut isize, ram: &mut RAM) -> Option<u8> {
        match self {
            Implied => None,
            Immediate => Some(cpu.fetch_byte(cycles, ram)),
            ZeroPage => {
                let addr = cpu.fetch_byte(cycles, ram);
                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
            ZeroPageX => {
                let addr = cpu.fetch_byte(cycles, ram);
                *cycles -= 1; // may be consumed by add x
                Some(cpu.read_byte(cycles, ram, (addr + cpu.x) as usize))
            }
            ZeroPageY => {
                let addr = cpu.fetch_byte(cycles, ram);
                *cycles -= 1; // may be consumed by add x
                Some(cpu.read_byte(cycles, ram, (addr + cpu.y) as usize))
            }
            Absolute => {
                let addr = cpu.fetch_byte(cycles, ram) as u16
                    + ((cpu.fetch_byte(cycles, ram) as u16) << 8);

                Some(cpu.read_byte(cycles, ram, addr as usize))
            }
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
            _ => panic!("You can't call get_address from {:?}!", self),
        }
    }
}

pub struct OpCode(Instruction, AdressingMode);

impl OpCode {
    pub fn execute(&self, cpu: &mut CPU, cycles: &mut isize, ram: &mut RAM) {
        let ins = &self.0;
        let adr_mode = &self.1;
        println!("instruction: {:?}", ins);
        println!("adr_mode:    {:?}", adr_mode);
        match ins {
            LDA => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.a = byte;
                cpu.flags.z = byte == 0;
                cpu.flags.n = byte >> 6 & 1 == 1;
            }
            LDX => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.x = byte;
                cpu.flags.z = byte == 0;
                cpu.flags.n = byte >> 6 & 1 == 1;
            }
            LDY => {
                let byte = adr_mode.fetch(cpu, cycles, ram).unwrap();
                cpu.y = byte;
                cpu.flags.z = byte == 0;
                cpu.flags.n = byte >> 6 & 1 == 1;
            }
            STA => {
                let addr = adr_mode.get_address(cpu, cycles, ram).unwrap();
                cpu.write_byte(cycles, ram, addr as usize, cpu.a);
            }
            NOP => {
                *cycles -= 1;
                println!("nop");
            }
        }
    }
}

use AdressingMode::*;
use Instruction::*;
#[allow(dead_code)]
pub const OPCODES: [Option<OpCode>; 0x100] = [
    None,                         // $00    BRK	         Implied
    None,                         // $01    ORA ($NN,X)  Indexed Indirect
    None,                         // $02
    None,                         // $03
    None,                         // $04
    None,                         // $05    ORA $NN      Zero Page
    None,                         // $06    ASL $NN      Zero Page
    None,                         // $07
    None,                         // $08    PHP          Implied
    None,                         // $09    ORA #$NN     Immediate
    None,                         // $0A    ASL A        Accumulator
    None,                         // $0B
    None,                         // $0C
    None,                         // $0D    ORA $NNNN    Absolute
    None,                         // $0E    ASL $NNNN    Absolute
    None,                         // $0F
    None,                         // $10    BPL $NN      Relative
    None,                         // $11    ORA ($NN),Y  Indirect Indexed
    None,                         // $12
    None,                         // $13
    None,                         // $14
    None,                         // $15    ORA $NN,X    Zero Page,X
    None,                         // $16    ASL $NN,X    Zero Page,X
    None,                         // $17
    None,                         // $18    CLC          Implied
    None,                         // $19    ORA $NNNN,Y  Absolute,Y
    None,                         // $1A
    None,                         // $1B
    None,                         // $1C
    None,                         // $1D    ORA $NNNN,X  Absolute,X
    None,                         // $1E    ASL $NNNN,X  Absolute,X
    None,                         // $1F
    None,                         // $20    JSR $NNNN    Absolute
    None,                         // $21    AND ($NN,X)  Indexed Indirect
    None,                         // $22
    None,                         // $23
    None,                         // $24    BIT $NN      Zero Page
    None,                         // $25    AND $NN      Zero Page
    None,                         // $26    ROL $NN      Zero Page
    None,                         // $27
    None,                         // $28    PLP          Implied
    None,                         // $29    AND #$NN     Immediate
    None,                         // $2A    ROL A        Accumulator
    None,                         // $2B
    None,                         // $2C    BIT $NNNN    Absolute
    None,                         // $2D    AND $NNNN    Absolute
    None,                         // $2E    ROL $NNNN    Absolute
    None,                         // $2F
    None,                         // $30    BMI $NN      Relative
    None,                         // $31    AND ($NN),Y  Indirect Indexed
    None,                         // $32
    None,                         // $33
    None,                         // $34
    None,                         // $35    AND $NN,X    Zero Page,X
    None,                         // $36    ROL $NN,X    Zero Page,X
    None,                         // $37
    None,                         // $38    SEC          Implied
    None,                         // $39    AND $NNNN,Y  Absolute,Y
    None,                         // $3A
    None,                         // $3B
    None,                         // $3C
    None,                         // $3D    AND $NNNN,X  Absolute,X
    None,                         // $3E    ROL $NNNN,X  Absolute,X
    None,                         // $3F
    None,                         // $40    RTI          Implied
    None,                         // $41    EOR ($NN,X)  Indexed Indirect
    None,                         // $42
    None,                         // $43
    None,                         // $44
    None,                         // $45    EOR $NN      Zero Page
    None,                         // $46    LSR $NN      Zero Page
    None,                         // $47
    None,                         // $48    PHA          Implied
    None,                         // $49    EOR #$NN     Immediate
    None,                         // $4A    LSR A        Accumulator
    None,                         // $4B
    None,                         // $4C    JMP $NNNN    Absolute
    None,                         // $4D    EOR $NNNN    Absolute
    None,                         // $4E    LSR $NNNN    Absolute
    None,                         // $4F
    None,                         // $50    BVC $NN      Relative
    None,                         // $51    EOR ($NN),Y  Indirect Indexed
    None,                         // $52
    None,                         // $53
    None,                         // $54
    None,                         // $55    EOR $NN,X    Zero Page,X
    None,                         // $56    LSR $NN,X    Zero Page,X
    None,                         // $57
    None,                         // $58    CLI          Implied
    None,                         // $59    EOR $NNNN,Y  Absolute,Y
    None,                         // $5A
    None,                         // $5B
    None,                         // $5C
    None,                         // $5D    EOR $NNNN,X  Absolute,X
    None,                         // $5E    LSR $NNNN,X  Absolute,X
    None,                         // $5F
    None,                         // $60    RTS          Implied
    None,                         // $61    ADC ($NN,X)  Indexed Indirect
    None,                         // $62
    None,                         // $63
    None,                         // $64
    None,                         // $65    ADC $NN      Zero Page
    None,                         // $66    ROR $NN      Zero Page
    None,                         // $67
    None,                         // $68    PLA          Implied
    None,                         // $69    ADC #$NN     Immediate
    None,                         // $6A    ROR A        Accumulator
    None,                         // $6B
    None,                         // $6C    JMP $NN      Indirect
    None,                         // $6D    ADC $NNNN    Absolute
    None,                         // $6E    ROR $NNNN,X  Absolute,X
    None,                         // $6F
    None,                         // $70    BVS $NN      Relative
    None,                         // $71    ADC ($NN),Y  Indirect Indexed
    None,                         // $72
    None,                         // $73
    None,                         // $74
    None,                         // $75    ADC $NN,X    Zero Page,X
    None,                         // $76    ROR $NN,X    Zero Page,X
    None,                         // $77
    None,                         // $78    SEI          Implied
    None,                         // $79    ADC $NNNN,Y  Absolute,Y
    None,                         // $7A
    None,                         // $7B
    None,                         // $7C
    None,                         // $7D    ADC $NNNN,X  Absolute,X
    None,                         // $7E    ROR $NNNN    Absolute
    None,                         // $7F
    None,                         // $80
    None,                         // $81    STA ($NN,X)  Indexed Indirect
    None,                         // $82
    None,                         // $83
    None,                         // $84    STY $NN      Zero Page
    Some(OpCode(STA, ZeroPage)),  // $85    STA $NN      Zero Page
    None,                         // $86    STX $NN      Zero Page
    None,                         // $87
    None,                         // $88    DEY          Implied
    None,                         // $89
    None,                         // $8A    TXA          Implied
    None,                         // $8B
    None,                         // $8C    STY $NNNN    Absolute
    Some(OpCode(STA, Absolute)),  // $8D    STA $NNNN    Absolute
    None,                         // $8E    STX $NNNN    Absolute
    None,                         // $8F
    None,                         // $90    BCC $NN      Relative
    None,                         // $91    STA ($NN),Y  Indirect Indexed
    None,                         // $92
    None,                         // $93
    None,                         // $94    STY $NN,X    Zero Page,X
    None,                         // $95    STA $NN,X    Zero Page,X
    None,                         // $96    STX $NN,Y    Zero Page,Y
    None,                         // $97
    None,                         // $98    TYA          Implied
    None,                         // $99    STA $NNNN,Y  Absolute,Y
    None,                         // $9A    TXS          Implied
    None,                         // $9B
    None,                         // $9C
    None,                         // $9D    STA $NNNN,X  Absolute,X
    None,                         // $9E
    None,                         // $9F
    Some(OpCode(LDY, Immediate)), // $A0    LDY #$NN     Immediate
    None,                         // $A1    LDA ($NN,X)  Indexed Indirect
    Some(OpCode(LDX, Immediate)), // $A2    LDX #$NN     Immediate
    None,                         // $A3
    Some(OpCode(LDY, ZeroPage)),  // $A4    LDY $NN      Zero Page
    Some(OpCode(LDA, ZeroPage)),  // $A5    LDA $NN      Zero Page
    Some(OpCode(LDX, ZeroPage)),  // $A6    LDX $NN      Zero Page
    None,                         // $A7
    None,                         // $A8    TAY          Implied
    Some(OpCode(LDA, Immediate)), // $A9    LDA #$NN     Immediate
    None,                         // $AA    TAX          Implied
    None,                         // $AB
    Some(OpCode(LDY, Absolute)),  // $AC    LDY $NNNN    Absolute
    Some(OpCode(LDA, Absolute)),  // $AD    LDA $NNNN    Absolute
    Some(OpCode(LDX, Absolute)),  // $AE    LDX $NNNN    Absolute
    None,                         // $AF
    None,                         // $B0    BCS $NN      Relative
    None,                         // $B1    LDA ($NN),Y  Indirect Indexed
    None,                         // $B2
    None,                         // $B3
    Some(OpCode(LDY, ZeroPageX)), // $B4    LDY $NN,X    Zero Page,X
    Some(OpCode(LDA, ZeroPageX)), // $B5    LDA $NN,X    Zero Page,X
    Some(OpCode(LDX, ZeroPageY)), // $B6    LDX $NN,Y    Zero Page,Y
    None,                         // $B7
    None,                         // $B8    CLV          Implied
    None,                         // $B9    LDA $NNNN,Y  Absolute,Y
    None,                         // $BA    TSX          Implied
    None,                         // $BB
    None,                         // $BC    LDY $NNNN,X  Absolute,X
    None,                         // $BD    LDA $NNNN,X  Absolute,X
    None,                         // $BE    LDX $NNNN,Y  Absolute,Y
    None,                         // $BF
    None,                         // $C0    CPY #$NN     Immediate
    None,                         // $C1    CMP ($NN,X)  Indexed Indirect
    None,                         // $C2
    None,                         // $C3
    None,                         // $C4    CPY $NN      Zero Page
    None,                         // $C5    CMP $NN      Zero Page
    None,                         // $C6    DEC $NN      Zero Page
    None,                         // $C7
    None,                         // $C8    INY          Implied
    None,                         // $C9    CMP #$NN     Immediate
    None,                         // $CA    DEX          Implied
    None,                         // $CB
    None,                         // $CC    CPY $NNNN    Absolute
    None,                         // $CD    CMP $NNNN    Absolute
    None,                         // $CE    DEC $NNNN    Absolute
    None,                         // $CF
    None,                         // $D0    BNE $NN      Relative
    None,                         // $D1    CMP ($NN),Y  Indirect Indexed
    None,                         // $D2
    None,                         // $D3
    None,                         // $D4
    None,                         // $D5    CMP $NN,X    Zero Page,X
    None,                         // $D6    DEC $NN,X    Zero Page,X
    None,                         // $D7
    None,                         // $D8    CLD          Implied
    None,                         // $D9    CMP $NNNN,Y  Absolute,Y
    None,                         // $DA
    None,                         // $DB
    None,                         // $DC
    None,                         // $DD    CMP $NNNN,X  Absolute,X
    None,                         // $DE    DEC $NNNN,X  Absolute,X
    None,                         // $DF
    None,                         // $E0    CPX #$NN     Immediate
    None,                         // $E1    SBC ($NN,X)  Indexed Indirect
    None,                         // $E2
    None,                         // $E3
    None,                         // $E4    CPX $NN      Zero Page
    None,                         // $E5    SBC $NN      Zero Page
    None,                         // $E6    INC $NN      Zero Page
    None,                         // $E7
    None,                         // $E8    INX          Implied
    None,                         // $E9    SBC #$NN     Immediate
    Some(OpCode(NOP, Implied)),   // $EA    NOP          Implied
    None,                         // $EB
    None,                         // $EC    CPX $NNNN    Absolute
    None,                         // $ED    SBC $NNNN    Absolute
    None,                         // $EE    INC $NNNN    Absolute
    None,                         // $EF
    None,                         // $F0    BEQ $NN      Relative
    None,                         // $F1    SBC ($NN),Y  Indirect Indexed
    None,                         // $F2
    None,                         // $F3
    None,                         // $F4
    None,                         // $F5    SBC $NN,X    Zero Page,X
    None,                         // $F6    INC $NN,X    Zero Page,X
    None,                         // $F7
    None,                         // $F8    SED          Implied
    None,                         // $F9    SBC $NNNN,Y  Absolute,Y
    None,                         // $FA
    None,                         // $FB
    None,                         // $FC
    None,                         // $FD    SBC $NNNN,X  Absolute,X
    None,                         // $FE    INC $NNNN,X  Absolute,X
    None,                         // $FF
];