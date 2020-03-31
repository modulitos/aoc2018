#[macro_use]
extern crate lazy_static;
use regex::Regex;
use std::boxed::Box;
use std::collections::{HashSet, VecDeque};
use std::error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::ops::RangeInclusive;
use std::result;
use std::str::FromStr;

type Error = Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let mut ground = input.parse::<Ground>()?;

    let count = run_simulation(&mut ground);

    writeln!(std::io::stdout(), "final ground:\n{}\n^ final ground ^", ground)?;
    writeln!(std::io::stdout(), "number of wet areas: {}", count)?;
    writeln!(std::io::stdout(), "number of flooded areas: {}", ground.flooded_sand.len())?;

    Ok(())
}

struct ClayScan {
    x: RangeInclusive<u16>,
    y: RangeInclusive<u16>,
}

impl FromStr for ClayScan {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref X_RE: Regex =
                Regex::new("x=(?P<x>[0-9]+), y=(?P<y_start>[0-9]+)..(?P<y_end>[0-9]+)").unwrap();
        }
        lazy_static! {
            static ref Y_RE: Regex =
                Regex::new("y=(?P<y>[0-9]+), x=(?P<x_start>[0-9]+)..(?P<x_end>[0-9]+)").unwrap();
        }

        if s.starts_with("x=") {
            let caps = X_RE.captures(s).unwrap();

            let x = caps["x"].parse()?;
            let y_start = caps["y_start"].parse()?;
            let y_end = caps["y_end"].parse()?;
            Ok(Self {
                x: x..=x,
                y: y_start..=y_end,
            })
        } else if s.starts_with("y=") {
            let caps = Y_RE.captures(s).unwrap();

            let y = caps["y"].parse()?;
            let x_start = caps["x_start"].parse()?;
            let x_end = caps["x_end"].parse()?;
            Ok(Self {
                y: y..=y,
                x: x_start..=x_end,
            })
        } else {
            Err(Error::from(format!("incorrect line format: {:?}", s)))
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct Coordinate {
    y: u16,
    x: u16,
}

struct Ground {
    clay: HashSet<Coordinate>,
    wet_sand: HashSet<Coordinate>,
    flooded_sand: HashSet<Coordinate>,
    min: Coordinate, // top left
    max: Coordinate, // bottom right
}

impl FromStr for Ground {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clayscans = s
            .lines()
            .map(|line| line.parse::<ClayScan>())
            .collect::<Result<Vec<ClayScan>>>()?;
        if clayscans.len() == 0 {
            return Err(Error::from("no clayscans!"));
        }
        let x_min = clayscans
            .iter()
            .map(|scan| scan.x.start().clone())
            .min()
            // subtract 1 to account for water overflowing down the left sides of the clay
            .unwrap()
            - 1;
        let x_max = clayscans
            .iter()
            .map(|scan| scan.x.end().clone())
            .max()
            // add 1 to account for water overflowing down the right sides of the clay
            .unwrap()
            + 1;
        let y_min = clayscans
            .iter()
            .map(|scan| scan.y.start().clone())
            .min()
            .unwrap();
        let y_max = clayscans
            .iter()
            .map(|scan| scan.y.end().clone())
            .max()
            .unwrap();

        // Parse the clay scans into a HashSet<Coordinate>
        let clay = clayscans
            .into_iter()
            .fold(HashSet::<Coordinate>::new(), |mut set, scan| {
                let ClayScan { x: xs, y: ys } = scan;
                let new_set = xs
                    .flat_map(|x| {
                        ys.clone()
                            .map(|y| Coordinate { x, y })
                            .collect::<Vec<Coordinate>>()
                    })
                    .collect::<HashSet<Coordinate>>();
                set = set.union(&new_set).cloned().collect();
                set
            });

        Ok(Self {
            min: Coordinate { x: x_min, y: y_min },
            max: Coordinate { x: x_max, y: y_max },
            clay,
            wet_sand: HashSet::new(),
            flooded_sand: HashSet::new(),
        })
    }
}

