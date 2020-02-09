use std::io::{self, Read, Write};
use std::str::FromStr;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let input = input.trim().parse::<AsciiEncodedString>()?; // removes a trailing LF escape char

    writeln!(
        io::stdout(),
        "length of polymer left: {}",
        react(&input).len()
    )?;

    writeln!(
        io::stdout(),
        "length of shortest inert polymer after 1 pair removal: {}",
        find_shortest_inert_length(&input)
    )?;

    Ok(())
}

struct AsciiEncodedString(pub String);

impl FromStr for AsciiEncodedString {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(String::from("input string is non-ascii!"));
        }
        Ok(AsciiEncodedString(String::from(s)))
    }
}

// Note that we can return a string slice from a function only if the returned slice is derived from
// the lifetime of the originating string/slice

fn react(polymer: &AsciiEncodedString) -> String {
    let mut polymer = polymer.0.as_bytes().to_vec();
    let mut i = 0;
    loop {
        if i + 1 >= polymer.len() {
            return String::from_utf8(polymer).expect("should not have non-utf8 string");
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

fn find_shortest_inert_length(polymer: &AsciiEncodedString) -> usize {
    (b'A'..b'Z')
        .map(|byte| {
            let byte_pair = byte + 32;
            let test_polymer = polymer
                .0
                .replace(char::from(byte), "")
                .replace(char::from(byte_pair), "")
                .parse()
                .expect("test_polymer should remain ascii encoded");
            react(&test_polymer).len()
        })
        .min()
        .expect("should not have an empty iter")
}

#[test]
fn test_shortest_inert_length() -> Result<()> {
    let polymer = "dabAcCaCBAcCcaDA".parse()?;
    assert_eq!(find_shortest_inert_length(&polymer), 4);
    println!("shortest inert length successful!");
    Ok(())
}

#[test]
fn test_react() -> Result<()> {
    let polymer = "dabAcCaCBAcCcaDA".parse()?;
    assert_eq!(react(&polymer), "dabCBAcaDA");
    assert_eq!(react(&polymer).len(), 10);
    println!("react successful!");
    Ok(())
}

#[test]
fn test_emptying_polymer() -> Result<()> {
    let polymer = "aAbB".parse()?;
    assert_eq!(react(&polymer), "");
    assert_eq!(react(&polymer).len(), 0);
    println!("emptying successful!");
    Ok(())
}
