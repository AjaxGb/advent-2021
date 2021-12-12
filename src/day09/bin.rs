#![feature(nonzero_ops)]
use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Display};
use std::iter::FusedIterator;
use std::num::NonZeroUsize;
use std::str::FromStr;

use arrayvec::ArrayVec;

#[derive(Clone, PartialEq, Eq)]
pub struct Heightmap {
    heights: Vec<u8>,
    width: NonZeroUsize,
}

impl Heightmap {
    pub fn size(&self) -> (usize, usize) {
        (self.width.get(), self.height())
    }

    pub fn try_get(&self, x: usize, y: usize) -> Option<u8> {
        if x >= self.width.get() {
            None
        } else {
            self.heights.get(self.get_pos(x, y)).copied()
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.try_get(x, y).unwrap()
    }

    pub fn adjacent(&self, x: usize, y: usize) -> ArrayVec<((usize, usize), u8), 4> {
        let mut adj = ArrayVec::new();
        let mut add_if_valid = |x, y| {
            if let (Some(x), Some(y)) = (x, y) {
                if let Some(h) = self.try_get(x, y) {
                    adj.push(((x, y), h))
                }
            }
        };
        add_if_valid(Some(x), y.checked_add(1));
        add_if_valid(x.checked_sub(1), Some(y));
        add_if_valid(Some(x), y.checked_sub(1));
        add_if_valid(x.checked_add(1), Some(y));
        adj
    }

    fn height(&self) -> usize {
        self.heights.len() / self.width
    }

    const fn get_pos(&self, x: usize, y: usize) -> usize {
        y * self.width.get() + x
    }
}

impl Debug for Heightmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Heightmap [")?;
        for chunk in self.heights.chunks_exact(self.width.get()) {
            writeln!(f, "    {:?},", chunk)?
        }
        write!(f, "]")
    }
}

impl<'a> IntoIterator for &'a Heightmap {
    type Item = ((usize, usize), u8);

    type IntoIter = HeightmapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HeightmapIter { map: self, pos: 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightmapIter<'a> {
    map: &'a Heightmap,
    pos: usize,
}

impl<'a> Iterator for HeightmapIter<'a> {
    type Item = ((usize, usize), u8);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(height) = self.map.heights.get(self.pos) {
            let x = self.pos % self.map.width;
            let y = self.pos / self.map.width;
            self.pos += 1;
            Some(((x, y), *height))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for HeightmapIter<'_> {
    fn len(&self) -> usize {
        self.map.heights.len() - self.pos
    }
}

impl FusedIterator for HeightmapIter<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeightmapParseError {
    InvalidDigit {
        byte: u8,
    },
    EmptyFirstLine,
    InconsistentWidths {
        line: NonZeroUsize,
        expected: NonZeroUsize,
        actual: usize,
    },
}

impl Display for HeightmapParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDigit { byte } => {
                if let Ok(text) = std::str::from_utf8(std::slice::from_ref(byte)) {
                    write!(f, "invalid digit {:?}", text)
                } else {
                    write!(f, "invalid digit 0x{:x}", byte)
                }
            }
            Self::EmptyFirstLine => write!(f, "first line was empty"),
            Self::InconsistentWidths {
                line,
                expected,
                actual,
            } => write!(
                f,
                "expected all rows to be {} wide, but line {} was {} wide",
                expected, line, actual
            ),
        }
    }
}

impl std::error::Error for HeightmapParseError {}

impl FromStr for Heightmap {
    type Err = HeightmapParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // SAFETY: quick maffs
        let one = unsafe { NonZeroUsize::new_unchecked(1) };

        let mut lines = s.lines();
        let first_line = lines.next().unwrap_or("");
        let width: NonZeroUsize = first_line
            .len()
            .try_into()
            .or(Err(Self::Err::EmptyFirstLine))?;
        let mut heights = vec![];

        let mut parse_line = |i, line: &str| {
            if line.len() != width.get() {
                Err(Self::Err::InconsistentWidths {
                    line: one.checked_add(i).unwrap(),
                    expected: width,
                    actual: line.len(),
                })
            } else {
                heights.reserve(line.len());
                for byte in line.bytes() {
                    if let b'0'..=b'9' = byte {
                        heights.push(byte - b'0');
                    } else {
                        return Err(Self::Err::InvalidDigit { byte });
                    }
                }
                Ok(())
            }
        };

        parse_line(0, first_line)?;
        for (i, line) in lines.enumerate() {
            parse_line(i + 1, line)?;
        }

        Ok(Heightmap { heights, width })
    }
}

pub fn main() {
    let heights: Heightmap = include_str!("input.txt").parse().unwrap();

    let mut risk_sum = 0;

    let mut max_basins = [0usize; 3];
    let mut basin = HashSet::new();
    let mut to_visit = VecDeque::new();

    for ((x, y), height) in &heights {
        let min_adjacent = heights
            .adjacent(x, y)
            .into_iter()
            .map(|(_, h)| h)
            .min()
            .unwrap();
        if height < min_adjacent {
            risk_sum += height as u32 + 1;

            // Flood fill
            basin.clear();
            basin.insert((x, y));
            to_visit.clear();
            to_visit.push_back((x, y));

            while let Some((x, y)) = to_visit.pop_front() {
                for ((x, y), h) in heights.adjacent(x, y) {
                    if h < 9 && basin.insert((x, y)) {
                        to_visit.push_back((x, y));
                    }
                }
            }

            println!("({}, {}) -> {} {:?}", x, y, basin.len(), basin);

            let (min_i, min_len) = max_basins
                .iter()
                .copied()
                .enumerate()
                .min_by_key(|(_, l)| *l)
                .unwrap();
            if min_len < basin.len() {
                max_basins[min_i] = basin.len();
            }
        }
    }

    let max_basins_product: usize = max_basins.into_iter().product();

    println!("P1: risk sum is {}", risk_sum);
    println!(
        "P2: largest basins are {:?} = {}",
        max_basins, max_basins_product
    );
}
