mod error;
mod op_codes;
use std::result::Result::Err;

use error::{Error, Result};
use op_codes::{Op, Opcode, OpcodeId, RegisterId, Registers};
use std::fs::{canonicalize, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let mut cpu = input.parse::<CPU>()?;

    writeln!(
        std::io::stdout(),
        "value of Register 0 when halted: {}",
        cpu.run()
    )?;

    let mut cpu_2 = input.parse::<CPU>()?;

    cpu_2.registers.set(RegisterId::R0, 1);
    writeln!(
        std::io::stdout(),
        "when started with a value of 1 in R0, the value of Register 0 when halted is: {}",
        cpu_2.run()
    )?;

    Ok(())
}

struct CPU {
    registers: Registers,
    ops: Vec<Op>,
    ip_register: RegisterId,
}

impl CPU {

    // Steps through our program until it halts, returning the value at register 0.

    fn run(&mut self) -> u32 {
        loop {
            match self.step() {
                Ok(()) => continue,
                Err(r0_val) => return r0_val,
            }
        }
    }

    // Runs the Op at the current instruction pointer (IP), then increments the IP.
    //
    // If the IP is outside the range of our program, we halt, and return the invalid IP upon
    // halting.

    fn step(&mut self) -> Result<(), u32> {
        // TODO: ideally, we can update our IP RegisterId's type to be a usize...
        let ip = self.registers.get(self.ip_register) as usize;

        if ip == 3 {
            // Optimization for solving part 2 - when IP=3, skip the opcodes and execute an optimized
            // form instead.

            self.step_fast();
            return Ok(());
        }

        let op = self
            .ops
            .get(ip)
            .expect("IP should not point outside the program instruction range.");
        let mut next_registers = op.exec(&self.registers);
        let next_ip = next_registers.get(self.ip_register) + 1;

        next_registers.set(self.ip_register, next_ip);

        self.registers = next_registers;
        if next_ip < (self.ops.len() as u32) {
            Ok(())
        } else {
            // Stop the program once the IP goes out of range, returning the value in R0:
            Err(self.registers.get(RegisterId::R0))
        }
    }

    // An optimized implementation of the machine code at instruction pointer #3. This was specific
    // to my input, so every input will vary. For your input, you'll need to examine your assembly code for
    // sequences of opcodes that are looping excessively, and find a way to optimize it.

    fn step_fast(&mut self) {

        // The goal of the IP=3 loop is to find the value of R4 / R1, assuming that the value of R3
        // isn't already higher than the factor.
        //
        // Once found, increment R0 by the value of the factor, and set the value of R3 to be (R4 + 1), and set R5 to 1.

        // Here is the logic in the machine code, which is doing this calculation extremely
        // inefficiently:
        /*
        R5 = R1 * R3
        # when R2  pi=4
        if R4 == R5:
          R5 = 1
          R2 += 1 (R5)
          # goto IP=7
        else:
          R5 = 0
          # continue to IP=6

        # IP=6
        R2 += 1

        # IP=8
        R3 += 1

        # IP=9
        if R3 > R4:
          R5 = 1
          R2 += 1 (R5)
          # goto IP=12
        else:
          R5 = 0
          # continue to IP=10
          # IP=10
          R2 += R5
          # goto ip=3
        */

        // which can also be translated to this:
        /*
        loop {
            if r3 * r1 == r4 {
                let r0 = self.registers.get(RegisterId::R0);
                self.registers.set(RegisterId::R0, r0 + r1);
            }

            r3 += 1;

            if r3 > r4 {
                new_r3 = r3;
                break;
            }
        }
        */

        // and the loop above can be further optimized like so:

        let r4 = self.registers.get(RegisterId::R4);
        let r1 = self.registers.get(RegisterId::R1);
        let r3 = self.registers.get(RegisterId::R3);

        if r4 % r1 == 0 && r3 <= (r4 / r1) {
            // if a factor is possible, and we haven't gone passed it:
            let r0 = self.registers.get(RegisterId::R0);
            self.registers.set(RegisterId::R0, r0 + r1);
        }

        self.registers.set(RegisterId::R5, 1);
        self.registers.set(RegisterId::R3, r4 + 1);
        self.registers.set(self.ip_register, 12);
    }
}

impl FromStr for CPU {
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
        })
    }
}

#[test]
fn test_instruction_pointer() -> Result<()> {
    let input = "\
        #ip 0\n\
        seti 5 0 1\n\
        seti 6 0 2\n\
        addi 0 1 0\n\
        addr 1 2 3\n\
        setr 1 0 0\n\
        seti 8 0 4\n\
        seti 9 0 5\n\
    ";
    let mut cpu = input.parse::<CPU>()?;

    assert!(cpu.step().is_ok());
    assert_eq!(cpu.registers, Registers([1, 5, 0, 0, 0, 0]));
    assert!(cpu.step().is_ok());
    assert_eq!(cpu.registers, Registers([2, 5, 6, 0, 0, 0]));
    assert!(cpu.step().is_ok());
    assert_eq!(cpu.registers, Registers([4, 5, 6, 0, 0, 0]));
    assert!(cpu.step().is_ok());
    assert_eq!(cpu.registers, Registers([6, 5, 6, 0, 0, 0]));

    assert_eq!(cpu.run(), 7);

    println!("test_instruction_pointer passed.");

    Ok(())
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[test]
fn test_part_1() -> Result<()> {
    let file_name = PathBuf::from("./input/input.txt");
    println!("file_name: {:?}", file_name);
    // gets the file path relative to the cargo project dir
    let file_path = canonicalize(&file_name)?;
    println!("file_path: {:?}", file_path);
    let input = &lines_from_file(file_path).join("\n");

    let mut cpu = input.parse::<CPU>()?;

    assert_eq!(cpu.run(), 1248);
    println!("test_part_1 passed.");
    Ok(())
}

#[test]
fn test_part_2() -> Result<()> {
    let file_name = PathBuf::from("./input/input.txt");
    println!("file_name: {:?}", file_name);
    // gets the file path relative to the cargo project dir
    let file_path = canonicalize(&file_name)?;
    println!("file_path: {:?}", file_path);
    let input = &lines_from_file(file_path).join("\n");

    let mut cpu = input.parse::<CPU>()?;
    cpu.registers.set(RegisterId::R0, 1);

    assert_eq!(cpu.run(), 14952912);
    println!("test_part_2 passed.");
    Ok(())
}
