use std::borrow::Borrow;

mod p1 {
    pub fn dist(a: u32, b: u32) -> u32 {
        if a > b {
            a - b
        } else {
            b - a
        }
    }
}

mod p2 {
    pub fn dist(a: u32, b: u32) -> u32 {
        let d = super::p1::dist(a, b);
        (d * d + d) / 2
    }
}

pub fn fuel_to(
    crabs: impl IntoIterator<Item = impl Borrow<u32>>,
    dist: impl Fn(u32, u32) -> u32,
    point: u32,
) -> u32 {
    crabs.into_iter().map(|n| dist(*n.borrow(), point)).sum()
}

pub fn main() {
    let mut crabs: Vec<u32> = include_str!("input.txt")
        .trim()
        .split(',')
        .map(|n| n.parse().unwrap())
        .collect();

    crabs.sort();
    let target_pos = crabs[crabs.len() / 2];
    let fuel_usage: u32 = fuel_to(&crabs, p1::dist, target_pos);
    println!("P1: {} fuel to {}", fuel_usage, target_pos);

    let sum: u32 = crabs.iter().copied().sum();
    let mean_floor = sum / crabs.len() as u32;
    let mean_ceil = mean_floor + 1;
    let mean_floor_fuel = fuel_to(&crabs, p2::dist, mean_floor);
    let mean_ceil_fuel = fuel_to(&crabs, p2::dist, mean_ceil);
    let (target_pos, fuel_usage) = if mean_ceil_fuel < mean_floor_fuel {
        (mean_ceil, mean_ceil_fuel)
    } else {
        (mean_floor, mean_floor_fuel)
    };
    println!("P2: {} fuel to {}", fuel_usage, target_pos);
}
