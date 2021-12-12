#![feature(hash_drain_filter)]
use collect_array::CollectArrayResult;
use itertools::Itertools;
use std::collections::{hash_map::Entry, HashMap};

pub struct BoardLayout {
    pos_to_num: [[u32; Self::USIZE]; Self::USIZE],
    num_to_pos: HashMap<u32, (u8, u8)>,
}

impl BoardLayout {
    pub const SIZE: u8 = 5;
    pub const USIZE: usize = Self::SIZE as usize;

    pub fn new(nums: [[u32; Self::USIZE]; Self::USIZE]) -> BoardLayout {
        let mut num_to_pos = HashMap::with_capacity(Self::USIZE * Self::USIZE);
        for (y, row) in nums.iter().enumerate() {
            for (x, n) in row.iter().enumerate() {
                match num_to_pos.entry(*n) {
                    Entry::Occupied(_) => panic!("duplicate number: {}", n),
                    Entry::Vacant(e) => e.insert((x as u8, y as u8)),
                };
            }
        }
        BoardLayout {
            pos_to_num: nums,
            num_to_pos,
        }
    }

    pub fn get_pos_of(&self, num: u32) -> Option<(u8, u8)> {
        self.num_to_pos.get(&num).copied()
    }

    pub fn get_num_at(&self, x: u8, y: u8) -> u32 {
        self.pos_to_num[y as usize][x as usize]
    }
}

impl From<[[u32; Self::USIZE]; Self::USIZE]> for BoardLayout {
    fn from(input: [[u32; Self::USIZE]; Self::USIZE]) -> Self {
        BoardLayout::new(input)
    }
}

pub struct BoardState {
    layout: BoardLayout,
    horiz: [u8; BoardLayout::USIZE],
    vert: [u8; BoardLayout::USIZE],
}

impl BoardState {
    const WIN_VALUE: u8 = (1 << BoardLayout::SIZE) - 1;

    pub fn new(layout: BoardLayout) -> BoardState {
        BoardState {
            layout,
            horiz: Default::default(),
            vert: Default::default(),
        }
    }

    pub fn mark_number(&mut self, num: u32) -> bool {
        let (x, y) = match self.layout.get_pos_of(num) {
            Some(pos) => pos,
            None => return false,
        };

        fn mark_bit(bits: &mut u8, pos: u8) -> bool {
            *bits |= 1 << pos;
            *bits == BoardState::WIN_VALUE
        }

        let mut has_won = false;
        has_won |= mark_bit(&mut self.horiz[y as usize], x);
        has_won |= mark_bit(&mut self.vert[x as usize], y);
        has_won
    }

    pub fn has_won(&self) -> bool {
        self.horiz.contains(&Self::WIN_VALUE) || self.vert.contains(&Self::WIN_VALUE)
    }

    pub fn unmarked_sum(&self) -> u32 {
        let mut sum = 0;
        for (y, mark_row) in self.horiz.iter().enumerate() {
            for x in 0..BoardLayout::SIZE {
                if mark_row & (1 << x) == 0 {
                    sum += self.layout.get_num_at(x, y as u8);
                }
            }
        }
        sum
    }

    fn is_marked(&self, x: u8, y: u8) -> bool {
        (self.horiz[y as usize] & (1 << x)) != 0
    }

    pub fn print_row(row: &[Self]) {
        for y in 0..BoardLayout::SIZE {
            for (i, board) in row.iter().enumerate() {
                for x in 0..BoardLayout::SIZE {
                    let num = board.layout.get_num_at(x, y);
                    if board.is_marked(x, y) {
                        print!("({:2})", num);
                    } else {
                        print!(" {:2} ", num);
                    }
                }
                if i == row.len() - 1 {
                    println!();
                } else {
                    print!(" | ");
                }
            }
        }
    }

    pub fn print(&self) {
        Self::print_row(std::slice::from_ref(self));
    }
}

impl From<BoardLayout> for BoardState {
    fn from(layout: BoardLayout) -> Self {
        BoardState::new(layout)
    }
}

fn run(input: &str) {
    let mut lines = input.lines();

    let rand_numbers = lines
        .next()
        .unwrap()
        .split(',')
        .map(|n| n.parse::<u32>().unwrap());

    let mut boards: HashMap<usize, BoardState> = lines
        .chunks(6)
        .into_iter()
        .enumerate()
        .map(|(i, mut chunk)| {
            assert_eq!(chunk.next(), Some(""));
            let layout: BoardLayout = chunk
                .map(|r| {
                    r.split_ascii_whitespace()
                        .map(|n| n.parse().unwrap())
                        .collect::<CollectArrayResult<u32, { BoardLayout::USIZE }>>()
                        .unwrap()
                })
                .collect::<CollectArrayResult<_, { BoardLayout::USIZE }>>()
                .unwrap()
                .into();
            (i, layout.into())
        })
        .collect();

    for number in rand_numbers {
        if boards.is_empty() {
            println!("All done.");
            break;
        }
        println!("Running {}...", number);
        boards
            .drain_filter(|_, board| board.mark_number(number))
            .sorted_unstable_by_key(|(i, _)| *i)
            .for_each(|(i, board)| {
                let sum = board.unmarked_sum();
                println!(
                    "Board #{} wins! Score: {}*{} = {}",
                    i,
                    sum,
                    number,
                    sum * number,
                );
                board.print()
            });
    }
}

pub fn main() {
    run(include_str!("input.txt"));
}
