#[macro_use]
extern crate lazy_static;
use regex::Regex;

mod error;

use error::{Error, Result};
use std::cmp::Ordering;
use std::io::{Read, Write};
use std::str::FromStr;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let swarm = input.parse::<Swarm>()?;
    let counts = swarm.count_in_range(&swarm.bots.iter().max().unwrap());
    writeln!(std::io::stdout(), "counts in range: {}", counts)?;
    Ok(())
}

#[derive(Hash, Eq, PartialEq)]
struct Coord {
    x: i32,
    y: i32,
    z: i32,
}

impl Coord {
    fn manhattan_distance_from(&self, other: &Coord) -> u32 {
        let diff_x = std::cmp::max(other.x, self.x) - std::cmp::min(other.x, self.x);
        let diff_y = std::cmp::max(other.y, self.y) - std::cmp::min(other.y, self.y);
        let diff_z = std::cmp::max(other.z, self.z) - std::cmp::min(other.z, self.z);

        // We know this will always be positive, so we can cast to a u32:
        (diff_x + diff_y + diff_z) as u32
    }
}

#[derive(Eq, PartialEq)]
struct Nanobot {
    coord: Coord,
    radius: u32,
}

impl Nanobot {
    // returns whether this bot is within the radius of the other bot. Signal range is measured using manhattan distance.
    fn in_range_of(&self, other: &Nanobot) -> bool {
        other.radius >= self.coord.manhattan_distance_from(&other.coord)
    }
}

impl PartialOrd for Nanobot {
    fn partial_cmp(&self, other: &Nanobot) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Nanobot {
    fn cmp(&self, other: &Nanobot) -> Ordering {
        self.radius.cmp(&other.radius)
    }
}

impl FromStr for Nanobot {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref X_RE: Regex = Regex::new(
                "^pos=<(?P<x>-?[0-9]+),(?P<y>-?[0-9]+),(?P<z>-?[0-9]+)>, r=(?P<radius>[0-9]+)$"
            )
            .unwrap();
        }

        if let Some(caps) = X_RE.captures(s) {
            let x = caps["x"].parse()?;
            let y = caps["y"].parse()?;
            let z = caps["z"].parse()?;
            let radius = caps["radius"].parse()?;

            let coord = Coord { x, y, z };

            Ok(Self { coord, radius })
        } else {
            Err(Error::from(format!("unable to parse string: {}", s)))
        }
    }
}

struct Swarm {
    bots: Vec<Nanobot>,
}

impl Swarm {
    fn count_in_range(&self, bot: &Nanobot) -> usize {
        self.bots
            .iter()
            .filter(|target| target.in_range_of(bot))
            .count()
    }
}

impl FromStr for Swarm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bots = s
            .lines()
            .map(|line| line.parse())
            .collect::<Result<Vec<Nanobot>>>()?;
        Ok(Self { bots })
    }
}

#[test]
fn test_parse_bot() -> Result<()> {
    let bot = "pos=<0,11,12>, r=4".parse::<Nanobot>()?;
    assert_eq!(bot.radius, 4);
    println!("test_parse_bot passed.");
    Ok(())
}

#[test]
fn test_parse_bot_negatives() -> Result<()> {
    let bot = "pos=<1,2,-3>, r=4".parse::<Nanobot>()?;
    assert_eq!(bot.radius, 4);
    println!("test_parse_bot passed.");
    Ok(())
}

#[test]
fn test_swarm_counts() -> Result<()> {
    let input = "\
        pos=<0,0,0>, r=4\n\
        pos=<1,0,0>, r=1\n\
        pos=<4,0,0>, r=3\n\
        pos=<0,2,0>, r=1\n\
        pos=<0,5,0>, r=3\n\
        pos=<0,0,3>, r=1\n\
        pos=<1,1,1>, r=1\n\
        pos=<1,1,2>, r=1\n\
        pos=<1,3,1>, r=1\
    ";

    let swarm = input.parse::<Swarm>()?;
    assert_eq!(swarm.count_in_range(&swarm.bots.iter().max().unwrap()), 7);

    println!("test_swarm_counts passed.");
    Ok(())
}

#[test]
fn test_swarm_from_file() -> Result<()> {
    let input = std::fs::read_to_string("./input/test.txt")?.parse::<String>()?;

    input.parse::<Swarm>()?;
    println!("test_swarm_from_file");
    Ok(())
}
