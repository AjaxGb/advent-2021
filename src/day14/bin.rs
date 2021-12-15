use std::collections::HashMap;

use itertools::Itertools;

pub fn grow_polymer(polymer: &mut String, rules: &HashMap<(u8, u8), u8>) {
    unsafe {
        let polymer = polymer.as_mut_vec();
        let num_old_chars = polymer.len();
        let num_new_chars = num_old_chars.saturating_sub(1);
        polymer.resize(num_old_chars + num_new_chars, 0);
        for i in (1..num_old_chars).rev() {
            polymer[i * 2] = polymer[i];
            polymer[i * 2 - 1] = rules[&(polymer[i - 1], polymer[i])];
        }
    }
}

pub fn main() {
    let mut lines = include_str!("input.txt").lines();
    let mut polymer = lines.next().unwrap().to_string();
    lines.next().unwrap();

    let rules = lines
        .map(|line| {
            let (in_chars, out_char) = line.split_once(" -> ").unwrap();
            let [in_a, in_b]: [u8; 2] = in_chars.as_bytes().try_into().unwrap();
            let [out_char]: [u8; 1] = out_char.as_bytes().try_into().unwrap();
            ((in_a, in_b), out_char)
        })
        .collect();

    for _ in 0..10 {
        grow_polymer(&mut polymer, &rules);
    }

    let (min, max) = polymer
        .chars()
        .counts()
        .into_iter()
        .minmax_by_key(|(_, c)| *c)
        .into_option()
        .unwrap();
    println!("P1: {:?} - {:?} = {}", max, min, max.1 - min.1);
}
