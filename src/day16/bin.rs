use std::{str::FromStr, fmt::Display};

use bitvec::{prelude::*, view::BitView};
use itertools::Itertools;

mod err;
use err::*;

pub fn parse_hex(hex: &str) -> Result<BitVec, ParseHexErr> {
    let mut data = BitVec::with_capacity(hex.len() * 4);
    for c in hex.chars() {
        let nibble = c.to_digit(16).ok_or(ParseHexErr(c))? as u8;
        data.extend_from_bitslice(&nibble.view_bits::<Msb0>()[4..]);
    }
    Ok(data)
}

pub fn read_bits<V, T>(source: &mut &BitSlice<Lsb0, T>, num_bits: usize) -> Result<V, ReadBitsErr>
where
    V: BitView + Default,
    T: BitStore,
{
    if num_bits > source.len() {
        return Err(ReadBitsErr::NotEnoughData);
    }

    let mut value = V::default();
    let dest = value.view_bits_mut::<Msb0>();
    let dest_range_start = dest
        .len()
        .checked_sub(num_bits)
        .expect("requested more bits than would fit");
    let dest = &mut dest[dest_range_start..];

    let (bits, remainder) = source.split_at(num_bits);
    *source = remainder;

    dest.clone_from_bitslice(bits);
    Ok(value)
}

