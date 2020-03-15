use std::boxed;
use std::collections::{BTreeMap, HashSet};
use std::error;
use std::io::{Read, Write};
use std::ops::{Add, Sub};
use std::result;
use std::str::FromStr;

type Error = boxed::Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let sim = input.parse::<Simulation>()?;
    writeln!(
        std::io::stdout(),
        "coordinate of collision: {:?}",
        sim.get_first_collision()?
    )?;

    let sim = input.parse::<Simulation>()?;
    writeln!(
        std::io::stdout(),
        "coordinate of last cart: {:?}",
        sim.get_last_cart()?
    )?;
    Ok(())
}

#[derive(Copy, Clone, Debug)]
enum Track {
    Empty,
    Vertical,
    Horizontal,
    // when cart hits a junction, it turn LEFT, then STRAIGHT, then RIGHT, then repeats
    Junction,
    CurveForward,  // forward slash: /
    CurveBackward, // back slash: \
}

enum SimulationResult {
    Collision(Coordinate),
    LastCart(Coordinate), // returns coord of last cart, if there is one
    Step,
}

struct Simulation {
    track: Vec<Vec<Track>>,
    carts: BTreeMap<Coordinate, Cart>,
}

impl Simulation {
    fn get_first_collision(self) -> Result<Coordinate> {
        if let Some(coord) = self
            .into_iter()
            .collect::<Result<Vec<SimulationResult>>>()?
            .into_iter()
            .find_map(|result| {
                if let SimulationResult::Collision(coord) = result {
                    Some(coord)
                } else {
                    None
                }
            })
        {
            Ok(coord)
        } else {
            Err(Error::from("no collision found!"))
        }
    }

    fn get_last_cart(self) -> Result<Coordinate> {
        if let Some(coord) = self
            .into_iter()
            .collect::<Result<Vec<SimulationResult>>>()?
            .into_iter()
            .find_map(|result| {
                if let SimulationResult::LastCart(coord) = result {
                    Some(coord)
                } else {
                    None
                }
            })
        {
            Ok(coord)
        } else {
            Err(Error::from("no last cart found!"))
        }
    }

    fn into_iter(self) -> SimulationIter {
        SimulationIter {
            track: self.track,
            carts: self.carts,
            error_found: false,
        }
    }
}

impl FromStr for Simulation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Simulation, Self::Err> {
        let mut carts = BTreeMap::new();

        let track = s
            .lines()
            .enumerate()
            .map(|(y, line)| {
                Ok(line
                    .as_bytes()
                    .iter()
                    .enumerate()
                    .map(|(x, c)| {
                        use Track::*;
                        match c {
                            b'+' => Ok(Junction),
                            b'|' => Ok(Vertical),
                            b'-' => Ok(Horizontal),
                            b'/' => Ok(CurveForward),
                            b'\\' => Ok(CurveBackward),
                            b' ' => Ok(Empty),
                            c => {
                                let cart = Cart::from_char(c)?;
                                let direction = cart.direction;
                                carts.insert(
                                    Coordinate {
                                        x: x as u32,
                                        y: y as u32,
                                    },
                                    cart,
                                );
                                match direction {
                                    Direction::Up | Direction::Down => Ok(Vertical),
                                    Direction::Left | Direction::Right => Ok(Horizontal),
                                }
                            }
                        }
                    })
                    .collect::<Result<Vec<Track>>>()?)
            })
            .collect::<Result<Vec<Vec<Track>>>>()?;

        Ok(Simulation { track, carts })
    }
}

struct SimulationIter {
    track: Vec<Vec<Track>>,
    carts: BTreeMap<Coordinate, Cart>,
    // This is for easier error handling within the iterator:
    // https://users.rust-lang.org/t/handling-errors-from-iterators/2551/14
    // TODO: But maybe a loop would've been better than an iterator here, to avoid nesting Option<Result<...>>?
    // OR maybe it's better to panic than return Err's from the iterator?
    error_found: bool,
}

impl Iterator for SimulationIter {
    type Item = Result<SimulationResult>;

    // Cycle through all carts once. If two carts collide, return the coord of the collision and
    // remove the carts from the grid. Return the coord of the last remaining cart, if there is one.

