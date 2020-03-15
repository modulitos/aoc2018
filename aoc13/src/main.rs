use std::boxed;
use std::collections::BTreeMap;
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
    let coord = sim.run_to_first_collision()?;
    writeln!(
        std::io::stdout(),
        "coordinate of first collision: {:?}",
        coord
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

struct Simulation {
    track: Vec<Vec<Track>>,
    carts: BTreeMap<Coordinate, Cart>,
}

enum SimulationResult {
    Collision(Coordinate),
    LastCart(Coordinate),
    Step,
}

impl Simulation {

    fn run_to_first_collision(mut self) -> Result<Coordinate> {
        loop {
            if let SimulationResult::Collision(coord) = self.process_tick()? {
                return Ok(coord);
            }
        }
    }

    // Cycle through all carts once. If two carts collide, return the coord of the collision and
    // remove the carts from the grid.

    // Returns an error if there is an invariant violated on the track.

    fn process_tick(&mut self) -> Result<SimulationResult> {
        let previous_carts = std::mem::replace(&mut self.carts, BTreeMap::new());
        let mut iter = previous_carts.into_iter();

        while let Some((mut coord, mut cart)) = iter.next() {
            // Check whether any carts are in the current coordinate:
            if self.carts.contains_key(&coord) {
                // Note: instead of short-circuiting, we can also mark the cart as crashed here:
                return Ok(SimulationResult::Collision(coord));
            }

            coord.update_from_cart_direction(&cart.direction)?;

            // Check whether any carts are in the new coordinate:
            if self.carts.contains_key(&coord) {
                // Note: instead of short-circuiting, we can also mark the cart as crashed here:
                return Ok(SimulationResult::Collision(coord));
            }

            // update the cart's direction based on the new coordinate's track:
            let new_track = self.track[coord.y as usize][coord.x as usize];
            if let Err(error) = cart.update_from_track(&new_track) {
                // Pass along the error, but adding some extra context about the coordinate:
                return Err(Error::from(format!("{} at: {:?}", error, coord)));
            }

            self.carts.insert(coord, cart);
        }
        Ok(SimulationResult::Step)
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

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
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
            _ => panic!("unreachable code for self.turns: {}", self.turns)
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

    assert_eq!(sim.run_to_first_collision()?, Coordinate { x: 7, y: 3 });

    println!("track parsing passed!");
    Ok(())
}


#[test]
fn test_last_cart() -> Result<()> {
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
}
