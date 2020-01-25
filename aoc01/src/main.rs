use std::io::{self, Read, Write};
use std::iter::Map;

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    let sum: i32 = input.lines().map(|item| {
        // writeln!(io::stdout(), "unparsed item: {}", item);
        item.parse::<i32>().unwrap()
    }).sum(); // how to preven integer overflow?

    writeln!(io::stdout(), "sum: {}", sum);
    Ok(())
}

#[test]
fn sum() {}
