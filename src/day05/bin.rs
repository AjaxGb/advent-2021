#![feature(exclusive_range_pattern)]
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::iter::{FromIterator, FusedIterator};
use std::num::ParseIntError;
use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const MIN: Self = Self::uniform(i32::MIN);
    pub const MAX: Self = Self::uniform(i32::MAX);

    pub const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    pub const fn uniform(p: i32) -> Self {
        Point { x: p, y: p }
    }
}

impl FromStr for Point {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((x, y)) = s.split(',').collect_tuple() {
            Ok(Self {
                x: x.parse()?,
                y: y.parse()?,
            })
        } else {
            Err(Self::Err::BadFormat)
        }
    }
}

#[derive(Debug, Clone)]
struct LineDef(pub Point, pub Point);

impl LineDef {
    pub const fn is_horiz(&self) -> bool {
        self.0.y == self.1.y
    }

    pub const fn is_vert(&self) -> bool {
        self.0.x == self.1.x
    }

    pub const fn is_cardinal(&self) -> bool {
        self.is_horiz() || self.is_vert()
    }

    pub fn points(&self) -> LineDefIter {
        LineDefIter::Moving {
            curr: self.0,
            end: self.1,
        }
    }
}

impl FromStr for LineDef {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((a, b)) = s.split(" -> ").collect_tuple() {
            Ok(LineDef(a.parse()?, b.parse()?))
        } else {
            Err(Self::Err::BadFormat)
        }
    }
}

enum LineDefIter {
    Moving { curr: Point, end: Point },
    Done,
}

impl Iterator for LineDefIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if let Self::Moving { curr, end } = self {
            let result = *curr;
            let x_diff = end.x.cmp(&curr.x);
            let y_diff = end.y.cmp(&curr.y);
            if x_diff.is_eq() && y_diff.is_eq() {
                *self = Self::Done;
            } else {
                curr.x += x_diff as i32;
                curr.y += y_diff as i32;
            }
            Some(result)
        } else {
            None
        }
    }
}

impl FusedIterator for LineDefIter {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseError {
    BadFormat,
    NumberError(ParseIntError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadFormat => write!(f, "incorrect formatting"),
            Self::NumberError(err) => write!(f, "invalid number: {}", err),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NumberError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        Self::NumberError(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LineMap {
    min: Point,
    max: Point,
    filled: HashMap<Point, u32>,
}

impl LineMap {
    pub fn new() -> Self {
        LineMap {
            min: Point::MAX,
            max: Point::MIN,
            filled: HashMap::new(),
        }
    }

    pub fn add_line(&mut self, line: &LineDef) {
        self.min.x = self.min.x.min(line.0.x).min(line.1.x);
        self.max.x = self.max.x.max(line.0.x).max(line.1.x);
        self.min.y = self.min.y.min(line.0.y).min(line.1.y);
        self.max.y = self.max.y.max(line.0.y).max(line.1.y);
        for point in line.points() {
            *self.filled.entry(point).or_default() += 1;
        }
    }

    pub fn count_overlaps(&self) -> usize {
        self.filled.iter().filter(|(_, c)| **c > 1).count()
    }

    pub fn print(&self) {
        for y in self.min.y..=self.max.y {
            for x in self.min.x..=self.max.x {
                print!(
                    "{}",
                    match self.filled.get(&Point::new(x, y)) {
                        Some(num @ 0..=9) => std::char::from_digit(*num, 10).unwrap(),
                        Some(_) => '+',
                        None => '.',
                    }
                );
            }
            println!();
        }
    }
}

impl Default for LineMap {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Borrow<LineDef>> FromIterator<T> for LineMap {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut map = LineMap::new();
        for line in iter {
            map.add_line(line.borrow());
        }
        map
    }
}

pub fn main() {
    let input: Vec<LineDef> = include_str!("input.txt")
        .lines()
        .map(|s| s.parse().unwrap())
        .collect();

    // P1
    let map: LineMap = input.iter().filter(|d| d.is_cardinal()).collect();
    map.print();
    println!("P1: {} overlaps", map.count_overlaps());

    // P2
    let map: LineMap = input.iter().collect();
    map.print();
    println!("P2: {} overlaps", map.count_overlaps());
}
