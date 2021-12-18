use bitvec::prelude::*;
use std::error::Error;
use std::fmt::Display;

use super::Packet;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ParseHexErr(pub char);

impl Display for ParseHexErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} is not a valid hexadecimal character", self.0)
    }
}

impl Error for ParseHexErr {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ReadBitsErr {
    NotEnoughData,
}

impl Display for ReadBitsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not enough data to supply read")
    }
}

impl Error for ReadBitsErr {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ReadPacketErr {
    ReadError(ReadBitsErr),
    NoOperands,
    WrongNumberCompOperands(Vec<Packet>),
}

impl Display for ReadPacketErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(err) => write!(f, "bit read failed: {}", err),
            Self::NoOperands => write!(f, "operator had no operands"),
            Self::WrongNumberCompOperands(ops) => {
                write!(f, "comparison operation had {} child packets", ops.len())
            }
        }
    }
}

impl Error for ReadPacketErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ReadBitsErr> for ReadPacketErr {
    fn from(err: ReadBitsErr) -> Self {
        Self::ReadError(err)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ParseHexPacketErr {
    HexError(ParseHexErr),
    ReadError(ReadPacketErr),
    TrailingData(Packet, BitVec),
}

impl Display for ParseHexPacketErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HexError(err) => write!(f, "hex parse failed: {}", err),
            Self::ReadError(err) => err.fmt(f),
            Self::TrailingData(_, data) => write!(f, "trailing data after packet: {}", data),
        }
    }
}

impl Error for ParseHexPacketErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::HexError(err) => Some(err),
            Self::ReadError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ParseHexErr> for ParseHexPacketErr {
    fn from(err: ParseHexErr) -> Self {
        Self::HexError(err)
    }
}

impl From<ReadPacketErr> for ParseHexPacketErr {
    fn from(err: ReadPacketErr) -> Self {
        Self::ReadError(err)
    }
}
