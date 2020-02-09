use std::io::{self, Read, Write};

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

// TODO: figure out how to encode a string that is only ascii, and have this function
fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let input = input.trim(); // removes a trailing LF escape char

    writeln!(
        io::stdout(),
        "length of polymer left: {}",
        react(input)?.len()
    )?;

    writeln!(
        io::stdout(),
        "length of shortest inert polymer after 1 pair removal: {}",
        find_shortest_inert_length(input)?
    )?;

    Ok(())
}

// Note that we can return a string slice from a function only if the returned slice is derived from
// the lifetime of the originating string/slice

type InvalidInput = String;

fn react(polymer: &str) -> Result<String, InvalidInput> {
    if !polymer.is_ascii() {
        return Err(String::from("input string is non-ascii!"));
    }
    let mut polymer = polymer.as_bytes().to_vec();
    let mut i = 0;
    loop {
        if i + 1 >= polymer.len() {
            return Ok(String::from_utf8(polymer).expect("should not have non-utf8 string"));
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

// find the shortest inert length after removing one polymer pair

fn find_shortest_inert_length(polymer: &str) -> Result<usize, String> {
    let res = (b'A'..b'Z')
        .map(|byte| -> Result<usize, String> {
            let byte_pair = byte + 32;
            let test_polymer = polymer
                .replace(char::from(byte), "")
                .replace(char::from(byte_pair), "");
            react(&test_polymer).and_then(|inert| Ok(inert.len()))
        })
        .collect::<Result<Vec<usize>, String>>()?;

    Ok(res
        .into_iter()
        .min()
        .expect("should not have an empty iter"))
}

#[test]
fn test_shortest_inert_length() -> Result<()> {
    let polymer = "dabAcCaCBAcCcaDA";
    assert_eq!(find_shortest_inert_length(polymer)?, 4);
    println!("shortest inert length successful!");
    Ok(())
}

#[test]
fn test_react() -> Result<()> {
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
