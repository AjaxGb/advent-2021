#![feature(hash_drain_filter)]
#![feature(array_zip)]
use std::collections::HashSet;

pub fn main() {
    let mut lines = include_str!("input.txt").lines();

    let mut points: HashSet<[u32; 2]> = lines
        .by_ref()
        .map_while(|l| {
            if let Some((x, y)) = l.split_once(',') {
                let x = x.parse().unwrap();
                let y = y.parse().unwrap();
                Some([x, y])
            } else {
                None
            }
        })
        .collect();

    println!("At start, there are {} points", points.len());

    let mut to_fold = vec![];

    for fold in lines {
        let (axis_name, fold_pos) = fold
            .strip_prefix("fold along ")
            .unwrap()
            .split_once('=')
            .unwrap();
        let fold_pos: u32 = fold_pos.parse().unwrap();
        let axis = if axis_name == "x" {
            0
        } else {
            assert_eq!(axis_name, "y");
            1
        };
        for pos in points.drain_filter(|pos| pos[axis] > fold_pos) {
            to_fold.push(pos);
        }
        for mut pos in to_fold.drain(0..) {
            pos[axis] = 2 * fold_pos - pos[axis];
            points.insert(pos);
        }

        println!(
            "After {}={}, there are {} points",
            axis_name,
            fold_pos,
            points.len()
        );
    }

    let [max_x, max_y] = points
        .iter()
        .copied()
        .fold([0, 0], |max, curr| max.zip(curr).map(|(a, b)| a.max(b)));
    for y in 0..=max_y {
        for x in 0..=max_x {
            print!("{}", if points.contains(&[x, y]) { '#' } else { ' ' });
        }
        println!();
    }
}