pub fn read_bool<T>(source: &mut &BitSlice<Lsb0, T>) -> Result<bool, ReadBitsErr>
where
    T: BitStore,
{
    if source.is_empty() {
        return Err(ReadBitsErr::NotEnoughData);
    }
    let (bit, remainder) = source.split_first().unwrap();
    *source = remainder;
    Ok(*bit)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum PacketPayload {
    Literal(u64),
    Operation(Operation, Vec<Packet>),
    Comparison(Comparison, Box<[Packet; 2]>),
}

impl PacketPayload {
    pub fn sub_packets(&self) -> &[Packet] {
        match self {
            Self::Literal(_) => &[],
            Self::Operation(_, ops) => ops.as_slice(),
            Self::Comparison(_, ops) => &ops[..],
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Operation {
    Sum,
    Product,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Comparison {
    GreaterThan,
    LessThan,
    EqualTo,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Packet {
    version: u8,
    payload: PacketPayload,
}

impl Packet {
    pub fn read<T>(bits: &mut &BitSlice<Lsb0, T>) -> Result<Self, ReadPacketErr>
    where
        T: BitStore,
    {
        let version = read_bits(bits, 3)?;
        let kind: u8 = read_bits(bits, 3)?;
        let payload = match kind {
            0 => PacketPayload::Operation(Operation::Sum, Self::read_op_payload(bits)?),
            1 => PacketPayload::Operation(Operation::Product, Self::read_op_payload(bits)?),
            2 => PacketPayload::Operation(Operation::Min, Self::read_op_payload(bits)?),
            3 => PacketPayload::Operation(Operation::Max, Self::read_op_payload(bits)?),
            4 => {
                let mut value = 0;
                loop {
                    let chunk: u64 = read_bits(bits, 5)?;
                    let contents = chunk & 0b1111;
                    value = (value << 4) | contents;
                    if chunk == contents {
                        break;
                    }
                }
                PacketPayload::Literal(value)
            }
            5 => PacketPayload::Comparison(Comparison::GreaterThan, Self::read_comp_payload(bits)?),
            6 => PacketPayload::Comparison(Comparison::LessThan, Self::read_comp_payload(bits)?),
            7 => PacketPayload::Comparison(Comparison::EqualTo, Self::read_comp_payload(bits)?),
            _ => unreachable!(),
        };
        Ok(Self { version, payload })
    }

    fn read_op_payload<T>(bits: &mut &BitSlice<Lsb0, T>) -> Result<Vec<Self>, ReadPacketErr>
    where
        T: BitStore,
    {
        let mut operands = vec![];
        if read_bool(bits)? {
            let num_packets = read_bits(bits, 11)?;
            operands.reserve_exact(num_packets);
            for _ in 0..num_packets {
                operands.push(Packet::read(bits)?);
            }
        } else {
            let num_bits: usize = read_bits(bits, 15)?;
            let len_when_done = bits.len() - num_bits;
            while bits.len() > len_when_done {
                operands.push(Packet::read(bits)?);
            }
        }
        if operands.is_empty() {
            Err(ReadPacketErr::NoOperands)
        } else {
            Ok(operands)
        }
    }

    fn read_comp_payload<T>(bits: &mut &BitSlice<Lsb0, T>) -> Result<Box<[Self; 2]>, ReadPacketErr>
    where
        T: BitStore,
    {
        let operands = Self::read_op_payload(bits)?;
        if operands.len() != 2 {
            Err(ReadPacketErr::WrongNumberCompOperands(operands))
        } else {
            let (a, b) = operands.into_iter().collect_tuple().unwrap();
            Ok(Box::new([a, b]))
        }
    }

    pub fn version_sum(&self) -> u32 {
        let mut sum = self.version as u32;
        for operand in self.payload.sub_packets() {
            sum += operand.version_sum();
        }
        sum
    }

    pub fn evaluate(&self) -> u64 {
        match &self.payload {
            PacketPayload::Literal(val) => *val,
            PacketPayload::Operation(op, operands) => {
                let operands = operands.iter().map(|p| p.evaluate());
                match op {
                    Operation::Sum => operands.sum(),
                    Operation::Product => operands.product(),
                    Operation::Min => operands.min().unwrap(),
                    Operation::Max => operands.max().unwrap(),
                }
            }
            PacketPayload::Comparison(op, operands) => {
                let [a, b] = operands.as_ref();
                let a = a.evaluate();
                let b = b.evaluate();
                (match op {
                    Comparison::GreaterThan => a > b,
                    Comparison::LessThan => a < b,
                    Comparison::EqualTo => a == b,
                }) as u64
            }
        }
    }
	
	fn fmt_part(&self, f: &mut std::fmt::Formatter<'_>, enclosed: bool) -> std::fmt::Result {
        Ok(match &self.payload {
            PacketPayload::Literal(val) => write!(f, "{}", val)?,
            PacketPayload::Operation(op, operands) => {
				let (mid, is_func) = match op {
					Operation::Sum => (" + ", false),
					Operation::Product => (" * ", false),
					Operation::Min => {
						write!(f, "min(")?;
						(", ", true)
					},
					Operation::Max => {
						write!(f, "max(")?;
						(", ", true)
					},
				};
				if !is_func && operands.len() == 1 {
					return operands[0].fmt_part(f, enclosed);
				}
				if !is_func && !enclosed {
					write!(f, "(")?;
				}
				let mut operands = operands.iter();
				operands.next().unwrap().fmt_part(f, false)?;
				for operand in operands {
					write!(f, "{}", mid)?;
					operand.fmt_part(f, is_func)?;
				}
				if is_func || !enclosed {
					write!(f, ")")?;
				}
			},
            PacketPayload::Comparison(op, operands) => {
				let [a, b] = operands.as_ref();
				if !enclosed {
					write!(f, "(")?;
				}
				a.fmt_part(f, false)?;
				match op {
					Comparison::GreaterThan => write!(f, " > ")?,
					Comparison::LessThan => write!(f, " < ")?,
					Comparison::EqualTo => write!(f, " = ")?,
				}
				b.fmt_part(f, false)?;
				if !enclosed {
					write!(f, ")")?;
				}
			},
        })
	}
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_part(f, true)
    }
}

impl FromStr for Packet {
    type Err = ParseHexPacketErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bits = parse_hex(s)?;
        let mut bits = bits.as_bitslice();
        let packet = Packet::read(&mut bits)?;
        if bits.len() >= 8 || bits.count_ones() != 0 {
            Err(Self::Err::TrailingData(packet, bits.to_bitvec()))
        } else {
            Ok(packet)
        }
    }
}

pub fn main() {
    for line in include_str!("input.txt").lines() {
        let packet_tree: Packet = line.parse().unwrap();
		println!("{}", packet_tree);
        println!("P1: version sum is {}", packet_tree.version_sum());
        println!("P2: evaluation is {}", packet_tree.evaluate());
    }
}

#[cfg(test)]
mod tests {
    use crate::Packet;

    fn test_value(hex: &str, value: u64) {
        let packet: Packet = hex.parse().unwrap();
		let eval_value = packet.evaluate();
        assert_eq!(eval_value, value);
    }

    #[test]
    fn sum() {
        test_value("C200B40A82", 3);
    }

    #[test]
    fn product() {
        test_value("04005AC33890", 54);
    }

    #[test]
    fn min() {
        test_value("880086C3E88112", 7);
    }

    #[test]
    fn max() {
        test_value("CE00C43D881120", 9);
    }

    #[test]
    fn less_than() {
        test_value("D8005AC2A8F0", 1);
    }

    #[test]
    fn greater_than() {
        test_value("F600BC2D8F", 0);
    }

    #[test]
    fn equal() {
        test_value("9C005AC2F8F0", 0);
    }

    #[test]
    fn expressions() {
        test_value("9C0141080250320F1802104A08", 1);
    }
}
