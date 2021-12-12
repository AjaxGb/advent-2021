#![feature(bool_to_option)]
#![feature(iter_partition_in_place)]
use std::{borrow::Borrow, convert::TryInto};

fn most_common_bits<const BIT_WIDTH: usize>(
    input: impl IntoIterator<Item = impl Borrow<[bool; BIT_WIDTH]>>,
) -> [bool; BIT_WIDTH] {
    let mut bit_counts = [0u32; BIT_WIDTH];
    let mut total_count = 0u32;
    for bits in input {
        for (i, bit) in bits.borrow().iter().copied().enumerate() {
            if bit {
                bit_counts[i] += 1;
            }
        }
        total_count += 1;
    }
    let goal_count = (total_count as u32).checked_sub(1).unwrap() / 2;
    bit_counts.map(|c| c > goal_count)
}

fn str_to_bits<const BIT_WIDTH: usize>(text: &str) -> [bool; BIT_WIDTH] {
    let bytes: &[u8; BIT_WIDTH] = text.as_bytes().try_into().unwrap();
    bytes.map(|b| b != '0' as u8)
}

fn bits_to_int(input: impl IntoIterator<Item = impl Borrow<bool>>) -> u32 {
    let mut value = 0u32;
    for bit in input.into_iter() {
        value <<= 1;
        if *bit.borrow() {
            value |= 1;
        }
    }
    value
}

#[must_use]
fn filter_successive_bits<const BIT_WIDTH: usize>(
    mut input: &mut [[bool; BIT_WIDTH]],
    least_common: bool,
    start: usize,
) -> &[bool; BIT_WIDTH] {
    let mut i = start;
    loop {
        if let [result] = input {
            break result;
        }
        let bit_count = input.iter().filter_map(|b| b[i].then_some(())).count();
        let goal_count = input.len().checked_sub(1).unwrap() / 2;
        let goal_bit = (bit_count > goal_count) ^ least_common;
        let mid = input.iter_mut().partition_in_place(|b| b[i] == goal_bit);
        input = &mut input[..mid];
        i += 1;
    }
}

pub fn main() {
    const BIT_WIDTH: usize = 12;
    const BIT_MASK: u32 = (1 << BIT_WIDTH) - 1;

    let mut input: Vec<[_; BIT_WIDTH]> =
        include_str!("input.txt").lines().map(str_to_bits).collect();
    let most_common_bits = most_common_bits(&input);

    {
        let gamma = bits_to_int(&most_common_bits);
        let epsilon = !gamma & BIT_MASK;

        println!(
            "P1: gamma={:b} ({}), epsilon={:b}, ({}), product={}",
            gamma,
            gamma,
            epsilon,
            epsilon,
            gamma * epsilon,
        );
    }

    {
        let mid = input
            .iter_mut()
            .partition_in_place(|b| b[0] == most_common_bits[0]);
        let (oxygen_values, co2_values) = input.as_mut_slice().split_at_mut(mid);

        let oxygen_value = filter_successive_bits(oxygen_values, false, 1);
        let co2_value = filter_successive_bits(co2_values, true, 1);
        let oxygen_value = bits_to_int(oxygen_value);
        let co2_value = bits_to_int(co2_value);

        println!(
            "P2: oxygen={:b} ({}), CO2={:b}, ({}), product={}",
            oxygen_value,
            oxygen_value,
            co2_value,
            co2_value,
            oxygen_value * co2_value,
        );
    }
}
