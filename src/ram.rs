use std::ops::{Index, IndexMut};

use crate::reset::Reset;

pub trait MemIO {
    fn read_byte(&mut self, address: usize) -> u8;
    fn read_byte_without_effect(&mut self, address: usize) -> u8;
    fn write_byte(&mut self, address: usize, byte: u8);
}

const MAX_MEMORY: usize = 0x100 * 0x100;
#[derive(Debug)]
pub struct RAM {
    inner: Vec<u8>,
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
            inner: vec![0; MAX_MEMORY],
        }
    }
}

impl RAM {
    #[allow(dead_code)]
    pub fn new(buf: Vec<u8>) -> Self {
        Self { inner: buf }
    }

    #[allow(dead_code)]
    pub fn write_rom(&mut self, start_address: usize, data: &[u8]) {
        self.inner[start_address..(start_address + data.len())].clone_from_slice(data);
    }
}

impl MemIO for RAM {
    fn read_byte(&mut self, address: usize) -> u8 {
        self.inner[address]
    }

    fn read_byte_without_effect(&mut self, address: usize) -> u8 {
        self.inner[address]
    }

    fn write_byte(&mut self, address: usize, byte: u8) {
        self.inner[address] = byte;
    }
}

impl Reset for RAM {
    fn reset(&mut self) {}
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
}
