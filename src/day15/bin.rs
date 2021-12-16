#![feature(int_abs_diff)]
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::fmt::{Debug, Display};
use std::num::NonZeroU32;
use std::str::FromStr;

use arrayvec::ArrayVec;

pub fn taxicab_distance(from: (usize, usize), to: (usize, usize)) -> u32 {
    (from.0.abs_diff(to.0) + from.1.abs_diff(to.1))
        .try_into()
        .unwrap()
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Grid {
    risks: Vec<NonZeroU32>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn neighbors(&self, pos: (usize, usize), wraps: usize) -> ArrayVec<(usize, usize), 4> {
        let (x, y) = pos;
        let width = self.width * wraps;
        let height = self.height * wraps;
        let mut neighbors = ArrayVec::new();
        if y < height {
            if x + 1 < width {
                neighbors.push((x + 1, y));
            }
            if x > 0 && x <= width {
                neighbors.push((x - 1, y));
            }
        }
        if x < width {
            if y + 1 < height {
                neighbors.push((x, y + 1));
            }
            if y > 0 && y <= height {
                neighbors.push((x, y - 1));
            }
        }
        neighbors
    }

    pub fn risk_at(&self, pos: (usize, usize)) -> NonZeroU32 {
        let (x, y) = pos;
        assert!(
            x < self.width && y < self.height,
            "{:?} is out of range",
            pos
        );
        *unsafe { self.risks.get_unchecked(x + y * self.width) }
    }

    pub fn risk_at_wrapping(&self, pos: (usize, usize)) -> NonZeroU32 {
        let (x, y) = pos;
        let x_wraps = (x / self.width) as u32;
        let x = x % self.width;
        let y_wraps = (y / self.height) as u32;
        let y = y % self.height;
        let risk =
            x_wraps + y_wraps + unsafe { self.risks.get_unchecked(x + y * self.width) }.get();
        unsafe { NonZeroU32::new_unchecked((risk - 1) % 9 + 1) }
    }

    pub fn a_star(&self, start: (usize, usize), goal: (usize, usize), wraps: usize) -> Option<u32> {
        let make_visitable = |pos, curr_risk| {
            let h = curr_risk + taxicab_distance(pos, goal) / 2;
            Reverse((h, curr_risk, pos))
        };

        let mut to_visit = BinaryHeap::new();
        to_visit.push(make_visitable(start, 0));

        let mut visited = HashSet::new();
        visited.insert(start);

        while let Some(Reverse((_, risk, pos))) = to_visit.pop() {
            if pos == goal {
                return Some(risk);
            }

            for neighbor in self.neighbors(pos, wraps) {
                if visited.insert(neighbor) {
                    let curr_risk = risk + self.risk_at_wrapping(neighbor).get();
                    to_visit.push(make_visitable(neighbor, curr_risk));
                }
            }
        }
        None
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grid {{ width: {}, height: {},", self.width, self.height)?;
        if self.width == 0 {
            for _ in 0..self.height {
                writeln!(f, "[]")?;
            }
        } else {
            for row in self.risks.chunks_exact(self.width) {
                writeln!(f, "    {:?}", row)?;
            }
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GridParseErr {
    InconsistentWidth { expect: usize, actual: usize },
    InvalidDigit(char),
    ZeroWeight,
}

impl Display for GridParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InconsistentWidth { expect, actual } => {
                write!(
                    f,
                    "expected all lines to be {} bytes, but one was {}",
                    expect, actual
                )
            }
            Self::InvalidDigit(c) => write!(f, "invalid digit {}", c),
            Self::ZeroWeight => write!(f, "one of the weights was zero"),
        }
    }
}

impl std::error::Error for GridParseErr {}

impl FromStr for Grid {
    type Err = GridParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        if let Some(first) = lines.next() {
            let width = first.len();
            let expected_height = (s.len() + width) / (width + 1);
            let mut risks = Vec::with_capacity(width * expected_height);
            let mut height = 1;

            let mut parse_line = |line: &str| -> Result<(), Self::Err> {
                for c in line.chars() {
                    if let Some(risk) = c.to_digit(10) {
                        if let Some(risk) = NonZeroU32::new(risk) {
                            risks.push(risk);
                        } else {
                            return Err(Self::Err::ZeroWeight);
                        }
                    } else {
                        return Err(Self::Err::InvalidDigit(c));
                    }
                }
                Ok(())
            };

            parse_line(first)?;
            for line in lines {
                height += 1;
                if line.len() == width {
                    parse_line(line)?;
                } else {
                    return Err(Self::Err::InconsistentWidth {
                        expect: width,
                        actual: line.len(),
                    });
                }
            }

            Ok(Self {
                risks,
                width,
                height,
            })
        } else {
            Ok(Default::default())
        }
    }
}

pub fn main() {
    let grid: Grid = include_str!("input.txt").parse().unwrap();
    let start = (0, 0);
    let goal = (grid.width - 1, grid.height - 1);
    println!(
        "P1: min risk is {}",
        grid.a_star(start, goal, 1).expect("no solution to P1")
    );
    let goal = (grid.width * 5 - 1, grid.height * 5 - 1);
    println!(
        "P2: min risk is {}",
        grid.a_star(start, goal, 5).expect("no solution to P2")
    );
}
