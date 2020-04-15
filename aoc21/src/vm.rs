use std::result::Result::Err;

use crate::{Error, Result};
use crate::{Op, Opcode, OpcodeId, RegisterId, RegisterValue, Registers};
use std::collections::HashSet;
use std::str::FromStr;

pub enum Part {
    One,
    Two,
}

pub struct VM {
    registers: Registers,
    ops: Vec<Op>,
    ip_register: RegisterId,
    prev: RegisterValue,
}

impl VM {
    pub fn set_r0(self, r0: RegisterValue) -> Self {
        let registers = Registers([r0, 0, 0, 0, 0, 0]);
        Self {
            registers,
            ops: self.ops,
            ip_register: self.ip_register,
            prev: 0,
        }
    }

    // Steps through our program until it halts, returning the value at register 0.

    pub fn run(&mut self, part: Part) -> RegisterValue {
        let mut visited = HashSet::new();
        loop {
            match self.step(&mut visited, &part) {
                Ok(()) => continue,
                Err(r0_val) => return r0_val,
            }
        }
    }

    // Runs the Op at the current instruction pointer (IP), then increments the IP.
    //
    // If the IP is outside the range of our program, we halt, and return the invalid IP upon
    // halting.

    fn step(
        &mut self,
        visited: &mut HashSet<RegisterValue>,
        part: &Part,
    ) -> Result<(), RegisterValue> {
        // TODO: ideally, we can update our IP RegisterId's type to be a usize...
        let ip = self.registers.get(self.ip_register) as usize;

        let op = self
            .ops
            .get(ip)
            .expect("IP should not point outside the program instruction range.");
        let mut next_registers = op.exec(&self.registers);
        let next_ip = next_registers.get(self.ip_register) + 1;

        if self.registers.0[3] == 28 {
            let v = self.registers.0[1];
            match part {
                Part::One => {
                    if self.prev == 0 {
                        return Err(v);
                    }
                }
                Part::Two => {
                    if visited.contains(&v) {
                        return Err(self.prev);
                    } else {
                        visited.insert(v);
                        self.prev = v;
                    }
                }
            }
        }

        next_registers.set(self.ip_register, next_ip);

        self.registers = next_registers;
        if next_ip < (self.ops.len() as RegisterValue) {
            Ok(())
        } else {
            // Stop the program once the IP goes out of range, returning the value in R0:
            Err(self.registers.get(RegisterId::R0))
        }
    }
}

impl FromStr for VM {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let ip_register = match lines.next() {
            Some(line) => line.trim_start_matches("#ip ").parse::<RegisterId>()?,
            None => {
                return Err(Self::Err::from(format!(
                    "unable to parse first string from lines: {:?}",
                    lines
                )))
            }
        };
        let ops = lines
            .map(|line| line.parse::<Op>())
            .collect::<Result<Vec<Op>>>()?;
        let registers = Registers([0; 6]);
        Ok(Self {
            registers,
            ops,
            ip_register: ip_register,
            prev: 0,
        })
    }
}

// #[test]
// fn test_instruction_pointer() -> Result<()> {
//     let input = "\
//         #ip 0\n\
//         seti 5 0 1\n\
//         seti 6 0 2\n\
//         addi 0 1 0\n\
//         addr 1 2 3\n\
//         setr 1 0 0\n\
//         seti 8 0 4\n\
//         seti 9 0 5\n\
//     ";
//     let mut vm = input.parse::<VM>()?;
//
//     assert!(vm.step().is_ok());
//     assert_eq!(vm.registers, Registers([1, 5, 0, 0, 0, 0]));
//     assert!(vm.step().is_ok());
//     assert_eq!(vm.registers, Registers([2, 5, 6, 0, 0, 0]));
//     assert!(vm.step().is_ok());
//     assert_eq!(vm.registers, Registers([4, 5, 6, 0, 0, 0]));
//     assert!(vm.step().is_ok());
//     assert_eq!(vm.registers, Registers([6, 5, 6, 0, 0, 0]));
//
//     assert_eq!(vm.run(), 7);
//
//     println!("test_instruction_pointer passed.");
//
//     Ok(())
// }
