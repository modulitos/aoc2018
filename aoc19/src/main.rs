mod error;
mod op_codes;

use error::{Error, Result};
use op_codes::{Opcode, OpcodeName, Registers};

fn main() -> Result<()> {
    println!("Hello, world!");
    Ok(())
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
    println!("test_instruction_pointer passed.");
    Ok(())
}
