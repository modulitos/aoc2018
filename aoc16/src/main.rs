use std::boxed::Box;
use std::collections::HashSet;
use std::convert::{From, TryFrom};
use std::error;
use std::io::{Read, Write};
use std::result;
use std::slice::Iter;
use std::str::FromStr;

type Error = Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let samples = input.parse::<Samples>()?;
    writeln!(std::io::stdout(), "total samples: {:?}", samples.0.len())?;
    writeln!(std::io::stdout(), "samples of three or more: {:?}", samples.samples_three_or_more()?)?;
    Ok(())
}

#[derive(Copy, Clone)]
// This is basically a ranged type.
enum RegisterId {
    R0,
    R1,
    R2,
    R3,
}

impl RegisterId {
    fn from_number(n: InstructionValue) -> Result<Self> {
        match n {
            0 => Ok(RegisterId::R0),
            1 => Ok(RegisterId::R1),
            2 => Ok(RegisterId::R2),
            3 => Ok(RegisterId::R3),
            _ => Err(Error::from(format!("must be within [0-3]: {}", n))),
        }
    }
}

type RegisterValue = u8;

#[derive(Clone, Eq, PartialEq)]
struct Registers([RegisterValue; 4]);

impl Registers {
    fn get(&self, id: RegisterId) -> RegisterValue {
        match id {
            RegisterId::R0 => self.0[0],
            RegisterId::R1 => self.0[1],
            RegisterId::R2 => self.0[2],
            RegisterId::R3 => self.0[3],
            // _ => panic!("register id must be within [0-4): {}", id),
        }
    }
    fn set(&mut self, id: RegisterId, value: RegisterValue) {
        let register = match id {
            RegisterId::R0 => &mut self.0[0],
            RegisterId::R1 => &mut self.0[1],
            RegisterId::R2 => &mut self.0[2],
            RegisterId::R3 => &mut self.0[3],
            // _ => panic!("register id must be within [0-4): {}", id),
        };
        *register = value;
    }
}

impl FromStr for Registers {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals = s
            .split(", ")
            .map(|digit| digit.parse::<RegisterValue>())
            .collect::<Result<Vec<RegisterValue>, _>>()?;
        if vals.len() != 4 {
            return Err(Self::Err::from(format!(
                "cannot parse Registers from string: {:?}",
                s
            )));
        }
        Ok(Registers([vals[0], vals[1], vals[2], vals[3]]))
    }
}

type InstructionValue = u8;

// type OpcodeId = u8;
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum OpcodeName {
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

struct Opcode {
    id: OpcodeName,
    kind: Op,
    c: RegisterId, // the register that will take the output of the opcode
}

impl Opcode {
    fn exec(&self, registers: &Registers) -> Registers {
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

type OpcodeId = u8;

struct Instruction {
    opcode_id: OpcodeId,
    a: InstructionValue,
    b: InstructionValue,
    c: InstructionValue,
}
impl Instruction {
    fn get_opcodes(&self) -> Result<Vec<Opcode>> {
        let a = self.a;
        let b = self.b;
        let c = self.c;

        use Op::*;
        OpcodeName::iter()
            .map(|&id| {
                let mkid = RegisterId::from_number;
                let mkval = RegisterValue::try_from;

                let kind = match id {
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
                    id,
                    kind,
                    c: mkid(c)?,
                })
            })
            .collect::<Result<Vec<Opcode>>>()
    }
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals = s
            .split(' ')
            .map(|c| c.parse::<InstructionValue>())
            .collect::<Result<Vec<InstructionValue>, _>>()?;
        if vals.len() != 4 {
            return Err(Error::from(format!(
                "InstructionValues must have length within [0-3): {:?}",
                vals
            )));
        }
        Ok(Self {
            opcode_id: vals[0],
            a: vals[1],
            b: vals[2],
            c: vals[3],
        })
    }
}

struct Sample {
    start: Registers,
    end: Registers,
    instruction: Instruction,
}

impl Sample {
    fn opcode_matches(&self) -> Result<HashSet<OpcodeName>> {
        Ok(self
            .instruction
            .get_opcodes()?
            .into_iter()
            .filter(|opcode| opcode.exec(&self.start) == self.end)
            .map(|opcode| opcode.id)
            .collect())
    }
}

impl FromStr for Sample {
    type Err = Error;

    // assumes 3 lines of input per sample

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<&str>>();
        if lines.len() != 3 {
            return Err(Error::from(format!(
                "cannot parse Sample from input: {:?}",
                lines
            )));
        }
        let start = lines[0]
            .trim_start_matches("Before: [")
            .trim_end_matches(']')
            .parse::<Registers>()?;
        let instruction = lines[1].parse::<Instruction>()?;
        let end = lines[2]
            .trim_start_matches("After:  [")
            .trim_end_matches(']')
            .parse::<Registers>()?;

        Ok(Self {
            start,
            end,
            instruction,
        })
    }
}

struct Samples(Vec<Sample>);

impl Samples {
    fn samples_three_or_more(&self) -> Result<usize> {
        Ok(self.0
            .iter()
            .map(|sample| {
                Ok(sample.opcode_matches()?.len())
            })
            .collect::<Result<Vec<usize>>>()?
            .iter()
            .filter(|len| len >= &&3)
            .count())
    }
}
impl FromStr for Samples {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        // TODO: is there a way to iterator over this string in chunks of 4, splitting on newlines?
        // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=386698a9afe74ec4a16b4189b487959f

        let mut iter = s.lines().enumerate();
        let mut samples: Vec<Sample> = vec![];
        let mut chunk: Vec<&str> = vec![];
        while let Some((i, line)) = iter.next() {
            if i % 4 == 0 && line.len() == 0 {
                // we're done - we've hit a section with double blank new lines
                break;
            }

            if i % 4 == 3 {
                samples.push(chunk.join("\n").parse()?);
                chunk = vec![];
            } else {
                chunk.push(line);
            }
        }
        Ok(Samples(samples))
    }
}

#[test]
fn test_opcode() -> Result<()> {
    let input = "\
        Before: [3, 2, 1, 1]\n\
        9 2 1 2\n\
        After:  [3, 2, 2, 1]\n\
    ";

    let sample = input.parse::<Sample>()?;
    use OpcodeName::*;
    assert_eq!(
        sample.opcode_matches()?,
        vec![Mulr, Addi, Seti].into_iter().collect()
    );
    println!("test_opcode passed.");
    Ok(())
}

#[test]
fn test_multi_opcodes() -> Result<()> {
    let input = "\
        Before: [3, 2, 1, 1]\n\
        9 2 1 2\n\
        After:  [3, 2, 2, 1]\n\
        \n\
        Before: [3, 2, 1, 1]\n\
        9 2 1 2\n\
        After:  [3, 2, 2, 1]\n\
        \n\
        Before: [3, 2, 100, 1]\n\
        9 2 1 2\n\
        After:  [3, 2, 2, 1]\n\
        \n\
        \n\
        This should not be parsed
    ";

    let samples = input.parse::<Samples>()?;
    assert_eq!(samples.samples_three_or_more()?, 2);
    println!("test_multi_opcode passed.");
    Ok(())
}


