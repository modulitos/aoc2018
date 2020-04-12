mod error;

use error::{Error, Result};
use std::collections::HashMap;
use std::fs::{canonicalize, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let map = input.trim().parse::<Map>()?;
    writeln!(
        std::io::stdout(),
        "room with greatest distance, ie 'number of doors to pass through': {}",
        map.get_distance_to_furthest_room()
    )?;
    writeln!(
        std::io::stdout(),
        "count of rooms requiring at least 1000 doors to pass through: {}",
        map.count_rooms_at_least_1000()
    )?;
    Ok(())
}

enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn from_char(c: char) -> Result<Direction> {
        Ok(match c {
            'N' => Direction::North,
            'E' => Direction::East,
            'S' => Direction::South,
            'W' => Direction::West,
            _ => {
                return Err(Error::from(format!(
                    "cannot parse direction from char: {}",
                    c
                )))
            }
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Coordinate {
    y: i32,
    x: i32,
}

impl Coordinate {
    // Moves the coordinate in the provided direction

    fn move_in_direction(&self, direction: Direction) -> Coordinate {
        use Direction::*;
        match direction {
            North => Coordinate {
                x: self.x,
                y: self.y - 1,
            },
            South => Coordinate {
                x: self.x,
                y: self.y + 1,
            },
            East => Coordinate {
                x: self.x + 1,
                y: self.y,
            },
            West => Coordinate {
                x: self.x - 1,
                y: self.y,
            },
        }
    }
}

// The number of doors that must be opened to reach a given Coordinate

type Distance = u16;

struct Map {
    distances: HashMap<Coordinate, Distance>,
}

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut distances = HashMap::new();

        // TODO: Is there a more functional way to do this?

        let mut curr = Coordinate { x: 0, y: 0 };
        distances.insert(curr.clone(), 0);

        let mut stack = Vec::<Coordinate>::new();

        for c in s.trim_start_matches("^").trim_end_matches("$").chars() {
            if let Ok(direction) = Direction::from_char(c) {
                let new_curr = curr.move_in_direction(direction);
                let next_distance = *distances.get(&curr).unwrap() + 1;
                distances
                    .entry(new_curr.clone())
                    .and_modify(|e| {
                        // if this path already exists, and the new path is less than the old one:

                        if &next_distance < e {
                            *e = next_distance;
                        }
                    })
                    .or_insert(next_distance);
                curr = new_curr;
            } else {
                match c {
                    '(' => {
                        stack.push(curr.clone());
                    }
                    '|' => {
                        curr = stack.last().unwrap().clone();
                    }
                    ')' => {
                        curr = stack.pop().unwrap();
                    }
                    _ => {
                        return Err(Error::from(format!("invalid char: {}", c)));
                    }
                };
            }
        }
        Ok(Map { distances })
    }
}

impl Map {
    // Returns the number of doors required to access the "furthest" room. Furthest is defined by
    // the number of doors required to pass through from the start point (0,0).

    fn get_distance_to_furthest_room(&self) -> u16 {
        // println!("self.distances: {:?}", self.distances);
        *self.distances.values().max().unwrap()
    }

    fn count_rooms_at_least_1000(&self) -> usize {
        // println!("self.distances: {:?}", self.distances);
        self.distances.values().filter(|v| v >= &&1000).count()
    }
}

#[test]
fn test_flat_route() -> Result<()> {
    let input = "^WNE$";
    let map = input.parse::<Map>()?;
    assert_eq!(map.get_distance_to_furthest_room(), 3);
    println!("test test_flat_route passed.");
    Ok(())
}

#[test]
fn test_nested_branched_route() -> Result<()> {
    let input = "^ENWWW(NEEE|SSE(EE|N))$";
    let map = input.parse::<Map>()?;
    assert_eq!(map.get_distance_to_furthest_room(), 10);
    println!("test_nested_branched_route passed.");
    Ok(())
}

#[test]
fn test_multi_branched_route() -> Result<()> {
    let input = "^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$";
    let map = input.parse::<Map>()?;
    assert_eq!(map.get_distance_to_furthest_room(), 18);
    println!("test test_multi_branched_route.");
    Ok(())
}

#[test]
fn test_read_from_file() -> Result<()> {
    let input = std::fs::read_to_string("./input/test_flat.txt")?.parse::<String>()?;

    let map = input.trim().parse::<Map>()?;

    assert_eq!(map.get_distance_to_furthest_room(), 3);
    println!("test_read_from_file passed.");
    Ok(())
}
