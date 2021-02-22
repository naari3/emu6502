mod cpu;
mod instruction;
mod ram;

use std::ops::{Index, IndexMut};

use cpu::CPU;
use ram::RAMBus;

struct TestRAM {
    inner: [u8; 0x0800],
    rom: [u8; 0x8000],
}

impl Default for TestRAM {
    fn default() -> Self {
        TestRAM {
            inner: [0; 0x0800],
            rom: [0; 0x8000],
        }
    }
}

impl RAMBus for TestRAM {
    fn read_byte_ref(&self, address: usize) -> &u8 {
        match address {
            0x0000..=0x07FF => &self.inner[address],
            0x0800..=0x1FFF => &self.inner[address - 0x0800],
            0x8000..=0xFFFF => &self.rom[address - 0x8000],
            _ => &0,
        }
    }

    fn write_byte_ref(&mut self, address: usize) -> &mut u8 {
        match address {
            0x0000..=0x07FF => &mut self.inner[address],
            0x0800..=0x1FFF => &mut self.inner[address - 0x0800],
            0x8000..=0xFFFF => &mut self.rom[address - 0x8000],
            _ => &mut self.inner[address],
        }
    }

    fn initialize(&mut self) {
        self.inner = [0; 0x0800];
    }
}

impl Index<usize> for TestRAM
where
    TestRAM: RAMBus,
{
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        self.read_byte_ref(index)
    }
}

impl IndexMut<usize> for TestRAM
where
    TestRAM: RAMBus,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.write_byte_ref(index)
    }
}
fn main() {
    let mut cpu = CPU::default();
    let mut ram = TestRAM::default();
    cpu.reset(&mut ram);
    ram[0x8000] = 0xA9; // LDA #$02
    ram[0x8001] = 0x42; // LDA #$02

    ram[0xFFFC] = 0x00;
    ram[0xFFFD] = 0x80;

    ram[0x42] = 0x84;
    cpu.execute(4, &mut ram);
    println!("CPU: {:?}", cpu);
}

#[cfg(test)]
mod tests {
    use super::*;

    use cpu::CPU;

    impl TestRAM {
        #[allow(dead_code)]
        pub fn write_rom(&mut self, data: &[u8]) {
            self.rom[0..data.len()].clone_from_slice(data);
        }
    }

    #[test]
    fn test_case1() {
        let mut cpu = CPU::default();
        let mut ram = TestRAM::default();
        ram.write_rom(&[
            0xA2, 0x02, // LDX #$02
            0xB5, 0x40, // LDA $40,x
            0x85, 0x43, // STA $43
            0xAC, 0xFD, 0xFF, // LDY $FFFD
        ]);

        ram[0xFFFC] = 0x00;
        ram[0xFFFD] = 0x80;

        cpu.reset(&mut ram);
        ram[0x42] = 0x84;

        cpu.execute(15, &mut ram);
        assert_eq!(cpu.a, 0x84);
        assert_eq!(cpu.x, 0x02);
        assert_eq!(cpu.y, 0x80);
        assert_eq!(ram[0x43], 0x84);
    }

    #[test]
    fn test_case2() {
        // https://gist.github.com/pedrofranceschi/1285964
        let mut cpu = CPU::default();
        let mut ram = TestRAM::default();
        let to_loop = -11_i8 as u8;
        ram.write_rom(&[
            // https://gist.github.com/pedrofranceschi/1285964
            0xA2, 0x01, //     LDX #$01; x = 1
            0x86, 0x00, //     STX $00; stores x
            //
            0x38, //           SEC; clean carry;
            0xA0, 0x07, //     LDY #$07; calculates 7th fibonacci number (13 = D in hex)
            0x98, //           TYA; transfer y register to accumulator
            0xE9, 0x03, //     SBC #$03; handles the algorithm iteration counting
            0xA8, //           TAY; transfer the accumulator to the y register
            //
            0x18, //           CLC; clean carry
            0xA9, 0x02, //     LDA #$02; a = 2
            0x85, 0x01, //     STA $01; stores a
            //
            //             loop:
            0xA6, 0x01, //     LDX $01; x = a
            0x65, 0x00, //     ADC $00; a += x
            0x85, 0x01, //     STA $01; stores a
            0x86, 0x00, //     STX $00; stores x
            0x88, //           DEY; y -= 1
            0xD0, to_loop, //  BNE loop; jumps back to loop if Z bit != 0
        ]);

        ram[0xFFFC] = 0x00;
        ram[0xFFFD] = 0x80;
        cpu.reset(&mut ram);

        let cycles = 93;
        cpu.execute(cycles, &mut ram);
        assert_eq!(cpu.a, 0x0D);
    }

    #[test]
    fn test_case3() {
        // https://gist.github.com/pedrofranceschi/1285964
        let mut cpu = CPU::default();
        let mut ram = TestRAM::default();
        /*
        ROUTINE:
            LDA #$42
            RTS
        MAIN:
            JSR ROUTINE
         */
        ram.write_rom(&[
            0xA9, 0x42, //
            0x60, //
            // start here
            0x20, 0x00, 0x80, //
            0xEA,
        ]);

        ram[0xFFFC] = 0x00;
        ram[0xFFFD] = 0x80;
        cpu.reset(&mut ram);

        let cycles = 18;
        cpu.execute(cycles, &mut ram);
        assert_eq!(cpu.a, 0x42);
    }
}