impl Display for Ground {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Coordinate { x: x_min, y: y_min } = self.min;
        let Coordinate { x: x_max, y: y_max } = self.max;
        // print the first line to show the water spring.
        let first_line = (x_min..=x_max)
            .map(|x| if x == 500 { '+' } else { '.' })
            .collect::<String>();
        writeln!(f, "{}", first_line)?;
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let coord = Coordinate { x, y };
                let v = if self.clay.contains(&coord) {
                    '#'
                } else if self.wet_sand.contains(&coord) {
                    '|'
                } else if self.flooded_sand.contains(&coord) {
                    '~'
                } else {
                    '.'
                };
                write!(f, "{}", v)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Ground {
    fn _is_supported(&self, coord: &Coordinate) -> bool {
        let below = &Coordinate {
            x: coord.x,
            y: coord.y + 1,
        };
        self.flooded_sand.contains(below) || self.clay.contains(below)
    }

    // run the water from the spring downward, updating ground.wet_sand, until it is "supported".

    fn trickle_down(&mut self, coord: Coordinate) -> TrickleDownResult {
        let mut curr = coord;
        loop {
            if curr.y > self.max.y {
                return TrickleDownResult::OutOfBounds;
            }

            if self._is_supported(&curr) {
                return TrickleDownResult::SpreadAcross(curr);
            }

            self.wet_sand.insert(curr.clone());
            curr.y += 1;
        }
    }

    // spread_across(&mut self, coord): adds coords to self.wet_sand or self.saturated_sand

    // once supported, the wet_sand spreads left and right, adding to ground.wet_sand until either:

    // 1. it is no longer supported, in which case it starts going down again (repeat)

    // 2. it hits a wall of clay in it's direction.
    //
    // If there are clay walls in both directions, this run is no longer wet_sand, but added to
    // ground.saturated_sand. Then we find the cell above this run that is wet_sand, and spread that
    // left and right (repeat)

    // Also, we explicitly won't check for colliding trickle_apart ops. They will be done separately
    // for simplicity.

    fn trickle_across(&mut self, start: Coordinate) -> TrickleAcrossResult {
        // Build up an InclusiveRange to represent all of the left/right coords affected by the spread.

        #[derive(Debug)]
        enum TrickleResult {
            Flowing,
            Blocked,
            Fallthrough,
        }
        use TrickleResult::*;

        let mut left = start.clone();
        let mut left_result = TrickleResult::Flowing;
        loop {
            if !self._is_supported(&left) {
                left_result = Fallthrough;
                break;
            }

            if self.clay.contains(&Coordinate {
                x: left.x - 1,
                y: left.y,
            }) {
                left_result = Blocked;
                break;
            }
            left.x -= 1;

            if self.flooded_sand.contains(&left) {
                return TrickleAcrossResult::AlreadyFlooded
            }
        }

        let mut right = start.clone();
        let mut right_result = Flowing;
        loop {
            if !self._is_supported(&right) {
                right_result = Fallthrough;
                break;
            }

            if self.clay.contains(&Coordinate {
                x: right.x + 1,
                y: right.y,
            }) {
                right_result = Blocked;
                break;
            }
            right.x += 1;

            if self.flooded_sand.contains(&right) {
                return TrickleAcrossResult::AlreadyFlooded
            }
        }

        let y = start.y;
        (left.x..=right.x).for_each(|x| {
            self.wet_sand.insert(Coordinate { x, y });
        });
        match (&left_result, &right_result) {
            (Fallthrough, Fallthrough) => TrickleAcrossResult::TrickleDownBoth { left, right },
            (Blocked, Fallthrough) => TrickleAcrossResult::TrickleDownRight(right),
            (Fallthrough, Blocked) => TrickleAcrossResult::TrickleDownLeft(left),
            (Blocked, Blocked) => {
                let y = start.y;
                (left.x..=right.x).for_each(|x| {
                    // undo the wet sand additions, and make it flooded instead:
                    self.wet_sand.remove(&Coordinate { x, y });
                    self.flooded_sand.insert(Coordinate { x, y });
                });
                TrickleAcrossResult::Flood(Coordinate {
                    x: start.x,
                    y: start.y - 1,
                })
            }
            _ => panic!(
                "Invalid state: left_result: {:?}, right_result: {:?}",
                left_result, right_result
            ),
        }
    }
}

enum TrickleDownResult {
    OutOfBounds,
    SpreadAcross(Coordinate),
}

enum TrickleAcrossResult {
    TrickleDownLeft(Coordinate),
    TrickleDownRight(Coordinate),
    TrickleDownBoth { left: Coordinate, right: Coordinate },
    Flood(Coordinate),
    AlreadyFlooded,
}

// Updates the ground based on the water physics. Returns the sum of the ground's wet_sand and
// flooded_sand areas.

fn run_simulation(ground: &mut Ground) -> usize {
    let spring = Coordinate {
        x: 500,
        y: ground.min.y,
    };
    let mut trickle_down = VecDeque::<Coordinate>::new();
    let mut trickle_across = VecDeque::<Coordinate>::new();
    trickle_down.push_back(spring);
    loop {
        if let Some(coord) = trickle_down.pop_front() {
            match ground.trickle_down(coord) {
                TrickleDownResult::OutOfBounds => {}
                TrickleDownResult::SpreadAcross(across_coord) => {
                    trickle_across.push_back(across_coord);
                }
            }
        } else if let Some(coord) = trickle_across.pop_front() {
            use TrickleAcrossResult::*;
            match ground.trickle_across(coord) {
                TrickleDownLeft(left) => {
                    trickle_down.push_back(left);
                }
                TrickleDownRight(right) => {
                    trickle_down.push_back(right);
                }
                TrickleDownBoth { left, right } => {
                    trickle_down.push_back(left);
                    trickle_down.push_back(right);
                }
                Flood(across) => {
                    trickle_across.push_back(across);
                }
                AlreadyFlooded => {}
            }
        } else {
            break;
        }
    }

    ground.wet_sand.len() + ground.flooded_sand.len()
}

#[test]
fn test_simulation() -> Result<()> {
    let s = "\
x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504\n\
    ";

    let mut ground = s.parse::<Ground>()?;
    let output = "\
    ......+.......\n\
    ............#.\n\
    .#..#.......#.\n\
    .#..#..#......\n\
    .#..#..#......\n\
    .#.....#......\n\
    .#.....#......\n\
    .#######......\n\
    ..............\n\
    ..............\n\
    ....#.....#...\n\
    ....#.....#...\n\
    ....#.....#...\n\
    ....#######...\n\
    ";
    assert_eq!(format!("{}", ground), output);

    let count = run_simulation(&mut ground);
    assert_eq!(count, 57);

    let result = "\
    ......+.......\n\
    ......|.....#.\n\
    .#..#||||...#.\n\
    .#..#~~#|.....\n\
    .#..#~~#|.....\n\
    .#~~~~~#|.....\n\
    .#~~~~~#|.....\n\
    .#######|.....\n\
    ........|.....\n\
    ...|||||||||..\n\
    ...|#~~~~~#|..\n\
    ...|#~~~~~#|..\n\
    ...|#~~~~~#|..\n\
    ...|#######|..\n\
    ";
    println!("final ground:\n{}", ground);
    assert_eq!(format!("{}", ground), result);
    println!("test_simulation passed.");
    Ok(())
}
