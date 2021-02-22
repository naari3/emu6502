use std::ops::{Index, IndexMut};

const MAX_MEMORY: usize = 0x100 * 0x100;
pub struct RAM {
    inner: [u8; MAX_MEMORY],
}

pub trait RAMBus {
    fn read_byte(&mut self, address: usize) -> u8 {
        *self.read_byte_ref(address)
    }
    fn read_byte_ref(&self, address: usize) -> &u8;
    fn write_byte(&mut self, address: usize, byte: u8) {
        *self.write_byte_ref(address) = byte;
    }
    fn write_byte_ref(&mut self, address: usize) -> &mut u8;
    fn initialize(&mut self);
}

impl RAMBus for RAM {
    fn read_byte(&mut self, address: usize) -> u8 {
        *self.read_byte_ref(address)
    }

    fn read_byte_ref(&self, address: usize) -> &u8 {
        &self.inner[address]
    }

    fn write_byte(&mut self, address: usize, byte: u8) {
        *self.write_byte_ref(address) = byte;
    }

    fn write_byte_ref(&mut self, address: usize) -> &mut u8 {
        &mut self.inner[address]
    }

    fn initialize(&mut self) {
        self.inner = [0; MAX_MEMORY];
    }
}

impl Index<usize> for RAM
where
    RAM: RAMBus,
{
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        self.read_byte_ref(index)
    }
}

impl IndexMut<usize> for RAM
where
    RAM: RAMBus,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.write_byte_ref(index)
    }
}

impl Default for RAM {
    fn default() -> Self {
        RAM {
            inner: [0; MAX_MEMORY],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index() {
        let mut ram = RAM::default();
        ram[0] = 0;
        ram[1] = 1;
        ram[2] = 2;
        assert_eq!(ram.read_byte(0), 0);
        assert_eq!(ram.read_byte(1), 1);
        assert_eq!(ram.read_byte(2), 2);
        assert_eq!(ram[0], 0);
        assert_eq!(ram[1], 1);
        assert_eq!(ram[2], 2);
    }

    #[test]
    fn test_initialize() {
        let mut ram = RAM::default();
        ram[0] = 0xFF;
        ram[MAX_MEMORY - 1] = 0xFF;
        ram.initialize();
        assert_eq!(ram.read_byte(0), 0);
        assert_eq!(ram.read_byte(MAX_MEMORY - 1), 0);
    }
}
