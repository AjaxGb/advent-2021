use std::fmt::{Display, Write};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BracketShape {
    Round,
    Square,
    Curly,
    Angle,
}

impl BracketShape {
    pub const fn open(&self) -> Bracket {
        Bracket(*self, BracketSide::Open)
    }

    pub const fn close(&self) -> Bracket {
        Bracket(*self, BracketSide::Close)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BracketSide {
    Open,
    Close,
}

impl BracketSide {
    pub const fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }

    pub const fn is_close(&self) -> bool {
        matches!(self, Self::Close)
    }

    pub const fn opposite(&self) -> Self {
        match self {
            Self::Open => Self::Close,
            Self::Close => Self::Open,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Bracket(BracketShape, BracketSide);

impl Bracket {
    pub const fn try_from_char(c: char) -> Option<Self> {
        Some(match c {
            '(' => Self(BracketShape::Round, BracketSide::Open),
            ')' => Self(BracketShape::Round, BracketSide::Close),
            '[' => Self(BracketShape::Square, BracketSide::Open),
            ']' => Self(BracketShape::Square, BracketSide::Close),
            '{' => Self(BracketShape::Curly, BracketSide::Open),
            '}' => Self(BracketShape::Curly, BracketSide::Close),
            '<' => Self(BracketShape::Angle, BracketSide::Open),
            '>' => Self(BracketShape::Angle, BracketSide::Close),
            _ => return None,
        })
    }

    pub const fn get_char(&self) -> char {
        match self {
            Self(BracketShape::Round, BracketSide::Open) => '(',
            Self(BracketShape::Round, BracketSide::Close) => ')',
            Self(BracketShape::Square, BracketSide::Open) => '[',
            Self(BracketShape::Square, BracketSide::Close) => ']',
            Self(BracketShape::Curly, BracketSide::Open) => '{',
            Self(BracketShape::Curly, BracketSide::Close) => '}',
            Self(BracketShape::Angle, BracketSide::Open) => '<',
            Self(BracketShape::Angle, BracketSide::Close) => '>',
        }
    }

    pub const fn is_open(&self) -> bool {
        self.1.is_open()
    }

    pub const fn is_close(&self) -> bool {
        self.1.is_close()
    }

    pub const fn opposite(&self) -> Self {
        Self(self.0, self.1.opposite())
    }
}

impl TryFrom<char> for Bracket {
    type Error = BracketCharErr;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Self::try_from_char(value).ok_or(BracketCharErr(value))
    }
}

impl Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(self.get_char())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BracketCharErr(pub char);

impl std::error::Error for BracketCharErr {}

impl Display for BracketCharErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bracket {:?}", self.0)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ParseBracketsErr {
    Unclosed(Vec<BracketShape>),
    IncorrectClose {
        expected: BracketShape,
        actual: BracketShape,
    },
    UnexpectedClose(BracketShape),
    InvalidBracket(char),
}

impl std::error::Error for ParseBracketsErr {}

impl Display for ParseBracketsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseBracketsErr::Unclosed(shapes) => Ok({
                write!(f, "code ended without closing the ")?;
                for shape in shapes {
                    write!(f, "{}", shape.open())?;
                }
            }),
            ParseBracketsErr::IncorrectClose { expected, actual } => {
                write!(
                    f,
                    "code closed the {} with a {}",
                    expected.open(),
                    actual.close(),
                )
            }
            ParseBracketsErr::UnexpectedClose(shape) => {
                write!(
                    f,
                    "code contained {} with no matching {}",
                    shape.close(),
                    shape.open(),
                )
            }
            ParseBracketsErr::InvalidBracket(bracket) => {
                write!(
                    f,
                    "code contained {:?}, which is not a valid bracket",
                    bracket,
                )
            }
        }
    }
}

impl From<BracketCharErr> for ParseBracketsErr {
    fn from(value: BracketCharErr) -> Self {
        let BracketCharErr(bracket) = value;
        Self::InvalidBracket(bracket)
    }
}

pub fn parse_brackets(text: &str) -> Result<(), ParseBracketsErr> {
    let mut open = vec![];
    for bracket in text.chars() {
        match bracket.try_into()? {
            Bracket(shape, BracketSide::Open) => open.push(shape),
            Bracket(shape, BracketSide::Close) => {
                if let Some(expected) = open.pop() {
                    if expected != shape {
                        return Err(ParseBracketsErr::IncorrectClose {
                            expected,
                            actual: shape,
                        });
                    }
                } else {
                    return Err(ParseBracketsErr::UnexpectedClose(shape));
                }
            }
        }
    }
    if open.is_empty() {
        Ok(())
    } else {
        Err(ParseBracketsErr::Unclosed(open))
    }
}

pub const fn get_syntax_error_value(shape: BracketShape) -> u64 {
    match shape {
        BracketShape::Round => 3,
        BracketShape::Square => 57,
        BracketShape::Curly => 1197,
        BracketShape::Angle => 25137,
    }
}

pub const fn get_autocomplete_value(shape: BracketShape) -> u64 {
    match shape {
        BracketShape::Round => 1,
        BracketShape::Square => 2,
        BracketShape::Curly => 3,
        BracketShape::Angle => 4,
    }
}

pub fn main() {
    let mut syntax_error_score = 0;
    let mut autocomplete_scores = vec![];

    for line in include_str!("input.txt").lines() {
        match parse_brackets(line).unwrap_err() {
            ParseBracketsErr::IncorrectClose { actual, .. } => {
                syntax_error_score += get_syntax_error_value(actual);
            }
            ParseBracketsErr::Unclosed(unclosed) => {
                let mut autocomplete_score = 0;
                for shape in unclosed.into_iter().rev() {
                    autocomplete_score *= 5;
                    autocomplete_score += get_autocomplete_value(shape);
                }
                autocomplete_scores.push(autocomplete_score);
            }
            err => panic!("{}", err),
        }
    }

    autocomplete_scores.sort();
    let autocomplete_score = autocomplete_scores[autocomplete_scores.len() / 2];

    println!("P1: syntax error score = {}", syntax_error_score);
    println!("P2: autocomplete score = {}", autocomplete_score);
}
