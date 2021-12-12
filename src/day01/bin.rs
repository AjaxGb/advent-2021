use itertools::Itertools;

fn count_increases(input: impl IntoIterator<Item = u32>) -> usize {
    input
        .into_iter()
        .tuple_windows()
        .filter(|(a, b)| a < b)
        .count()
}

fn count_sum3_increases(input: impl IntoIterator<Item = u32>) -> usize {
    count_increases(input.into_iter().tuple_windows().map(|(a, b, c)| a + b + c))
}

fn main() {
    let input: Vec<u32> = include_str!("input.txt")
        .lines()
        .map(|t| t.parse().unwrap())
        .collect();
    println!("P1: {} increases", count_increases(input.iter().cloned()));
    println!(
        "P2: {} increases",
        count_sum3_increases(input.iter().cloned())
    );
}
