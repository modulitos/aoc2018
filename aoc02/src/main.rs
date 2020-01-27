use std::collections::HashMap;
use std::io::{self, Read, Write};

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    writeln!(io::stdout(), "checksum: {}", get_checksum(&input)?)?;

    Ok(())
}

fn get_checksum(input: &str) -> Result<i32> {
    let mut twos = 0;
    let mut threes = 0;

    for line in input.lines() {
        // Note: If assuming only ASCII chars, this can be done in a byte array.

        let mut counts = HashMap::new();
        // generate a counts mapping for all our chars:
        for c in line.chars() {
            counts
                .entry(c)
                .and_modify(|v: &mut i32| *v = v.saturating_add(1))
                .or_insert(1);
        }

        if counts.values().find(|v| **v == 2).is_some() {
            twos += 1;
        }

        if counts.values().find(|v| **v == 3).is_some() {
            threes += 1;
        }
    }

    Ok(twos * threes)
}

#[test]
fn test_checksum() -> Result<()> {
    let s = "asdf\nasdf";
    assert_eq!(get_checksum(s)?, 0);

    let s = "aasdf";
    assert_eq!(get_checksum(s)?, 0);

    let s = "aaassdf";
    assert_eq!(get_checksum(s)?, 1);

    let s = "aaassdf\naaassdf";
    assert_eq!(get_checksum(s)?, 4);

    let s = "aasdf\naaassdf\naaa";
    assert_eq!(get_checksum(s)?, 4);

    let s = "aasdf\naaassdf\naaa\nxxx";
    assert_eq!(get_checksum(s)?, 6);

    println!("get_checksum passed!");
    Ok(())
}
