use crate::{Error, Result};
use std::convert::{From, TryFrom};
use std::slice::Iter;
use std::str::FromStr;

// This module contains the data structures pertaining to our registers and opcodes.

#[derive(Copy, Clone, Debug)]
// This is just a ranged type.
// TODO: is there a better way to implement a ranged type?
pub enum RegisterId {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
}

type UnknownInstructionValue = u64;

impl RegisterId {
    fn from_number(n: UnknownInstructionValue) -> Result<Self> {
        match n {
            0 => Ok(RegisterId::R0),
            1 => Ok(RegisterId::R1),
            2 => Ok(RegisterId::R2),
            3 => Ok(RegisterId::R3),
            4 => Ok(RegisterId::R4),
            5 => Ok(RegisterId::R5),
            _ => Err(Error::from(format!("must be within [0-3]: {}", n))),
        }
    }
}

impl FromStr for RegisterId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.parse::<UnknownInstructionValue>()? {
            0 => RegisterId::R0,
            1 => RegisterId::R1,
            2 => RegisterId::R2,
            3 => RegisterId::R3,
            4 => RegisterId::R4,
            5 => RegisterId::R5,
            _ => {
                return Err(Self::Err::from(format!(
                    "cannot parse RegisterId from string: {}",
                    s
                )))
            }
        })
    }
}

pub type RegisterValue = u64;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Registers(pub [RegisterValue; 6]);

impl Registers {
    pub fn get(&self, id: RegisterId) -> RegisterValue {
        match id {
            RegisterId::R0 => self.0[0],
            RegisterId::R1 => self.0[1],
            RegisterId::R2 => self.0[2],
            RegisterId::R3 => self.0[3],
            RegisterId::R4 => self.0[4],
            RegisterId::R5 => self.0[5],
        }
    }
    pub fn set(&mut self, id: RegisterId, value: RegisterValue) {
        let register = match id {
            RegisterId::R0 => &mut self.0[0],
            RegisterId::R1 => &mut self.0[1],
            RegisterId::R2 => &mut self.0[2],
            RegisterId::R3 => &mut self.0[3],
            RegisterId::R4 => &mut self.0[4],
            RegisterId::R5 => &mut self.0[5],
        };
        *register = value;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum OpcodeId {
    Addr,
    Addi,
    Mulr,
    Muli,
    Banr,
    Bani,
    Borr,
    Bori,
    Setr,
    Seti,
    Gtir,
    Gtri,
    Gtrr,
    Eqir,
    Eqri,
    Eqrr,
}

impl OpcodeId {
    pub fn iter() -> Iter<'static, OpcodeId> {
        use OpcodeId::*;
        static IDS: [OpcodeId; 16] = [
            Addr, Addi, Mulr, Muli, Banr, Bani, Borr, Bori, Setr, Seti, Gtir, Gtri, Gtrr, Eqir,
            Eqri, Eqrr,
        ];
        IDS.iter()
    }
}

impl FromStr for OpcodeId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use OpcodeId::*;
        let name = match s {
            "addr" => Addr,
            "addi" => Addi,
            "mulr" => Mulr,
            "muli" => Muli,
            "banr" => Banr,
            "bani" => Bani,
            "borr" => Borr,
            "bori" => Bori,
            "setr" => Setr,
            "seti" => Seti,
            "gtir" => Gtir,
            "gtri" => Gtri,
            "gtrr" => Gtrr,
            "eqir" => Eqir,
            "eqri" => Eqri,
            "eqrr" => Eqrr,
            _ => return Err(Self::Err::from(format!("could not convert from {}", s))),
        };
        Ok(name)
    }
}

#[derive(Debug)]
pub struct Op {
    opcode: Opcode,
    output: RegisterId, // the register that will take the output of the opcode
}

impl Op {
    // Get the opcode corresponding to the provided OpcodeName, using the values from the
    // instruction set

