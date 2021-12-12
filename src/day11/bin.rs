use std::collections::HashSet;

use collect_array::CollectArrayResult;

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

pub struct Octopi {
    energy: [[u8; WIDTH]; HEIGHT],
}

impl Octopi {
    pub fn new(energy: [[u8; WIDTH]; HEIGHT]) -> Self {
        Octopi { energy }
    }

    pub fn increment_all(&mut self) -> u32 {
        let mut to_reset: HashSet<(usize, usize)> = HashSet::new();
        let mut flashes = 0;
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                flashes += self.increment(x, y, &mut to_reset);
            }
        }
        for (x, y) in to_reset {
            self.energy[y][x] = 0;
        }
        flashes
    }

    fn increment(&mut self, x: usize, y: usize, mut to_reset: &mut HashSet<(usize, usize)>) -> u32 {
        let mut flashes = 0;
        if self.energy[y][x] == 9 {
            if to_reset.insert((x, y)) {
                flashes += 1;

                let min_x = x.saturating_sub(1);
                let max_x = x.saturating_add(1).min(WIDTH - 1);
                let min_y = y.saturating_sub(1);
                let max_y = y.saturating_add(1).min(HEIGHT - 1);
                for adj_y in min_y..=max_y {
                    for adj_x in min_x..=max_x {
                        if adj_x != x || adj_y != y {
                            flashes += self.increment(adj_x, adj_y, &mut to_reset);
                        }
                    }
                }
            }
        } else {
            self.energy[y][x] += 1;
        }
        flashes
    }

    fn print(&self) {
        for row in self.energy {
            for e in row {
                print!("{}", e);
            }
            println!();
        }
    }
}

pub fn main() {
    let input: CollectArrayResult<_, HEIGHT> = include_str!("input.txt")
        .lines()
        .map(|r| {
            let r: &[u8; WIDTH] = r.as_bytes().try_into().unwrap();
            r.map(|b| b - b'0')
        })
        .collect();
    let mut octopi = Octopi::new(input.unwrap());

    octopi.print();

    let mut flashes = 0;
    let mut first_all_flash = None;
    for i in 1..=100 {
        let curr_flashes = octopi.increment_all();
        if curr_flashes == 100 {
            first_all_flash.get_or_insert(i);
        }
        flashes += curr_flashes;
    }

    println!("P1: {} flashes after 100 increments", flashes);

    let first_all_flash = first_all_flash.unwrap_or_else(|| {
        for i in 101..=u64::MAX {
            if octopi.increment_all() == 100 {
                return i;
            }
        }
        panic!()
    });
    println!("P1: {} steps to all flash", first_all_flash);
}
