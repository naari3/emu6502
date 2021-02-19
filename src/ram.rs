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
}
