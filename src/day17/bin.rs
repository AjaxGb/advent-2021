use std::ops::RangeInclusive;

pub const fn triangular_number(n: i32) -> i32 {
    n * (n + 1) / 2
}

pub fn simulate_launch(
    vel: (i32, i32),
    target: (&RangeInclusive<i32>, &RangeInclusive<i32>),
) -> Option<i32> {
    let (mut ax, mut ay) = vel;
    let (target_x, target_y) = target;
    let mut x = 0;
    let mut y = 0;
    let mut max_y = y;
    while x <= *target_x.end() && y >= *target_y.start() {
        if target_x.contains(&x) && target_y.contains(&y) {
            return Some(max_y);
        }
        x += ax;
        y += ay;
        max_y = max_y.max(y);
        if ax > 0 {
            ax -= 1;
        }
        ay -= 1;
    }
    None
}

pub fn main() {
    let (target_x, target_y) = include_str!("input.txt")
        .trim_end()
        .strip_prefix("target area: x=")
        .unwrap()
        .split_once(", y=")
        .unwrap();
    let [target_x, target_y] = [target_x, target_y].map(|r| {
        let (min, max) = r.split_once("..").unwrap();
        min.parse::<i32>().unwrap()..=max.parse::<i32>().unwrap()
    });

    let x_vel_range = {
        let mut i = 1;
        let min = loop {
            if triangular_number(i) >= *target_x.start() {
                break i;
            }
            i += 1;
        };
        min..=*target_x.end()
    };
    let y_vel_range = *target_y.start()..(-*target_y.start());

    let mut curr_max = None;
    let mut num_solutions = 0;
    for vel_y in y_vel_range {
        for vel_x in x_vel_range.clone() {
            let vel = (vel_x, vel_y);
            let target = (&target_x, &target_y);
            if let Some(max_y) = simulate_launch(vel, target) {
                if curr_max.map(|(_, prev)| max_y > prev).unwrap_or(true) {
                    curr_max = Some((vel, max_y));
                }
                num_solutions += 1;
            }
        }
    }

    let (vel, max_y) = curr_max.unwrap();
    println!("P1: max Y ({}) occurs at {:?}", max_y, vel);
    println!("P2: {} possible solutions", num_solutions);
}
