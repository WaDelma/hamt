use std::mem::size_of;
use std::hash::Hasher;

pub const BITS_IN_BYTE: usize = 8;
pub const BITS_IN_POINTER: usize = size_of::<usize>() * BITS_IN_BYTE;

fn bits_to_represent_pointer() -> usize {
    BITS_IN_POINTER.trailing_zeros() as usize
}

pub struct HashStream<H: Hasher> {
    hasher: H,
    reservoy: u64,
    eaten: u8,
}

impl<H: Hasher> HashStream<H> {
    pub fn new(hasher: H) -> Self {
        HashStream {
            reservoy: hasher.finish(),
            hasher,
            eaten: 0,
        }
    }
}

impl<H: Hasher> Iterator for HashStream<H> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let bits = bits_to_represent_pointer() as u8;
        let limit = 64 - bits;
        Some(if self.eaten > limit {
            let need = self.eaten - limit;
            let left = bits - need;
            let leftovers = self.reservoy;

            // TODO: Is this good idea?
            self.hasher.write_u8(1);
            self.reservoy = self.hasher.finish();

            let mask = (1 << need) - 1;
            let needed = self.reservoy & mask;
            self.reservoy >>= need;
            self.eaten = need;
            (needed << left | leftovers) as u8
        } else {
            let mask = (1 << bits) - 1;
            let result = self.reservoy & mask;
            self.reservoy >>= bits;
            self.eaten += bits;
            result as u8
        })
    }
}

#[test]
fn hash_stream() {
    use std::cell::Cell;
    struct MockHasher(Cell<usize>, &'static [u64]);
    impl Hasher for MockHasher {
        fn finish(&self) -> u64 {
            let n = self.0.get();
            let result = self.1[n];
            self.0.set(n + 1);
            result
        }
        fn write(&mut self, _: &[u8]) {}
    }

    let hasher = MockHasher(Cell::new(0), &[
        0b1011_001010_001001_001000_000111_000110_000101_000100_000011_000010_000001,
        0b10_010101_010100_010011_010010_010001_010000_001111_001110_001101_001100_00,
        0b100000_011111_011110_011101_011100_011011_011010_011001_011000_010111_0101,
        0b0000_101010_101001_101000_100111_100110_100101_100100_100011_100010_100001,
    ]);

    assert_eq!(
        (1..=42).collect::<Vec<_>>(),
        HashStream::new(hasher).take(42).collect::<Vec<_>>()
    );
}