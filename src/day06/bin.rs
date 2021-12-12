use std::borrow::Borrow;
use std::collections::HashMap;

const REPEAT_SPAWN_DELAY: u32 = 7;
const NEW_SPAWN_DELAY: u32 = 9;

pub fn fish_after_n_days(fish: impl IntoIterator<Item = impl Borrow<u32>>, num_days: u32) -> usize {
    let mut spawn_times: HashMap<u32, usize> = HashMap::new();
    for delay in fish.into_iter() {
        *spawn_times.entry(*delay.borrow()).or_default() += 1;
    }

    for day in 0..num_days {
        if let Some(count) = spawn_times.remove(&day) {
            *spawn_times.entry(day + REPEAT_SPAWN_DELAY).or_default() += count;
            *spawn_times.entry(day + NEW_SPAWN_DELAY).or_default() += count;
        }
    }

    spawn_times.drain().map(|(_, c)| c).sum()
}

pub fn main() {
    let input: Vec<u32> = include_str!("input.txt")
        .trim()
        .split(',')
        .map(|n| n.parse().unwrap())
        .collect();

    println!("P1: after 80 days: {}", fish_after_n_days(&input, 80));
    println!("P2: after 256 days: {}", fish_after_n_days(&input, 256));
}