    // Returns an error if there is an invariant violated on the track.

    fn next(&mut self) -> Option<Self::Item> {
        if self.carts.len() == 0 || self.error_found {
            return None;
        } else if self.carts.len() == 1 {
            // Base case: there are 1 or 0 carts left.
            let coord = self.carts.keys().cloned().last().unwrap();
            self.carts.clear();
            return Some(Ok(SimulationResult::LastCart(coord)));
        }
        let previous_carts = std::mem::replace(&mut self.carts, BTreeMap::new());
        let mut previous_cart_coords = previous_carts
            .keys()
            .cloned()
            .collect::<HashSet<Coordinate>>();
        let mut crash_coords = HashSet::new();

        let mut first_collision_coord = None;
        let mut iter = previous_carts.into_iter();
        while let Some((mut coord, mut cart)) = iter.next() {
            if crash_coords.contains(&coord) {

                // another cart has run into this cart on a previous round, but the cart wasn't yet
                // removed

                continue;
            }

            previous_cart_coords.remove(&coord);

            if let Err(error) = coord.update_from_cart_direction(&cart.direction) {
                self.error_found = true;
                return Some(Err(error));
            }

            // update the cart's direction based on the new coordinate's track:
            let new_track = self.track[coord.y as usize][coord.x as usize];
            if let Err(error) = cart.update_from_track(&new_track) {
                self.error_found = true;
                // Pass along the error, but adding some extra context about the coordinate:
                return Some(Err(Error::from(format!("{} at: {:?}", error, coord))));
            }

            // Check whether any carts are in the new coordinate:
            if crash_coords.contains(&coord)
                || self.carts.contains_key(&coord)
                || previous_cart_coords.contains(&coord)
            {
                crash_coords.insert(coord);
                // remove the crashed cart, and don't add this cart to our collection.
                self.carts.remove(&coord);
                if first_collision_coord.is_none() {
                    first_collision_coord = Some(coord);
                }
            } else {
                // There was no collision, so update the cart with its new location:
                self.carts.insert(coord.clone(), cart);
            }
        }
        if let Some(coord) = first_collision_coord {
            Some(Ok(SimulationResult::Collision(coord)))
        } else {
            Some(Ok(SimulationResult::Step))
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Debug, Hash)]
struct Coordinate {
    y: u32,
    x: u32,
}

impl Coordinate {
    fn update_from_cart_direction(&mut self, cart_kind: &Direction) -> Result<()> {
        use Direction::*;
        match cart_kind {
            Up => self.y -= 1,
            Down => self.y += 1,
            Left => self.x -= 1,
            Right => self.x += 1,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

// Enables us to add a number n to a direction, to rotate that direction n times 90 degrees clockwise.

impl Add<u32> for Direction {
    type Output = Self;

    fn add(self, n: u32) -> Self::Output {
        use Direction::*;
        fn get_clockwise(direction: &Direction) -> Direction {
            match direction {
                Up => Right,
                Right => Down,
                Down => Left,
                Left => Up,
            }
        }
        let mut curr = self;
        for _ in 0..(n % 4) {
            curr = get_clockwise(&curr)
        }
        curr
    }
}

impl Sub<u32> for Direction {
    type Output = Self;

    fn sub(self, n: u32) -> Self::Output {
        use Direction::*;
        fn get_counter_clockwise(direction: &Direction) -> Direction {
            match direction {
                Up => Left,
                Right => Up,
                Down => Right,
                Left => Down,
            }
        }
        let mut curr = self;
        for _ in 0..(n % 4) {
            curr = get_counter_clockwise(&curr)
        }
        curr
    }
}

struct Cart {
    direction: Direction,
    turns: u32,
}

impl Cart {
    fn from_char(c: &u8) -> Result<Self> {
        use Direction::*;
        let direction = match (c) {
            b'^' => Up,
            b'v' => Down,
            b'>' => Right,
            b'<' => Left,
            _ => {
                return Err(Error::from(format!(
                    "unable to build cart from input: {}",
                    c
                )))
            }
        };
        Ok(Cart {
            direction,
            turns: 0,
        })
    }

    fn turn_on_junction(&mut self, direction: Direction) -> Direction {
        self.turns = (self.turns + 1) % 3;
        match self.turns {
            0 => direction + 1, // turn right
            1 => direction - 1, // turn left
            2 => direction,     // go straight
            _ => panic!("unreachable code for self.turns: {}", self.turns),
        }
    }

    // update the cart's kind based on the new track it's on. If an invariant between the cart's
    // direction and the cart's next steps is violated, then return an error.

    fn update_from_track(&mut self, new_track: &Track) -> Result<(), String> {
        fn track_error(track: &Track, direction: &Direction) -> Result<(), String> {
            Err(format!(
                "invalid state: on track: {:?}, with cart direction: {:?}",
                track, direction
            ))
        }

        use Direction::*;
        use Track::*;

        // TODO: this can be simplified by rotating the direction on "UP", calculating the resulting
        // direction based on UP, then applying the inverse rotations.

        let new_direction = match (self.direction, new_track) {
            (kind, Empty) => return track_error(&Empty, &kind),
            (direction, Junction) => self.turn_on_junction(direction),
            (Up, Horizontal) => return track_error(&Horizontal, &Up),
            (Up, Vertical) => Up,
            (Up, CurveForward) => Up + 1,
            (Up, CurveBackward) => Up - 1,
            (Right, Horizontal) => Right,
            (Right, Vertical) => return track_error(&Vertical, &Right),
            (Right, CurveForward) => Right - 1,
            (Right, CurveBackward) => Right + 1,
            (Down, Horizontal) => return track_error(&Horizontal, &Up),
            (Down, Vertical) => Down,
            (Down, CurveForward) => Down + 1,
            (Down, CurveBackward) => Down - 1,
            (Left, Horizontal) => Left,
            (Left, Vertical) => return track_error(&Vertical, &Left),
            (Left, CurveForward) => Left - 1,
            (Left, CurveBackward) => Left + 1,
        };
        self.direction = new_direction;
        Ok(())
    }
}

#[test]
fn test_first_crash_detection() -> Result<()> {
    let s = r"/->-\
|   |  /----\
| /-+--+-\  |
| | |  | v  |
\-+-/  \-+--/
  \------/";

    println!("s: \n{}", s);
    let sim = s.parse::<Simulation>()?;

    println!("getting first collision...");
    assert_eq!(sim.get_first_collision()?, Coordinate { x: 7, y: 3 });

    println!("test_first_crash_detection passed!");
    Ok(())
}

#[test]
fn test_last_cart() -> Result<()> {
    let s = r"/>-<\
|   |
| /<+-\
| | | v
\>+</ |
  |   ^
  \<->/";
    println!("s: \n{}", s);
    let sim = s.parse::<Simulation>()?;

    println!("testing last_cart...");
    assert_eq!(sim.get_last_cart()?, Coordinate { x: 6, y: 4 });

    println!("test_last_cart passed!");
    Ok(())
}

#[test]
fn test_btree_sorts_coord_keys() {
    let coord_1 = Coordinate { x: 2, y: 8 };
    let coord_2 = Coordinate { x: 1, y: 8 };
    let coord_3 = Coordinate { x: 1, y: 9 };
    let coord_4 = Coordinate { x: 3, y: 1 };

    let mut map = BTreeMap::new();
    map.insert(coord_1, 'c');
    map.insert(coord_2, 'b');
    map.insert(coord_3, 'd');
    map.insert(coord_4, 'a');

    assert_eq!(
        map.iter().map(|(_, &v)| v).collect::<Vec<char>>(),
        vec!['a', 'b', 'c', 'd']
    );

    println!("test btree passed!");
}

#[test]
fn test_direction_arithmetic() {
    use Direction::*;
    assert_eq!(Up + 1, Right);
    assert_eq!(Up + 3, Left);
    assert_eq!(Up + 2, Down);
    assert_eq!(Up - 1, Left);
    assert_eq!(Up - 2, Down);
    assert_eq!(Left + 2, Right);
    assert_eq!(Left - 2, Right);
    assert_eq!(Left - 4, Left);
    assert_eq!(Left + 4, Left);
    println!("test direction arithmetic passed!");
}
