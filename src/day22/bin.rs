#![feature(array_from_fn)]

const fn const_min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

const fn const_max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct Vec3([i32; 3]);

impl Vec3 {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self([x, y, z])
    }

    pub const fn new_uniform(u: i32) -> Self {
        Self([u, u, u])
    }

    pub const fn x(&self) -> i32 {
        self.0[0]
    }

    pub const fn y(&self) -> i32 {
        self.0[1]
    }

    pub const fn z(&self) -> i32 {
        self.0[2]
    }

    pub const fn axes(&self) -> &[i32; 3] {
        &self.0
    }

    #[must_use]
    pub const fn min(&self, rhs: &Self) -> Self {
        Self([
            const_min(self.0[0], rhs.0[0]),
            const_min(self.0[1], rhs.0[1]),
            const_min(self.0[2], rhs.0[2]),
        ])
    }

    #[must_use]
    pub const fn max(&self, rhs: &Self) -> Self {
        Self([
            const_max(self.0[0], rhs.0[0]),
            const_max(self.0[1], rhs.0[1]),
            const_max(self.0[2], rhs.0[2]),
        ])
    }

    #[must_use]
    pub const fn minus(&self, rhs: &Self) -> Self {
        Self([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }

    #[must_use]
    pub const fn plus(&self, rhs: &Self) -> Self {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }

    #[must_use]
    pub const fn div_scalar(&self, rhs: i32) -> Self {
        Self([self.0[0] / rhs, self.0[1] / rhs, self.0[2] / rhs])
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Cuboid {
    min: Vec3,
    max: Vec3,
}

impl Cuboid {
    pub const fn new(min: Vec3, max: Vec3) -> Option<Self> {
        if min.x() < max.x() && min.y() < max.y() && min.z() < max.z() {
            Some(Self { min, max })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn intersection(&self, rhs: &Self) -> Option<Self> {
        Self::new(self.min.max(&rhs.min), self.max.min(&rhs.max))
    }

    #[must_use]
    pub const fn union(&self, rhs: &Self) -> Self {
        Self {
            min: self.min.min(&rhs.min),
            max: self.max.max(&rhs.max),
        }
    }

    pub const fn volume(&self) -> u64 {
        let d = self.max.minus(&self.min);
        let [dx, dy, dz] = d.axes();
        dx.abs() as u64 * dy.abs() as u64 * dz.abs() as u64
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReactorCore {
    bounds: Option<Cuboid>,
    cuboids: Vec<(Cuboid, bool)>,
    num_on: u64,
}

impl ReactorCore {
    pub fn new(bounds: Option<Cuboid>) -> Self {
        ReactorCore {
            bounds,
            cuboids: vec![],
            num_on: 0,
        }
    }

    fn bounds_intersection(&self, cuboid: Cuboid) -> Option<Cuboid> {
        if let Some(bounds) = &self.bounds {
            cuboid.intersection(bounds)
        } else {
            Some(cuboid)
        }
    }

    pub fn add_cuboid(&mut self, cuboid: Cuboid, is_on: bool) {
        if let Some(cuboid) = self.bounds_intersection(cuboid) {
            for i in 0..self.cuboids.len() {
                let (overlap, is_added) = {
                    let (exist_cuboid, exist_is_added) = &self.cuboids[i];
                    if let Some(overlap) = cuboid.intersection(&exist_cuboid) {
                        (overlap, !exist_is_added)
                    } else {
                        continue;
                    }
                };
                if is_added {
                    self.num_on += overlap.volume();
                } else {
                    self.num_on -= overlap.volume();
                }
                self.cuboids.push((overlap, is_added));
            }
            if is_on {
                self.num_on += cuboid.volume();
                self.cuboids.push((cuboid, is_on));
            }
        }
    }

    pub const fn num_on(&self) -> u64 {
        self.num_on
    }
}

fn main() {
    let cuboids: Vec<_> = include_str!("input.txt")
        .lines()
        .map(|t| {
            let (on_off, cuboid) = t.split_once(' ').unwrap();
            let (x_range, rest) = cuboid.split_once(',').unwrap();
            let (y_range, z_range) = rest.split_once(',').unwrap();
            let [x_range, y_range, z_range] = [x_range, y_range, z_range].map(|range| {
                let (_, range) = range.split_once('=').unwrap();
                let (min, max) = range.split_once("..").unwrap();
                let min: i32 = min.parse().unwrap();
                let max: i32 = max.parse().unwrap();
                (min, max + 1)
            });
            let min = Vec3::new(x_range.0, y_range.0, z_range.0);
            let max = Vec3::new(x_range.1, y_range.1, z_range.1);
            (Cuboid::new(min, max).unwrap(), on_off == "on")
        })
        .collect();

    let bounds = Cuboid::new(Vec3::new_uniform(-50), Vec3::new_uniform(51)).unwrap();
    let mut core = ReactorCore::new(Some(bounds));

    for (cuboid, is_on) in &cuboids {
        core.add_cuboid(cuboid.clone(), *is_on);
    }
    println!("P1: {} cubes on", core.num_on());

    let mut core = ReactorCore::new(None);
    for (cuboid, is_on) in &cuboids {
        core.add_cuboid(cuboid.clone(), *is_on);
    }
    println!("P2: {} cubes on", core.num_on());
}
