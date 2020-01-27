use std::collections::HashMap;
use std::fmt::Error;
use std::io::{self, Read, Write};

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    writeln!(io::stdout(), "checksum: {}", get_checksum(&input)?)?;

    writeln!(
        io::stdout(),
        "common letters: {}",
        get_common_letters(&input)?
    )?;

    Ok(())
}

// part 1
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

// part 2
fn get_common_letters(input: &str) -> Result<String> {
    let lines: Vec<&str> = input.lines().collect();

    for (i, line_1) in lines.iter().enumerate() {
        for line_2 in lines[i + 1..].iter() {
            if line_1.len() != line_2.len() {
                continue;
            }

            if !line_1.is_ascii() || !line_2.is_ascii() {
                return Err(From::from("All input must be ascii"));
            }

            // Determine whether our two string differ by more than one char:

            let mut mismatch_found = false;
            let result: String = line_1
                .chars()
                .zip(line_2.chars())
                // Perhaps a Rust filter_while would be ideal here?
                .take_while(|&(c_1, c_2)| {
                    if c_1 != c_2 {
                        if mismatch_found {
                            false;
                        } else {
                            mismatch_found = true;
                            true;
                        }
                    }
                    true
                })
                .filter_map(|(c_1, c_2)| if c_1 == c_2 { Some(c_1) } else { None })
                .collect();

            if result.len() == line_1.len() - 1 {
                return Ok(result);
            }
        }
    }

    Err(From::from("No matches found!"))
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

#[test]
fn test_common_letters() -> Result<()> {
    let s = "abcde\nfghij\nklmno\npqrst\nfguij\naxcye\nwvxyz\n";
    assert_eq!(get_common_letters(s)?, "fgij");

    let s = "abcde\nfghix\nklmno\npqrst\nfguij\naxcye\nwvxyz\n";
    assert!(get_common_letters(s).is_err());

    println!("get_common_letters passed!");
    Ok(())
}
