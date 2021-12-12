#![feature(const_char_convert)]
#![feature(const_for)]
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Debug, Display, Write};
use std::iter::FromIterator;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum Seg {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Seg {
    pub const fn values() -> &'static [Self; 7] {
        &[
            Self::A,
            Self::B,
            Self::C,
            Self::D,
            Self::E,
            Self::F,
            Self::G,
        ]
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0..=6 => Some(Self::values()[value as usize]),
            _ => None,
        }
    }

    pub const fn from_u8_panicking(value: u8) -> Self {
        Self::values()[value as usize]
    }

    pub const fn from_char(value: char) -> Option<Self> {
        Some(Self::from_u8_panicking(match value {
            'a'..='g' => value as u8 - b'a',
            'A'..='G' => value as u8 - b'A',
            _ => return None,
        }))
    }

    pub const fn into_char(&self) -> char {
        unsafe {
            // SAFETY: a+0 through a+6 are all valid char values
            char::from_u32_unchecked((b'a' + *self as u8) as u32)
        }
    }
}

impl TryFrom<u8> for Seg {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(())
    }
}

impl TryFrom<char> for Seg {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::from_char(value).ok_or(())
    }
}

impl FromStr for Seg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let [c] = s.as_bytes() {
            (*c as char).try_into()
        } else {
            Err(())
        }
    }
}

impl From<Seg> for char {
    fn from(value: Seg) -> Self {
        value.into_char()
    }
}

impl Display for Seg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.into_char())
    }
}

#[derive(Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct Segments(u8);

impl Segments {
    pub const fn none() -> Segments {
        Segments(0)
    }

    pub const fn all() -> Segments {
        Segments(0b1111111)
    }

    pub const fn from_bits_masked(bits: u8) -> Self {
        Segments(bits & ((1 << 7) - 1))
    }

    pub const fn len(&self) -> u8 {
        self.0.count_ones() as u8
    }

    pub const fn contains(&self, seg: Self) -> bool {
        (self.0 & seg.0) == seg.0
    }

    pub fn map(&self, map: impl Fn(Seg) -> Seg) -> Segments {
        self.into_iter().map(map).collect()
    }

    pub fn print(&self) {
        let get_fill = |seg: u8| {
            if self.0 & (1 << seg) != 0 {
                '#'
            } else {
                ' '
            }
        };
        let b = get_fill(1);
        let c = get_fill(2);
        let e = get_fill(4);
        let f = get_fill(5);
        println!(" {0}{0}{0} ", get_fill(0));
        println!("{}   {}", b, c);
        println!("{}   {}", b, c);
        println!(" {0}{0}{0} ", get_fill(3));
        println!("{}   {}", e, f);
        println!("{}   {}", e, f);
        println!(" {0}{0}{0} ", get_fill(6));
    }
}

impl Debug for Segments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.into_iter()).finish()
    }
}

impl BitOr for Segments {
    type Output = Segments;

    fn bitor(self, rhs: Self) -> Self::Output {
        Segments(self.0 | rhs.0)
    }
}

impl BitOrAssign for Segments {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAnd for Segments {
    type Output = Segments;

    fn bitand(self, rhs: Self) -> Self::Output {
        Segments(self.0 & rhs.0)
    }
}

impl BitAndAssign for Segments {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl From<Seg> for Segments {
    fn from(value: Seg) -> Self {
        Segments(1 << value as u8)
    }
}

impl FromStr for Segments {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = Segments::none();
        for c in s.chars() {
            let seg: Seg = c.try_into()?;
            res |= seg.into();
        }
        Ok(res)
    }
}

impl<S> FromIterator<S> for Segments
where
    S: Borrow<Seg>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut res = Self::default();
        for item in iter.into_iter() {
            res |= (*item.borrow()).into();
        }
        res
    }
}

pub struct SegmentsIter(u8);

impl IntoIterator for Segments {
    type Item = Seg;

    type IntoIter = SegmentsIter;

    fn into_iter(self) -> Self::IntoIter {
        SegmentsIter(self.0)
    }
}

