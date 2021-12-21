#![feature(array_chunks)]

use std::{error::Error, fmt::Display};

use bitvec::prelude::*;

pub struct Algo(BitArray<Lsb0, [u64; 8]>);

impl Algo {
    pub const LEN: u32 = u64::BITS * 8;

    #[must_use]
    pub fn parse(text: &str) -> Self {
        assert_eq!(text.len(), 512);
        let mut r = BitArray::zeroed();
        for (i, c) in text.as_bytes().into_iter().enumerate() {
            if *c == b'#' {
                r.set(i, true);
            }
        }
        Self(r)
    }

    #[must_use]
    pub fn get(&self, index: u32) -> bool {
        self.0[index as usize]
    }
}

impl Display for Algo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..Self::LEN {
            if self.get(i) {
                write!(f, "#")?;
            } else {
                write!(f, ".")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Image {
    bits: BitVec,
    width: usize,
    background: bool,
}

impl Image {
    #[must_use]
    pub fn parse(mut lines: std::str::Lines) -> Self {
        let first = lines.next().unwrap();
        let width = first.len();
        let mut bits = bitvec![];
        let mut append_line = |line: &str| {
            for c in line.as_bytes().into_iter() {
                bits.push(*c == b'#');
            }
        };
        append_line(first);
        for line in lines {
            assert_eq!(line.len(), width);
            append_line(line);
        }
        Self {
            bits,
            width,
            background: false,
        }
    }

    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.bits.len() / self.width
    }

    #[must_use]
    pub fn get_pixel(&self, x: isize, y: isize) -> bool {
        if x < 0 || x as usize >= self.width || y < 0 || y as usize >= self.height() {
            self.background
        } else {
            self.bits[y as usize * self.width + x as usize]
        }
    }

    #[must_use]
    pub fn get_3x3(&self, x: isize, y: isize) -> u32 {
        let mut r = 0;
        for y in (y - 1)..=(y + 1) {
            for x in (x - 1)..=(x + 1) {
                r <<= 1;
                r |= self.get_pixel(x, y) as u32;
            }
        }
        r
    }

    #[must_use]
    pub fn enhance(&self, algo: &Algo) -> Self {
        let new_width = self.width + 2;
        let new_height = self.height() + 2;
        let mut new_bits = BitVec::with_capacity(new_width * new_height);

        for y in 0..new_width {
            for x in 0..new_height {
                let algo_index = self.get_3x3(x as isize - 1, y as isize - 1);
                new_bits.push(algo.get(algo_index));
            }
        }

        let new_bg_index = if self.background { 0b111_111_111 } else { 0 };

        Self {
            bits: new_bits,
            width: new_width,
            background: algo.get(new_bg_index),
        }
    }

    pub fn num_lit(&self) -> Result<usize, InfiniteBitsError> {
        let lit_in_bounds = self.bits.count_ones();
        if self.background {
            Err(InfiniteBitsError::new(lit_in_bounds))
        } else {
            Ok(lit_in_bounds)
        }
    }

    pub fn print(&self) {
        for row in self.bits.chunks_exact(self.width) {
            for bit in row {
                if *bit {
                    print!("#");
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InfiniteBitsError(usize);

impl InfiniteBitsError {
    pub const fn new(bits_in_bound: usize) -> Self {
        Self(bits_in_bound)
    }

    pub const fn bits_in_bounds(&self) -> usize {
        self.0
    }
}

impl Error for InfiniteBitsError {}
impl Display for InfiniteBitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "image has infinite matching bits ({} in bounds)", self.0)
    }
}

fn main() {
    let mut lines = include_str!("input.txt").lines();
    let algo = Algo::parse(lines.next().unwrap());
    assert_eq!(lines.next(), Some(""));
    let mut image = Image::parse(lines);

    image.print();
    println!();
    image = image.enhance(&algo);
    image.print();
    println!();
    image = image.enhance(&algo);
    image.print();
    println!();

    println!("P1: {} lit pixels", image.num_lit().unwrap());

    for _ in 2..50 {
        image = image.enhance(&algo);
    }

    println!("P2: {} lit pixels", image.num_lit().unwrap());
}
