#[macro_use]
extern crate lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::str::FromStr;

type Error = std::boxed::Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let grid = input.parse::<Grid>()?;
    writeln!(std::io::stdout(), "\n\nmessage:\n{}", grid.get_message()?)?;
    Ok(())
}

struct Grid {
    points: Vec<Point>,
    index: HashSet<(i32, i32)>, // index point's x/y coords for querying
}

struct Bounds {
    minx: i32,
    maxx: i32,
    miny: i32,
    maxy: i32,
}

impl Grid {
    // Update the points, and update the indexes for our Grid's points mapping

    fn step(&mut self) {
        let mut index = HashSet::new();
        self.points.iter_mut().for_each(|mut point| {
            point.step();
            index.insert((point.x, point.y));
        });
        self.index = index;
    }

    // Gets the min/max bounds for our x/y coords
    fn get_bounds(&self) -> Bounds {
        self.points.iter().fold(
            Bounds {
                minx: i32::max_value(),
                maxx: i32::min_value(),
                miny: i32::max_value(),
                maxy: i32::min_value(),
            },
            |mut bounds, point| {
                bounds.minx = std::cmp::min(bounds.minx, point.x);
                bounds.maxx = std::cmp::max(bounds.maxx, point.x);
                bounds.miny = std::cmp::min(bounds.miny, point.y);
                bounds.maxy = std::cmp::max(bounds.maxy, point.y);
                bounds
            },
        )
    }

    // iterates until we hit the message

    fn get_message(mut self) -> Result<String> {
        for i in 0..1_000_000 {
            self.step();
            if self.message_found() {
                return Ok(self.to_str());
            }
        }
        Err(Error::from(
            "Unable to find a message after 1,000,000 seconds!",
        ))
    }

    // Determine whether the message has been found, according to criteria where if there are no
    // more than 5% of the points that are surrounded by nothing but spaces.

    fn message_found(&self) -> bool {
        let disjoint_max = (f32::from(self.points.len() as u16) * 0.05).ceil() as usize;
        let disjoint_count = self
            .points
            .iter()
            .filter(|&point| {
                point
                    .get_adjacent()
                    .iter()
                    .all(|&(x, y)| !self.index.contains(&(x, y)))
            })
            .count();
        disjoint_count <= disjoint_max
    }

    fn to_str(&self) -> String {
        let bounds = self.get_bounds();

        (bounds.miny..=bounds.maxy)
            .map(|y| {
                (bounds.minx..=bounds.maxx)
                    .map(|x| {
                        if self.index.contains(&(x, y)) {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl FromStr for Grid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let points = s
            .lines()
            .map(|line| {
                lazy_static! {
                    static ref RE: Regex = Regex::new(
                        r"(?x)
                          position=<\s*(?P<x>-?[0-9]+),\s*(?P<y>-?[0-9]+)>
                          \svelocity=<\s*(?P<vx>-?[0-9]+),\s*(?P<vy>-?[0-9]+)>
                         "
                    )
                    .unwrap();
                }

                let caps = RE.captures(line).unwrap();

                Ok(Point {
                    x: caps["x"].parse()?,
                    y: caps["y"].parse()?,
                    vx: caps["vx"].parse()?,
                    vy: caps["vy"].parse()?,
                })
            })
            .collect::<Result<Vec<Point>>>()?;
        Ok(Grid {
            index: points.iter().fold(HashSet::new(), |mut index, point| {
                index.insert((point.x, point.y));
                index
            }),
            points,
        })
    }
}

// Useful for printing the results

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

struct Point {
    x: i32,
    y: i32,
    vx: i8,
    vy: i8,
}

impl Point {
    fn step(&mut self) {
        self.x = self.x.saturating_add(i32::from(self.vx));
        self.y = self.y.saturating_add(i32::from(self.vy));
    }

    fn get_adjacent(&self) -> [(i32, i32); 8] {
        [
            (self.x + 1, self.y),
            (self.x + 1, self.y + 1),
            (self.x + 1, self.y - 1),
            (self.x, self.y - 1),
            (self.x, self.y + 1),
            (self.x - 1, self.y),
            (self.x - 1, self.y + 1),
            (self.x - 1, self.y - 1),
        ]
    }
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[test]
fn test() -> Result<()> {
    let input = "\
        position=< 9,  1> velocity=< 0,  2>
        position=< 7,  0> velocity=<-1,  0>
        position=< 3, -2> velocity=<-1,  1>
        position=< 6, 10> velocity=<-2, -1>
        position=< 2, -4> velocity=< 2,  2>
        position=<-6, 10> velocity=< 2, -2>
        position=< 1,  8> velocity=< 1, -1>
        position=< 1,  7> velocity=< 1,  0>
        position=<-3, 11> velocity=< 1, -2>
        position=< 7,  6> velocity=<-1, -1>
        position=<-2,  3> velocity=< 1,  0>
        position=<-4,  3> velocity=< 2,  0>
        position=<10, -3> velocity=<-1,  1>
        position=< 5, 11> velocity=< 1, -2>
        position=< 4,  7> velocity=< 0, -1>
        position=< 8, -2> velocity=< 0,  1>
        position=<15,  0> velocity=<-2,  0>
        position=< 1,  6> velocity=< 1,  0>
        position=< 8,  9> velocity=< 0, -1>
        position=< 3,  3> velocity=<-1,  1>
        position=< 0,  5> velocity=< 0, -1>
        position=<-2,  2> velocity=< 2,  0>
        position=< 5, -2> velocity=< 1,  2>
        position=< 1,  4> velocity=< 2,  1>
        position=<-2,  7> velocity=< 2, -2>
        position=< 3,  6> velocity=<-1, -1>
        position=< 5,  0> velocity=< 1,  0>
        position=<-6,  0> velocity=< 2,  0>
        position=< 5,  9> velocity=< 1, -2>
        position=<14,  7> velocity=<-2,  0>
        position=<-3,  6> velocity=< 2, -1>\
    ";
    let grid = input.parse::<Grid>()?;
    assert_eq!(grid.points.len(), 31);
    assert_eq!(
        grid.get_message()?,
        "\
            #...#..###\n\
            #...#...#.\n\
            #...#...#.\n\
            #####...#.\n\
            #...#...#.\n\
            #...#...#.\n\
            #...#...#.\n\
            #...#..###\
        "
        .to_string()
    );
    println!("parsed points.");
    Ok(())
}
