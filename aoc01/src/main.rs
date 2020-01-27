use std::io::{self, Read, Write};

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    let sum = part1(&input)?;
    writeln!(io::stdout(), "sum: {}", sum);

    part2(&input);

    Ok(())
}

// Return a sum of the numbers.
fn part1(input: &str) -> Result<i32> {
    // how to prevent integer overflow when summing?

    // Can we unwrap in a way that surfaces this error, without having to resort to a for-loop? It
    // would be ideal to use the '?' operator instead of expect to unwrap the result from
    // item.parse: https://stackoverflow.com/a/26370894/1884158

    Ok(input.lines().map(|item| item.parse::<i32>().expect("Unable to parse item into an i32")).sum())
}

//
fn part2(input: &str) {

}

#[test]
fn test_part1() -> Result<()> {
    let s = "0\n1\n-5\n+3";
    assert_eq!(part1(&s)?, -1);
    println!("test_part1 passed!");
    Ok(())
}
