#![feature(box_patterns)]
use std::fmt::Display;
use std::ops::AddAssign;
use std::str::FromStr;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SfNumber {
    Pair(Box<(SfNumber, SfNumber)>),
    Plain(u32),
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct SfPath {
    bits: u8,
    len: u8,
}

impl SfPath {
    pub const fn empty() -> Self {
        Self { bits: 0, len: 0 }
    }

    pub const fn len(&self) -> u8 {
        self.len
    }

    pub const fn push_left(&self) -> Self {
        SfPath {
            bits: self.bits,
            len: self.len + 1,
        }
    }

    pub const fn push_right(&self) -> Self {
        SfPath {
            bits: self.bits | (1 << self.len),
            len: self.len + 1,
        }
    }

    pub fn pop_first(&mut self) -> Option<bool> {
        if self.len == 0 {
            None
        } else {
            let r = self.bits & 1 != 0;
            self.bits >>= 1;
            self.len -= 1;
            Some(r)
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum ReduceResult {
    Explode(Option<u32>, Option<u32>),
    Split(SfPath),
    None,
}

impl SfNumber {
    pub fn new_pair(a: Self, b: Self) -> Self {
        Self::Pair(Box::new((a, b)))
    }

    pub fn reduce(&mut self) {
        loop {
            match self.reduce_once(SfPath::empty()) {
                ReduceResult::Split(path) => {
                    let to_split = self.get_at_mut(path).unwrap();
                    let num = to_split.get_plain().unwrap();
                    let a = num / 2;
                    let b = num - a;
                    *to_split = Self::new_pair(Self::Plain(a), Self::Plain(b));
                }
                ReduceResult::Explode(_, _) => (),
                ReduceResult::None => break,
            };
        }
    }

    pub fn magnitude(&self) -> u32 {
        match self {
            Self::Pair(box (a, b)) => 3 * a.magnitude() + 2 * b.magnitude(),
            Self::Plain(n) => *n,
        }
    }

    pub fn get_at(&self, mut path: SfPath) -> Option<&Self> {
        let mut curr = self;
        while let Some(is_right) = path.pop_first() {
            if let Self::Pair(box (a, b)) = curr {
                curr = if is_right { b } else { a }
            } else {
                return None;
            }
        }
        Some(curr)
    }

    pub fn get_at_mut(&mut self, mut path: SfPath) -> Option<&mut Self> {
        let mut curr = self;
        while let Some(is_right) = path.pop_first() {
            if let Self::Pair(box (a, b)) = curr {
                curr = if is_right { b } else { a }
            } else {
                return None;
            }
        }
        Some(curr)
    }

    pub fn get_plain(&self) -> Option<u32> {
        if let Self::Plain(num) = self {
            Some(*num)
        } else {
            None
        }
    }

    pub fn sum(nums: impl IntoIterator<Item = Self>) -> Option<Self> {
        let mut nums = nums.into_iter();
        if let Some(mut sum) = nums.next() {
            for num in nums {
                sum += num;
            }
            Some(sum)
        } else {
            None
        }
    }

    pub fn sum_refs<'a>(nums: impl IntoIterator<Item = &'a Self>) -> Option<Self> {
        let mut nums = nums.into_iter();
        if let Some(sum) = nums.next() {
            let mut sum = sum.clone();
            for num in nums {
                sum += num.clone();
            }
            Some(sum)
        } else {
            None
        }
    }

    fn reduce_once(&mut self, path: SfPath) -> ReduceResult {
        match self {
            Self::Pair(box (a, b)) => {
                if path.len() >= 4 {
                    let a = a.get_plain().unwrap();
                    let b = b.get_plain().unwrap();
                    *self = Self::Plain(0);
                    ReduceResult::Explode(Some(a), Some(b))
                } else {
                    let mut r_a = a.reduce_once(path.push_left());
                    if let ReduceResult::Explode(_, e_b) = &mut r_a {
                        if let Some(n) = e_b.take() {
                            b.explode_into_leftmost(n);
                        }
                        return r_a;
                    }
                    let mut r_b = b.reduce_once(path.push_right());
                    if let ReduceResult::Explode(e_a, _) = &mut r_b {
                        if let Some(n) = e_a.take() {
                            a.explode_into_rightmost(n);
                        }
                        return r_b;
                    }
                    if r_a != ReduceResult::None {
                        r_a
                    } else {
                        r_b
                    }
                }
            }
            Self::Plain(num) => {
                if *num >= 10 {
                    ReduceResult::Split(path)
                } else {
                    ReduceResult::None
                }
            }
        }
    }

    fn explode_into_leftmost(&mut self, num: u32) {
        match self {
            SfNumber::Pair(box (a, _)) => a.explode_into_leftmost(num),
            SfNumber::Plain(n) => {
                *n += num;
            }
        }
    }

    fn explode_into_rightmost(&mut self, num: u32) {
        match self {
            SfNumber::Pair(box (_, b)) => b.explode_into_rightmost(num),
            SfNumber::Plain(n) => {
                *n += num;
            }
        }
    }

    fn parse_one(s: &mut &str) -> Result<Self, ()> {
        Ok(if let Some(rest) = s.strip_prefix('[') {
            *s = rest;
            let a = Self::parse_one(s)?;
            *s = s.strip_prefix(',').ok_or(())?;
            let b = Self::parse_one(s)?;
            *s = s.strip_prefix(']').ok_or(())?;
            Self::new_pair(a, b)
        } else {
            let end = s.find(&[',', ']']).unwrap_or(s.len());
            let (num, rest) = s.split_at(end);
            *s = rest;
            let num = num.parse().map_err(|_| ())?;
            Self::Plain(num)
        })
    }
}

impl AddAssign for SfNumber {
    fn add_assign(&mut self, rhs: Self) {
        let lhs = std::mem::replace(self, Self::Plain(0));
        *self = Self::new_pair(lhs, rhs);
        self.reduce();
    }
}

impl Display for SfNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pair(box (a, b)) => {
                write!(f, "[{},{}]", a, b)
            }
            Self::Plain(num) => write!(f, "{}", num),
        }
    }
}

