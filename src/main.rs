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

impl CPU {
    pub fn reset(&mut self) {
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
    }
}

fn main() {
    let mut cpu = CPU::default();
    cpu.reset();
    println!("Hello, world!");
}