impl Iterator for SegmentsIter {
    type Item = Seg;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.0.trailing_zeros() as u8;
        let result = Seg::from_u8(i as u8);
        if result.is_some() {
            self.0 &= !(1 << i);
        }
        result
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl std::iter::FusedIterator for SegmentsIter {}
impl std::iter::ExactSizeIterator for SegmentsIter {
    fn len(&self) -> usize {
        self.0.count_ones() as usize
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Digit {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
}

impl Digit {
    pub const fn segments(&self) -> Segments {
        Segments::from_bits_masked(match self {
            Self::D0 => 0b1110111,
            Self::D1 => 0b0100100,
            Self::D2 => 0b1011101,
            Self::D3 => 0b1101101,
            Self::D4 => 0b0101110,
            Self::D5 => 0b1101011,
            Self::D6 => 0b1111011,
            Self::D7 => 0b0100101,
            Self::D8 => 0b1111111,
            Self::D9 => 0b1101111,
        })
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0..=9 => Some(Self::values()[value as usize]),
            _ => return None,
        }
    }

    pub const fn from_segments(seg: Segments) -> Option<Self> {
        Some(match seg.0 {
            0b1110111 => Self::D0,
            0b0100100 => Self::D1,
            0b1011101 => Self::D2,
            0b1101101 => Self::D3,
            0b0101110 => Self::D4,
            0b1101011 => Self::D5,
            0b1111011 => Self::D6,
            0b0100101 => Self::D7,
            0b1111111 => Self::D8,
            0b1101111 => Self::D9,
            _ => return None,
        })
    }

    pub const fn values() -> &'static [Self; 10] {
        &[
            Self::D0,
            Self::D1,
            Self::D2,
            Self::D3,
            Self::D4,
            Self::D5,
            Self::D6,
            Self::D7,
            Self::D8,
            Self::D9,
        ]
    }

    pub const fn values_of_len(len: u8) -> &'static [Self] {
        match len {
            2 => &[Self::D1],
            3 => &[Self::D7],
            4 => &[Self::D4],
            5 => &[Self::D2, Self::D3, Self::D5],
            6 => &[Self::D0, Self::D6, Self::D9],
            7 => &[Self::D8],
            _ => &[],
        }
    }

    pub fn parse(digits: impl IntoIterator<Item = impl Borrow<Digit>>) -> u32 {
        let mut value = 0;
        for d in digits.into_iter() {
            value *= 10;
            value += *d.borrow() as u32;
        }
        value
    }
}

impl TryFrom<u8> for Digit {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(())
    }
}

pub trait AsSegments {
    fn as_segments(&self) -> Segments;
}

impl AsSegments for Segments {
    fn as_segments(&self) -> Segments {
        *self
    }
}

impl AsSegments for Seg {
    fn as_segments(&self) -> Segments {
        (*self).into()
    }
}

impl AsSegments for Digit {
    fn as_segments(&self) -> Segments {
        self.segments()
    }
}

impl<T: AsSegments> AsSegments for &T {
    fn as_segments(&self) -> Segments {
        (*self).as_segments()
    }
}

type SegId = [u8; 6];

pub fn calc_segment_ids(all_states: impl IntoIterator<Item = impl AsSegments>) -> [SegId; 7] {
    let mut ids = [[0u8; 6]; 7];
    for state in all_states {
        let segments = state.as_segments();
        for s in segments {
            ids[s as usize][segments.len() as usize - 2] += 1;
        }
    }
    ids
}

pub fn main() {
    let input: Vec<_> = include_str!("input.txt")
        .lines()
        .map(|s| {
            let (all_states, output) = s.split_once("|").unwrap();
            let all_states: Vec<Segments> = all_states
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let output: Vec<Segments> = output
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            (all_states, output)
        })
        .collect();

    let unique_len_digits = [Digit::D1, Digit::D4, Digit::D7, Digit::D8];
    let unique_digit_lens = unique_len_digits.map(|d| d.segments().len());

    let mut unique_lens_in_output = 0;
    for (_, output) in &input {
        for seg in output {
            if unique_digit_lens.contains(&seg.len()) {
                unique_lens_in_output += 1;
            }
        }
    }

    println!("P1: {} unique lengths in output", unique_lens_in_output);

    let normal_seg_ids: HashMap<SegId, Seg> = calc_segment_ids(Digit::values())
        .into_iter()
        .enumerate()
        .map(|(i, id)| (id, Seg::from_u8(i as u8).unwrap()))
        .collect();

    let mut output_sum = 0;
    for (all_states, output) in &input {
        let shifted_segs = calc_segment_ids(all_states)
            .map(|id| *normal_seg_ids.get(&id).expect("impossible layout"));
        let output = Digit::parse(
            output
                .into_iter()
                .map(|s| Digit::from_segments(s.map(|s| shifted_segs[s as usize])).unwrap()),
        );
        output_sum += output;
    }
    print!("P2: sum of outputs is {}", output_sum);
}
