use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Cave {
    // Start is not included, as it is not a valid destination
    End,
    Big(usize),
    Small(usize),
}

#[derive(Debug, Default, Clone)]
pub struct CaveSystem {
    big: Vec<Vec<Cave>>,
    small: Vec<Vec<Cave>>,
    start: Vec<Cave>,
}

impl CaveSystem {
    pub fn count_paths(&self) -> (u32, u32) {
        let mut num_paths_no_twice = 0;
        let mut num_paths_with_twice = 0;

        struct ToVisit {
            index: usize,
            is_small: bool,
            small_visited: u64,
            twice_unused: bool,
        }

        let mut to_visit = Vec::with_capacity(64);
        for dest in &self.start {
            let (index, is_small) = match *dest {
                Cave::End => {
                    num_paths_no_twice += 1;
                    num_paths_with_twice += 1;
                    continue;
                }
                Cave::Big(index) => (index, false),
                Cave::Small(index) => (index, true),
            };
            to_visit.push(ToVisit {
                index,
                is_small,
                small_visited: 0,
                twice_unused: true,
            });
        }

        assert!(
            self.small.len() < 64,
            "64 or more small caves will break the 'visited' bitfield"
        );

        while let Some(curr) = to_visit.pop() {
            let mut small_visited = curr.small_visited;
            let all_dest = if curr.is_small {
                small_visited |= 1 << curr.index;
                &self.small[curr.index]
            } else {
                &self.big[curr.index]
            };
            for dest in all_dest {
                match *dest {
                    Cave::End => {
                        num_paths_with_twice += 1;
                        if curr.twice_unused {
                            num_paths_no_twice += 1;
                        }
                    }
                    Cave::Big(index) => {
                        to_visit.push(ToVisit {
                            index,
                            is_small: false,
                            small_visited,
                            twice_unused: curr.twice_unused,
                        });
                    }
                    Cave::Small(index) => {
                        let first_visit = small_visited & (1 << index) == 0;
                        let twice_unused = if first_visit {
                            curr.twice_unused
                        } else if curr.twice_unused {
                            false
                        } else {
                            continue;
                        };
                        to_visit.push(ToVisit {
                            index,
                            is_small: true,
                            small_visited,
                            twice_unused,
                        });
                    }
                }
            }
        }
        (num_paths_no_twice, num_paths_with_twice)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CaveSystemBuilder<'a> {
    caves: CaveSystem,
    cave_ids: HashMap<&'a str, Cave>,
}

impl<'a> CaveSystemBuilder<'a> {
    pub fn new() -> CaveSystemBuilder<'a> {
        Default::default()
    }

    pub fn add_connection(&mut self, a: &'a str, b: &'a str) {
        assert_ne!(a, b);
        let a = self.get_cave(a);
        let b = self.get_cave(b);
        self.add_connection_one_way(a, b);
        self.add_connection_one_way(b, a);
    }

    pub fn build(self) -> CaveSystem {
        self.caves
    }

    fn get_cave(&mut self, name: &'a str) -> Option<Cave> {
        match name {
            "start" => None,
            "end" => Some(Cave::End),
            _ => Some(*self.cave_ids.entry(name).or_insert_with(|| {
                if name.chars().next().unwrap().is_lowercase() {
                    let index = self.caves.small.len();
                    self.caves.small.push(vec![]);
                    Cave::Small(index)
                } else {
                    let index = self.caves.big.len();
                    self.caves.big.push(vec![]);
                    Cave::Big(index)
                }
            })),
        }
    }

    fn add_connection_one_way(&mut self, from: Option<Cave>, to: Option<Cave>) {
        if let Some(to) = to {
            match from {
                None => self.caves.start.push(to),
                Some(Cave::Big(i)) => self.caves.big[i].push(to),
                Some(Cave::Small(i)) => self.caves.small[i].push(to),
                Some(Cave::End) => (),
            }
        }
    }
}

pub fn main() {
    let before_build_time = Instant::now();

    let mut builder = CaveSystemBuilder::new();
    for line in include_str!("input.txt").lines() {
        let (a, b) = line.split_once('-').unwrap();
        builder.add_connection(a, b);
    }
    let caves = builder.build();

    let after_build_time = Instant::now();

    let (num_paths_p1, num_paths_p2) = caves.count_paths();

    let done_time = Instant::now();
    println!(
        "Runtime: {:.3}ms to build graph, {:.3}ms to run, {:.3}ms total",
        (after_build_time - before_build_time).as_secs_f64() * 1000.0,
        (done_time - after_build_time).as_secs_f64() * 1000.0,
        (done_time - before_build_time).as_secs_f64() * 1000.0,
    );

    println!("P1: {} paths", num_paths_p1);
    println!("P2: {} paths", num_paths_p2);
}