    pub fn from_args(
        name: OpcodeId,
        a: UnknownInstructionValue,
        b: UnknownInstructionValue,
        c: UnknownInstructionValue,
    ) -> Result<Self> {
        let a = a;
        let b = b;
        let c = c;

        use Opcode::*;
        let mkid = RegisterId::from_number;
        let mkval = RegisterValue::try_from;

        let kind = match name {
            OpcodeId::Addr => Addr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeId::Addi => Addi {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeId::Mulr => Mulr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeId::Muli => Muli {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeId::Banr => Banr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeId::Bani => Bani {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeId::Borr => Borr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeId::Bori => Bori {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeId::Setr => Setr { a: mkid(a)? },
            OpcodeId::Seti => Seti { a: mkval(a)? },
            OpcodeId::Gtir => Gtir {
                a: mkval(a)?,
                b: mkid(b)?,
            },
            OpcodeId::Gtri => Gtri {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeId::Gtrr => Gtrr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeId::Eqir => Eqir {
                a: mkval(a)?,
                b: mkid(b)?,
            },
            OpcodeId::Eqri => Eqri {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeId::Eqrr => Eqrr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
        };
        Ok(Op {
            opcode: kind,
            output: mkid(c)?,
        })
    }

    // Given the state of registers, execute the Op (aka instruction) against those registers.
    // Return the new value of the registers.

    pub fn exec(&self, registers: &Registers) -> Registers {
        let mut result = registers.clone();
        use Opcode::*;
        let new_val = match &self.opcode {
            &Addr { a, b } => result.get(a) + result.get(b),
            &Addi { a, b } => result.get(a) + b,
            &Mulr { a, b } => result.get(a) * result.get(b),
            &Muli { a, b } => result.get(a) * b,
            &Banr { a, b } => result.get(a) & result.get(b),
            &Bani { a, b } => result.get(a) & b,
            &Borr { a, b } => result.get(a) | result.get(b),
            &Bori { a, b } => result.get(a) | b,
            &Setr { a } => result.get(a),
            &Seti { a } => a,
            &Gtir { a, b } => {
                if a > result.get(b) {
                    1
                } else {
                    0
                }
            }
            &Gtri { a, b } => {
                if result.get(a) > b {
                    1
                } else {
                    0
                }
            }
            &Gtrr { a, b } => {
                if result.get(a) > result.get(b) {
                    1
                } else {
                    0
                }
            }
            &Eqir { a, b } => {
                if a == result.get(b) {
                    1
                } else {
                    0
                }
            }
            &Eqri { a, b } => {
                if result.get(a) == b {
                    1
                } else {
                    0
                }
            }
            &Eqrr { a, b } => {
                if result.get(a) == result.get(b) {
                    1
                } else {
                    0
                }
            }
        };
        result.set(self.output, new_val);
        result
    }
}

impl FromStr for Op {
    type Err = Error;

    // eg: "seti 5 0 1"

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let matches = s.split(" ").collect::<Vec<&str>>();
        let (id, v0, v1, v2) = match matches.as_slice() {
            [name, v0, v1, v2] => (
                name.parse::<OpcodeId>()?,
                v0.parse::<UnknownInstructionValue>()?,
                v1.parse::<UnknownInstructionValue>()?,
                v2.parse::<UnknownInstructionValue>()?,
            ),
            _ => {
                return Err(Self::Err::from(format!(
                    "cannot parse the vec into an opcode: {:?}",
                    matches
                )))
            }
        };
        Op::from_args(id, v0, v1, v2)
    }
}

#[derive(Debug)]
pub enum Opcode {
    Addr { a: RegisterId, b: RegisterId },
    Addi { a: RegisterId, b: RegisterValue },
    Mulr { a: RegisterId, b: RegisterId },
    Muli { a: RegisterId, b: RegisterValue },
    Banr { a: RegisterId, b: RegisterId },
    Bani { a: RegisterId, b: RegisterValue },
    Borr { a: RegisterId, b: RegisterId },
    Bori { a: RegisterId, b: RegisterValue },
    Setr { a: RegisterId },
    Seti { a: RegisterValue },
    Gtir { a: RegisterValue, b: RegisterId },
    Gtri { a: RegisterId, b: RegisterValue },
    Gtrr { a: RegisterId, b: RegisterId },
    Eqir { a: RegisterValue, b: RegisterId },
    Eqri { a: RegisterId, b: RegisterValue },
    Eqrr { a: RegisterId, b: RegisterId },
}
