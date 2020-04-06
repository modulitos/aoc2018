use std::collections::{HashMap, HashSet};
use std::convert::{From, TryFrom};
use std::io::{Read, Write};
use std::str::FromStr;

mod error;
mod op_codes;

use error::{Error, Result};
use op_codes::{Opcode, OpcodeName, Registers, RegisterValue};

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let cpu = input.parse::<CPU>()?;
    writeln!(
        std::io::stdout(),
        "total samples: {:?}",
        cpu.samples.0.len()
    )?;
    writeln!(
        std::io::stdout(),
        "samples of three or more: {:?}",
        cpu.samples.with_three_or_more_matches()?
    )?;
    writeln!(
        std::io::stdout(),
        "final registers state: {:?}",
        cpu.evaluate_instructions()?
    )?;
    Ok(())
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

pub type UnknownOpcodeId = u8;
type InstructionValue = u8;

// Represents an opcode that is unknown, but with known arguments..

pub struct Instruction {
    pub opcode_id: UnknownOpcodeId,
    a: InstructionValue,
    b: InstructionValue,
    c: InstructionValue,
}
impl Instruction {
    // Returns a vec of all opcodes for instruction.

    pub fn get_opcodes(&self) -> Result<Vec<Opcode>> {
        OpcodeName::iter()
            .map(|&id| Opcode::from_args(id, self.a, self.b, self.c))
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
    // Returns the names of the opcodes that match the sample's execution.

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
    fn with_three_or_more_matches(&self) -> Result<usize> {
        Ok(self
            .0
            .iter()
            .map(|sample| Ok(sample.opcode_matches()?.len()))
            .collect::<Result<Vec<usize>>>()?
            .iter()
            .filter(|len| len >= &&3)
            .count())
    }

    // Returns a mapping of the opcode numerical id's to the opcode's name

    fn get_mapping_from_samples(&self) -> Result<HashMap<UnknownOpcodeId, OpcodeName>> {
        type OpcodeAccumulator = HashMap<UnknownOpcodeId, HashSet<OpcodeName>>;

        let mut map_acc = self.0.iter().try_fold::<OpcodeAccumulator, fn(
            OpcodeAccumulator,
            &Sample,
        ) -> Result<OpcodeAccumulator>, Result<HashMap<UnknownOpcodeId, HashSet<OpcodeName>>>>(
            OpcodeAccumulator::new(),
            |mut map, sample| {
                // union the existing and new sets of potential matches together
                let set = map
                    .entry(sample.instruction.opcode_id)
                    .or_insert(HashSet::new());
                *set = set
                    .union(&sample.opcode_matches()?)
                    .cloned()
                    .collect::<HashSet<_>>();
                Ok(map)
            },
        )?;

        // Iterate over the map of accumulations, reducing each HashSet<OpcodeName> value until they
        // becaome a single OpcodeName

        // number of uniequ mappings that we've found:
        let mut prev_found_len = 0;

        loop {
            if map_acc.values().filter(|set| set.len() > 1).count() == 0 {
                // map the single values that are left to a new map:
                return Ok(map_acc
                    .into_iter()
                    .fold(HashMap::new(), |mut map, (id, set)| {
                        if let Some(name) = set.into_iter().last() {
                            map.insert(id, name);
                        }
                        map
                    }));
            } else {
                let found = map_acc
                    .values()
                    .filter(|set| set.len() == 1)
                    .flat_map(|set| set.clone())
                    .collect::<HashSet<OpcodeName>>();

                if found.len() == prev_found_len {
                    return Err(Error::from("Could not find unique mappings"));
                } else {
                    prev_found_len = found.len();

                    map_acc
                        .values_mut()
                        .filter(|set| set.len() > 1)
                        .for_each(|set| *set = set.difference(&found).cloned().collect());
                }
            }
        }
    }
}
struct CPU {
    samples: Samples,
    instructions: Vec<Instruction>,
}

impl CPU {
    fn evaluate_instructions(&self) -> Result<Registers> {
        let map = self.samples.get_mapping_from_samples()?;
        println!("map: {:?}", map);
        println!("starting register calc...");
        let mut registers = Registers([0; 4]);
        self.instructions
            .iter()
            .map(|instruction| {
                // instruction.get_opcode(map.get(&instruction.opcode_id).unwrap().clone())
                Opcode::from_args(
                    map.get(&instruction.opcode_id).unwrap().clone(),
                    instruction.a,
                    instruction.b,
                    instruction.c,
                )
            })
            .collect::<Result<Vec<Opcode>>>()?
            .into_iter()
            .for_each(|opcode| registers = opcode.exec(&registers));
        Ok(registers)
    }
}

impl FromStr for CPU {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: is there a way to iterate over this string in chunks of 4, splitting on newlines?
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

        let instructions = iter
            .filter(|(i, line)| line.len() != 0)
            .map(|(_, line)| line.parse::<Instruction>())
            .collect::<Result<Vec<Instruction>>>()?;

        Ok(Self {
            samples: Samples(samples),
            instructions,
        })
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
fn test_opcodes_matches() -> Result<()> {
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
    ";

    let cpu = input.parse::<CPU>()?;
    assert_eq!(cpu.samples.with_three_or_more_matches()?, 2);
    println!("test_opcode_matches passed.");
    Ok(())
}

#[test]
fn test_evaluate_instructions() -> Result<()> {
    let input = "\
        Before: [3, 2, 1, 9]\n\
        1 2 3 2\n\
        After:  [3, 2, 4, 9]\n\
        \n\
        Before: [3, 2, 1, 1]\n\
        2 0 3 1\n\
        After:  [3, 9, 1, 1]\n\
        \n\
        Before: [2, 2, 3, 3]\n\
        3 2 3 1\n\
        After:  [2, 9, 3, 3]\n\
        \n\
        Before: [3, 2, 1, 1]\n\
        4 2 1 2\n\
        After:  [3, 2, 2, 1]\n\
        \n\
        \n\
        1 2 2 3\n\
        1 3 5 1\n\
        1 3 5 0\n\
        1 2 1 2\n\
        2 1 3 1\n\
        2 1 1 0\n\
    ";
    // registers state:
    // 0 0 0 0
    // 0 0 0 2
    // 0 7 0 2
    // 7 7 0 2
    // 7 7 1 2
    // 7 21 1 2
    // 21 21 1 2

    let cpu = input.parse::<CPU>()?;
    let mut map = HashMap::new();
    use OpcodeName::*;
    map.insert(1, Addi);
    map.insert(2, Muli);
    map.insert(3, Mulr);
    map.insert(4, Seti); // This instruction could have been mulr, addi, or seti
    assert_eq!(cpu.samples.get_mapping_from_samples()?, map);
    assert_eq!(cpu.evaluate_instructions()?, Registers([21, 21, 1, 2]));
    println!("test_evaluate_instructions passed.");
    Ok(())
}
