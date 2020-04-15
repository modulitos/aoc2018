mod error;
mod op_codes;
mod vm;

use error::{Error, Result};
// we need to use these here so that our vm module can bring them into scope:
use op_codes::{Op, Opcode, OpcodeId, RegisterId, Registers, RegisterValue};
use std::io::{Read, Write};
use vm::{VM, Part};
// use vm;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    // part 1:
    let mut vm = input.parse::<VM>()?.set_r0(0);
    writeln!(std::io::stdout(), "part1: {}", vm.run(Part::One))?;

    // part 2:
    let mut vm = input.parse::<VM>()?.set_r0(0);

    writeln!(std::io::stdout(), "part2: {}", vm.run(Part::Two))?;

    Ok(())
}

#[test]
fn test() -> Result<()> {
    println!("test passed.");
    Ok(())
}
