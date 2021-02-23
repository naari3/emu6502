use std::ops::{Index, IndexMut};

const MAX_MEMORY: usize = 0x100 * 0x100;
pub struct RAM {
    inner: [u8; MAX_MEMORY],
}

impl Index<usize> for RAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl Default for RAM {
    fn default() -> Self {
        RAM {
            inner: [0; MAX_MEMORY],
        }
    }
}

impl RAM {
    pub fn initialize(&mut self) {
        self.inner = [0; MAX_MEMORY];
    }

    pub fn read_byte(&mut self, address: usize) -> u8 {
        self.inner[address]
    }

    pub fn write_byte(&mut self, address: usize, byte: u8) {
        self.inner[address] = byte;
    }

    #[allow(dead_code)]
    pub fn write_rom(&mut self, start_address: usize, data: &[u8]) {
        self.inner[start_address..(start_address + data.len())].clone_from_slice(data);
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
