use std::collections::HashSet;
use std::io::{self, Lines, Read, Write};
use std::iter::Map;

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    // Parse the input from string into numbers:

    let sum = part1(&input)?;
    writeln!(io::stdout(), "sum: {}", sum);

    let repeated_frequency = part2(&input)?;
    writeln!(io::stdout(), "first repeated freq: {}", repeated_frequency);

    Ok(())
}

fn get_nums(input: &str) -> Map<std::str::Lines, fn(&str) -> i32> {
    // TODO: Can we unwrap in a way that surfaces this error, without having to resort to a for-loop? It
    // would be ideal to use the '?' operator instead of expect to unwrap the result from
    // item.parse: https://stackoverflow.com/a/26370894/1884158

    input.lines().map(|item| {
        item.parse::<i32>()
            .expect("Unable to parse item into an i32")
    })
}

// Return a sum of the numbers.

fn part1(input: &str) -> Result<i32> {
    // TODO: how to prevent integer overflow when summing?
    Ok(get_nums(input).sum())
}

// Find the value of the first ongoing sum that repeats twice, and looping through the nums if
// necessary.

fn part2(input: &str) -> Result<i32> {
    let mut seen: HashSet<i32> = HashSet::new();

    let mut freq = 0;
    seen.insert(freq);

    // TODO: Is there a way to do this without a loop?
    loop {
        if get_nums(input)
            .find(|num| {
                freq += *num;
                if seen.contains(&freq) {
                    true
                } else {
                    seen.insert(freq);
                    false
                }
            })
            .is_some()
        {
            return Ok(freq);
        }
    }
}

#[test]
fn test_part1() -> Result<()> {
    let s = "0\n1\n-5\n+3";
    assert_eq!(part1(s)?, -1);
    println!("test_part1 passed!");
    Ok(())
}

#[test]
fn test_part2() -> Result<()> {
    let s = "1\n-1";
    assert_eq!(part2(s)?, 0);

    let s = "3\n3\n4\n-2\n-4";
    assert_eq!(part2(s)?, 10);

    let s = "-6\n3\n8\n5\n-6";
    assert_eq!(part2(s)?, 5);

    let s = "7\n7\n-2\n-7\n-4";
    assert_eq!(part2(s)?, 14);

    println!("test_part2 passed!");
    Ok(())
}
