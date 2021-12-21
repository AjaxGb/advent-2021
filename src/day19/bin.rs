use std::collections::{HashMap, HashSet};

use itertools::Itertools;

const INT_SINES: [i32; 4] = [0, 1, 0, -1];

pub const fn int_sin(right_angles: i32) -> i32 {
    INT_SINES[right_angles.rem_euclid(4) as usize]
}

pub const fn int_cos(right_angles: i32) -> i32 {
    INT_SINES[(right_angles + 1).rem_euclid(4) as usize]
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct Vec3([i32; 3]);

impl Vec3 {
    pub const fn from_axes(x: i32, y: i32, z: i32) -> Self {
        Self([x, y, z])
    }

    pub const fn plus(&self, rhs: &Self) -> Self {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }

    pub const fn minus(&self, rhs: &Self) -> Self {
        Self([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }

    pub const fn manhattan_len(&self) -> u32 {
        self.0[0].unsigned_abs() + self.0[1].unsigned_abs() + self.0[2].unsigned_abs()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Rotation([[i32; 3]; 3]);

impl Rotation {
    pub const NONE: Self = Self([[1, 0, 0], [0, 1, 0], [0, 0, 1]]);

    pub const SIDES: [Self; 6] = [
        Self::NONE,
        Self::from_y_angle(1),
        Self::from_y_angle(-1),
        Self::from_y_angle(2),
        Self::from_x_angle(1),
        Self::from_x_angle(-1),
    ];

    pub const SPINS: [Self; 4] = [
        Self::NONE,
        Self::from_z_angle(1),
        Self::from_z_angle(2),
        Self::from_z_angle(3),
    ];

    pub const fn rows(&self) -> &[[i32; 3]; 3] {
        &self.0
    }

    #[rustfmt::skip]
    pub const fn from_x_angle(right_angles: i32) -> Self {
        let c = int_cos(right_angles);
        let s = int_sin(right_angles);
        Self([
            [1, 0,  0],
            [0, c, -s],
            [0, s,  c],
        ])
    }

    #[rustfmt::skip]
    pub const fn from_y_angle(right_angles: i32) -> Self {
        let c = int_cos(right_angles);
        let s = int_sin(right_angles);
        Self([
            [ c, 0, s],
            [ 0, 1, 0],
            [-s, 0, c],
        ])
    }

    #[rustfmt::skip]
    pub const fn from_z_angle(right_angles: i32) -> Self {
        let c = int_cos(right_angles);
        let s = int_sin(right_angles);
        Self([
            [c, -s, 0],
            [s,  c, 0],
            [0,  0, 1],
        ])
    }

    pub const fn mult(&self, rhs: &Self) -> Self {
        Self(mat_mul(&self.0, &rhs.0))
    }

    pub const fn rotate(&self, point: &Vec3) -> Vec3 {
        Vec3(vec_mul(&self.0, &point.0))
    }
}

pub const fn mat_mul<const H: usize, const S: usize, const W: usize>(
    a: &[[i32; S]; H],
    b: &[[i32; W]; S],
) -> [[i32; W]; H] {
    let mut r = [[0; W]; H];

    let mut y = 0;
    loop {
        if y == H {
            break;
        }
        let mut x = 0;
        loop {
            if x == W {
                break;
            }
            let mut s = 0;
            loop {
                if s == S {
                    break;
                }

                r[y][x] += a[y][s] * b[s][x];

                s += 1;
            }
            x += 1;
        }
        y += 1;
    }
    r
}

pub const fn vec_mul<const H: usize, const S: usize>(a: &[[i32; S]; H], b: &[i32; S]) -> [i32; H] {
    let mut r = [0; H];

    let mut y = 0;
    loop {
        if y == H {
            break;
        }
        let mut s = 0;
        loop {
            if s == S {
                break;
            }

            r[y] += a[y][s] * b[s];

            s += 1;
        }
        y += 1;
    }
    r
}

pub fn match_beacons(fixed: &HashSet<Vec3>, unfixed: &HashSet<Vec3>) -> Option<(Vec3, Vec<Vec3>)> {
    let mut new_pos = Vec::with_capacity(unfixed.len());
    for side in Rotation::SIDES {
        for spin in Rotation::SPINS {
            let rot = side.mult(&spin);
            for ufx in unfixed {
                let ufx = rot.rotate(ufx);
                for fx in fixed {
                    let offset = fx.minus(&ufx);
                    new_pos.clear();
                    for ufx in unfixed {
                        let ufx = rot.rotate(ufx).plus(&offset);
                        if !fixed.contains(&ufx) {
                            new_pos.push(ufx);
                        }
                    }
                    if unfixed.len() - new_pos.len() >= 12 {
                        return Some((offset, new_pos));
                    }
                }
            }
        }
    }
    None
}

fn main() {
    let mut lines = include_str!("input.txt").lines();
    let mut unfixed_scanners: HashMap<u32, HashSet<Vec3>> = HashMap::new();
    while let Some(line) = lines.next() {
        let index = line
            .strip_prefix("--- scanner ")
            .unwrap()
            .strip_suffix(" ---")
            .unwrap()
            .parse()
            .unwrap();

        let mut beacons = HashSet::new();
        while let Some(line) = lines.next() {
            if line.is_empty() {
                break;
            }
            let (x, rest) = line.split_once(',').unwrap();
            let (y, z) = rest.split_once(',').unwrap();
            beacons.insert(Vec3::from_axes(
                x.parse().unwrap(),
                y.parse().unwrap(),
                z.parse().unwrap(),
            ));
        }

        unfixed_scanners.insert(index, beacons);
    }

    let mut fixed_beacons = HashSet::new();
    fixed_beacons.extend(unfixed_scanners.remove(&0).unwrap());
    let mut fixed_scanners = HashMap::new();
    fixed_scanners.insert(0, Vec3::from_axes(0, 0, 0));

    while !unfixed_scanners.is_empty() {
        println!("{}...", unfixed_scanners.len());
        unfixed_scanners.retain(|i, unfixed_beacons| {
            if let Some((offset, new_pos)) = match_beacons(&fixed_beacons, &unfixed_beacons) {
                fixed_beacons.extend(new_pos);
                fixed_scanners.insert(*i, offset);
                false
            } else {
                true
            }
        });
    }

    println!("P1: there are {} beacons", fixed_beacons.len());

    let max_dist_pair = fixed_scanners
        .iter()
        .tuple_combinations()
        .max_by_key(|((_, a), (_, b))| a.minus(&b).manhattan_len())
        .unwrap();
    let ((a_i, a_pos), (b_i, b_pos)) = max_dist_pair;
    let max_dist = a_pos.minus(b_pos).manhattan_len();
    println!(
        "P2: the max distance is {} between {} at {:?} and {} at {:?}",
        max_dist, a_i, a_pos, b_i, b_pos,
    );
}