impl FromStr for SfNumber {
    type Err = ();

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let r = Self::parse_one(&mut s);
        if s.is_empty() {
            r
        } else {
            Err(())
        }
    }
}

fn main() {
    let nums: Vec<SfNumber> = include_str!("input.txt")
        .lines()
        .map(|l| l.parse().unwrap())
        .collect();

    let full_sum = SfNumber::sum_refs(&nums).unwrap();
    println!("P1: magnitude of {} is {}", full_sum, full_sum.magnitude());

    let mut max_magnitude = 0;
    for a in &nums {
        for b in &nums {
            if a != b {
                let mut sum = a.clone();
                sum += b.clone();
                let mag = sum.magnitude();
                max_magnitude = max_magnitude.max(mag);
            }
        }
    }
    println!("P2: max possible magnitude is {}", max_magnitude);
}

#[cfg(test)]
mod tests {
    use super::SfNumber;

    fn n(s: &str) -> SfNumber {
        s.parse().unwrap()
    }

    fn check_equal(a: &SfNumber, b: &str) {
        let b = n(b);
        assert!(*a == b, "{} != {}", a, b);
    }

    #[test]
    pub fn explode_no_left() {
        let mut x = n("[[[[[9,8],1],2],3],4]");
        x.reduce();
        check_equal(&x, "[[[[0,9],2],3],4]");
    }

    #[test]
    pub fn explode_no_right() {
        let mut x = n("[7,[6,[5,[4,[3,2]]]]]");
        x.reduce();
        check_equal(&x, "[7,[6,[5,[7,0]]]]");
    }

    #[test]
    pub fn explode_once() {
        let mut x = n("[[6,[5,[4,[3,2]]]],1]");
        x.reduce();
        check_equal(&x, "[[6,[5,[7,0]]],3]");
    }

    #[test]
    pub fn explode_twice() {
        let mut x = n("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]");
        x.reduce();
        check_equal(&x, "[[3,[2,[8,0]]],[9,[5,[7,0]]]]");
    }

    #[test]
    pub fn split() {
        let mut x = n("11");
        x.reduce();
        check_equal(&x, "[5,6]");
    }

    #[test]
    pub fn basic_sum_1() {
        let mut sum = n("[1,1]");
        sum += n("[2,2]");
        sum += n("[3,3]");
        sum += n("[4,4]");
        sum += n("[5,5]");
        check_equal(&sum, "[[[[3,0],[5,3]],[4,4]],[5,5]]");
    }

    #[test]
    pub fn basic_sum_2() {
        let mut sum = n("[1,1]");
        sum += n("[2,2]");
        sum += n("[3,3]");
        sum += n("[4,4]");
        sum += n("[5,5]");
        sum += n("[6,6]");
        check_equal(&sum, "[[[[5,0],[7,4]],[5,5]],[6,6]]");
    }

    #[test]
    pub fn basic_sum_3() {
        let mut sum = n("[[[[4,3],4],4],[7,[[8,4],9]]]");
        sum += n("[1,1]");
        check_equal(&sum, "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
    }

    #[test]
    pub fn basic_sum_4() {
        let mut sum = n("[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]");
        sum += n("[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]");
        check_equal(
            &sum,
            "[[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]",
        );
    }
}
