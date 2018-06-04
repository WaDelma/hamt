#[derive(Clone, Copy, Debug)]
pub struct BitMap(usize);

impl BitMap {
    pub fn empty() -> Self {
        BitMap(0)
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_set(&self, i: usize) -> bool {
        (self.0 >> i) & 1 == 1
    }

    pub fn set_bits(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn set_bits_under(&self, i: usize) -> usize {
        let mask = (1 << i) - 1;
        (self.0 & mask).count_ones() as usize
    }

    pub fn set(&mut self, i: usize) {
        self.0 |= 1 << i;
    }

    pub fn get(&self) -> usize {
        self.0
    }
}