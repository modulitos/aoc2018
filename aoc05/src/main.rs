use std::io::{self, Read, Write};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let input = input.trim(); // removes a trailing LF escape char

    writeln!(
        io::stdout(),
        "length of polymer left: {}",
        react(input)?.len()
    )?;

    Ok(())
}

// Note that we can return a string slice from a function only if the returned slice is derived from
// the lifetime of the originating string/slice

fn react(polymer: &str) -> Result<String> {
    if !polymer.is_ascii() {
        panic!("non-ascii!");
    }
    let mut polymer = polymer.as_bytes().to_vec();
    let mut i = 0;
    loop {
        if i + 1 >= polymer.len() {
            return Ok(String::from_utf8(polymer).expect("should not have ascii"));
        }
        if reacts(polymer[i], polymer[i + 1]) {
            // remove the reacting polymers
            polymer.remove(i + 1);
            polymer.remove(i);
            i = if i == 0 { 0 } else { i - 1 };
        } else {
            i += 1;
        }
    }
}

// returns whether the two ascii values are the same code point, but with mismatched capitalization

fn reacts(c1: u8, c2: u8) -> bool {
    if c1 < c2 {
        c2 - c1 == 32
    } else {
        c1 - c2 == 32
    }
}

#[test]
fn test_polymer() -> Result<()> {
    let polymer = "dabAcCaCBAcCcaDA";
    assert_eq!(react(polymer)?, "dabCBAcaDA");
    assert_eq!(react(polymer)?.len(), 10);
    println!("react successful!");
    Ok(())
}

#[test]
fn test_emptying_polymer() -> Result<()> {
    let polymer = "aAbB";
    assert_eq!(react(polymer)?, "");
    assert_eq!(react(polymer)?.len(), 0);
    println!("emptying successful!");
    Ok(())
}
