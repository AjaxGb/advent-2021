use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RuleCounter {
    rules: HashMap<(char, char), char>,
    counts: HashMap<(char, char, u32), HashMap<char, u64>>,
}

impl RuleCounter {
    pub fn new(rules: HashMap<(char, char), char>) -> Self {
        Self {
            rules,
            counts: Default::default(),
        }
    }

    pub fn count_polymer(&mut self, polymer: &[char], iters: u32) -> HashMap<char, u64> {
        let mut counts = HashMap::new();
        if !polymer.is_empty() {
            counts.insert(polymer[0], 1);
            for (a, b) in polymer.into_iter().copied().tuple_windows() {
                for (c, count) in self.get_counts_excl_a(a, b, iters) {
                    *counts.entry(*c).or_default() += count;
                }
            }
        }
        counts
    }

    fn get_counts_excl_a(&mut self, a: char, b: char, iters: u32) -> &HashMap<char, u64> {
        if !self.counts.contains_key(&(a, b, iters)) {
            let result = if iters == 0 {
                let mut c = HashMap::with_capacity(1);
                c.insert(b, 1);
                c
            } else {
                let mid = self.rules[&(a, b)];
                let mut c = self.get_counts_excl_a(a, mid, iters - 1).clone();
                for (lower, lower_count) in self.get_counts_excl_a(mid, b, iters - 1) {
                    *c.entry(*lower).or_default() += lower_count;
                }
                c
            };
            self.counts.insert((a, b, iters), result);
        }
        self.counts.get(&(a, b, iters)).unwrap()
    }
}

pub fn main() {
    let mut lines = include_str!("input.txt").lines();
    let polymer = lines.next().unwrap().chars().collect_vec();
    lines.next().unwrap();

    let mut rule_counter = RuleCounter::new(
        lines
            .map(|line| {
                let (in_chars, out_char) = line.split_once(" -> ").unwrap();
                let (in_a, in_b) = in_chars.chars().collect_tuple().unwrap();
                let (out_char,) = out_char.chars().collect_tuple().unwrap();
                ((in_a, in_b), out_char)
            })
            .collect(),
    );

    for (i, num_iters) in [(1, 10), (2, 40)] {
        let counts = rule_counter.count_polymer(polymer.as_slice(), num_iters);
        println!("{:?}", counts);
        let (min, max) = counts
            .into_iter()
            .minmax_by_key(|(_, c)| *c)
            .into_option()
            .unwrap();
        println!("P{}: {:?} - {:?} = {}", i, max, min, max.1 - min.1);
    }
}
