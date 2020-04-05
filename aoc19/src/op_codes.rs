use crate::{Error, Result};
use std::convert::{From, TryFrom};
use std::slice::Iter;

// This module contains all logic pertaining to our registers and opcodes.

#[derive(Copy, Clone, Debug)]
// This is just a ranged type.
enum RegisterId {
    R0,
    R1,
    R2,
    R3,
}

impl RegisterId {
    fn from_number(n: u8) -> Result<Self> {
        match n {
            0 => Ok(RegisterId::R0),
            1 => Ok(RegisterId::R1),
            2 => Ok(RegisterId::R2),
            3 => Ok(RegisterId::R3),
            _ => Err(Error::from(format!("must be within [0-3]: {}", n))),
        }
    }
}

type RegisterValue = u32;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Registers(pub [RegisterValue; 4]);

impl Registers {
    fn get(&self, id: RegisterId) -> RegisterValue {
        match id {
            RegisterId::R0 => self.0[0],
            RegisterId::R1 => self.0[1],
            RegisterId::R2 => self.0[2],
            RegisterId::R3 => self.0[3],
        }
    }
    fn set(&mut self, id: RegisterId, value: RegisterValue) {
        let register = match id {
            RegisterId::R0 => &mut self.0[0],
            RegisterId::R1 => &mut self.0[1],
            RegisterId::R2 => &mut self.0[2],
            RegisterId::R3 => &mut self.0[3],
        };
        *register = value;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum OpcodeName {
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

impl OpcodeName {
    pub fn iter() -> Iter<'static, OpcodeName> {
        use OpcodeName::*;
        static IDS: [OpcodeName; 16] = [
            Addr, Addi, Mulr, Muli, Banr, Bani, Borr, Bori, Setr, Seti, Gtir, Gtri, Gtrr, Eqir,
            Eqri, Eqrr,
        ];
        IDS.iter()
    }
}

pub struct Opcode {
    pub id: OpcodeName,
    kind: Op,
    c: RegisterId, // the register that will take the output of the opcode
}

impl Opcode {
    // Get the opcode corresponding to the provided OpcodeName, using the values from the
    // instruction set

    pub fn from_args(name: OpcodeName, a: u8, b: u8, c: u8) -> Result<Self> {
        let a = a;
        let b = b;
        let c = c;

        use Op::*;
        let mkid = RegisterId::from_number;
        let mkval = RegisterValue::try_from;

        let kind = match name {
            OpcodeName::Addr => Addr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeName::Addi => Addi {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeName::Mulr => Mulr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeName::Muli => Muli {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeName::Banr => Banr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeName::Bani => Bani {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeName::Borr => Borr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeName::Bori => Bori {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeName::Setr => Setr { a: mkid(a)? },
            OpcodeName::Seti => Seti { a: mkval(a)? },
            OpcodeName::Gtir => Gtir {
                a: mkval(a)?,
                b: mkid(b)?,
            },
            OpcodeName::Gtri => Gtri {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeName::Gtrr => Gtrr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
            OpcodeName::Eqir => Eqir {
                a: mkval(a)?,
                b: mkid(b)?,
            },
            OpcodeName::Eqri => Eqri {
                a: mkid(a)?,
                b: mkval(b)?,
            },
            OpcodeName::Eqrr => Eqrr {
                a: mkid(a)?,
                b: mkid(b)?,
            },
        };
        Ok(Opcode {
            id: name,
            kind,
            c: mkid(c)?,
        })
    }

    pub fn exec(&self, registers: &Registers) -> Registers {
        let mut result = registers.clone();
        use Op::*;
        let new_val = match &self.kind {
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
        result.set(self.c, new_val);
        result
    }
}

enum Op {
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

