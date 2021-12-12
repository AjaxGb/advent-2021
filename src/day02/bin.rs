use std::error::Error;
use std::fmt::Display;
use std::{num::ParseIntError, str::FromStr};

pub mod p1 {
    use super::SubCommand;
    use std::borrow::Borrow;

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct Sub {
        horizontal: u32,
        depth: u32,
    }

    impl Sub {
        pub fn new() -> Self {
            Sub::default()
        }

        pub fn follow_command(&mut self, command: &SubCommand) {
            match command {
                SubCommand::Forward(dist) => self.horizontal += dist,
                SubCommand::Down(dist) => self.depth += dist,
                SubCommand::Up(dist) => self.depth -= dist,
            }
        }

        pub fn follow_commands<I, C>(&mut self, commands: I)
        where
            I: IntoIterator<Item = C>,
            C: Borrow<SubCommand>,
        {
            for command in commands.into_iter() {
                self.follow_command(command.borrow());
            }
        }

        pub fn get_pos(&self) -> (u32, u32) {
            (self.horizontal, self.depth)
        }
    }
}

pub mod p2 {
    use super::SubCommand;
    use std::borrow::Borrow;

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct Sub {
        horizontal: u32,
        depth: u32,
        aim: u32,
    }

    impl Sub {
        pub fn new() -> Self {
            Sub::default()
        }

        pub fn follow_command(&mut self, command: &SubCommand) {
            match command {
                SubCommand::Down(dist) => self.aim += dist,
                SubCommand::Up(dist) => self.aim -= dist,
                SubCommand::Forward(dist) => {
                    self.horizontal += dist;
                    self.depth += dist * self.aim;
                }
            }
        }

        pub fn follow_commands<I, C>(&mut self, commands: I)
        where
            I: IntoIterator<Item = C>,
            C: Borrow<SubCommand>,
        {
            for command in commands.into_iter() {
                self.follow_command(command.borrow());
            }
        }

        pub fn get_pos(&self) -> (u32, u32) {
            (self.horizontal, self.depth)
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SubCommand {
    Forward(u32),
    Down(u32),
    Up(u32),
}

impl FromStr for SubCommand {
    type Err = SubCommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, distance_str) = s.split_once(" ").ok_or(Self::Err::NoSpace)?;
        let distance = distance_str
            .parse()
            .map_err(|e| Self::Err::InvalidDistance(e))?;
        match name {
            "forward" => Ok(Self::Forward(distance)),
            "down" => Ok(Self::Down(distance)),
            "up" => Ok(Self::Up(distance)),
            _ => Err(Self::Err::UnknownName),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SubCommandParseError {
    NoSpace,
    UnknownName,
    InvalidDistance(ParseIntError),
}

impl Error for SubCommandParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidDistance(int_err) => Some(int_err),
            _ => None,
        }
    }
}

impl Display for SubCommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSpace => write!(f, "no space in SubCommand string"),
            Self::UnknownName => write!(f, "unrecognized SubCommand name"),
            Self::InvalidDistance(err) => write!(f, "invalid SubCommand distance: {}", err),
        }
    }
}

pub fn main() {
    let input: Vec<SubCommand> = include_str!("input.txt")
        .lines()
        .map(|t| t.parse().unwrap())
        .collect();

    let mut sub = p1::Sub::new();
    sub.follow_commands(&input);
    let (x, y) = sub.get_pos();
    println!("P1: x={}, y={}, x*y={}", x, y, x * y);

    let mut sub = p2::Sub::new();
    sub.follow_commands(&input);
    let (x, y) = sub.get_pos();
    println!("P1: x={}, y={}, x*y={}", x, y, x * y);
}
