use std::boxed::Box;
use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::error;
use std::fmt::{Display, Formatter};
use std::io::Read;
use std::result;
use std::str::FromStr;

type Error = Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let mut sim = input.parse::<Simulation>()?;
    sim.tick();
    Ok(())
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Debug)]
struct Coordinate {
    y: u16,
    x: u16,
}

enum PlayerKind {
    Elf,
    Goblin,
}

#[derive(Debug, Eq, PartialEq)]
enum PlayerAction {
    Stay,               // No accessible enemies to attack, so stay in place
    Move(Coordinate),   // Move towards an opponent
    Attack(Coordinate), // Attacks the player at the given coordinate
}

struct Player {
    health: u16,
    kind: PlayerKind,
}

// Associates the number of steps it takes to get to a given coordinate
#[derive(Eq, PartialEq, Clone, Debug)]
struct Link {
    steps: u16,
    coord: Coordinate,
}

impl PartialOrd for Link {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other)) // Delegate to the implementation in `Ord`.
    }
}

impl Ord for Link {
    fn cmp(&self, other: &Self) -> Ordering {
        self.steps.cmp(&other.steps)
    }
}

impl Player {
    fn new(kind: PlayerKind) -> Self {
        Player { health: 200, kind }
    }
    fn to_char(&self) -> char {
        use PlayerKind::*;
        match self.kind {
            Goblin => 'G',
            Elf => 'E',
        }
    }

    fn is_opponent(&self, other: &Player) -> bool {
        use PlayerKind::*;
        match (&self.kind, &other.kind) {
            (Elf, Goblin) => true,
            (Goblin, Elf) => true,
            _ => false,
        }
    }

    // updates the player's position by moving one step toward the nearest, reachable, opponent. If
    // multiple steps are available, then the first one in reading order is chosen.
    //
    // Returns None if the player doesn't move.

    fn step<'a>(
        &self,
        current_pos: &Coordinate,
        arena: &Arena,
        players: &BTreeMap<Coordinate, Player>, // list of all players, except this player
    ) -> PlayerAction {
        // Use Dijkstra's algorithm to track each accessible cell in the arena, and map how many
        // steps it takes to get there, along with a link to the existing point to get to that point

        let mut heap = BinaryHeap::<Reverse<Link>>::new();
        // If the map's value is None, then there are no steps required to get to that coord.
        let mut pathfinder = HashMap::<Coordinate, Option<Link>>::new();
        pathfinder.insert(current_pos.clone(), None);
        heap.push(Reverse(Link {
            steps: 0,
            coord: current_pos.clone(),
        }));

        while let Some(Reverse(link)) = heap.pop() {
            arena
                .get_adjacent(&link.coord)
                .into_iter()
                // Filter out any points that are occupied by a player:
                .filter(|coord| coord != current_pos && !players.contains_key(&coord))
                .map(|coord| Link {
                    steps: link.steps + 1,
                    coord,
                })
                .for_each(|next_link| {
                    if let Some(Some(existing_link)) = pathfinder.get(&next_link.coord) {
                        if link.steps < existing_link.steps
                            || (link.steps == existing_link.steps
                                && link.coord < existing_link.coord)
                        {
                            // only insert if new link is using fewer steps, or if steps are equal
                            // and the new coord has a lower reading order:

                            pathfinder.insert(next_link.coord.clone(), Some(link.clone()));
                            heap.push(Reverse(next_link));
                        }
                    } else {
                        // the link doesn't already exist
                        pathfinder.insert(next_link.coord.clone(), Some(link.clone()));
                        heap.push(Reverse(next_link));
                    }
                });
        }

        // Get a list of all coords where we can attack the opponent. These will be our targets.
        let attack_coords = players
            .iter()
            .filter_map(|(coord, player)| {
                if self.is_opponent(&player) {
                    Some(coord)
                } else {
                    None
                }
            })
            .flat_map(|coord| arena.get_adjacent(coord))
            .filter(|attack_coord| {
                // filter our attack_coords for locations that aren't occupied by opponents, and
                // which are accessible via our path-finding algo:

                !players.contains_key(attack_coord) && pathfinder.contains_key(attack_coord)
            })
            .collect::<Vec<Coordinate>>();

        if attack_coords.len() == 0 {
            // println!("no targets found!");
            return PlayerAction::Stay;
        }

        // println!("map: {:?}", pathfinder);
        // println!("attack_coords: {:?}", attack_coords);
        if attack_coords.contains(&current_pos) {
            // We are already at a location where we can attack an opponent.
            // TODO: return the coord of the opponent that we want to attack.
            // Do this by: Getting our adjacent coords, filter for opponents, and return the min.
            // println!("attacking!");
            return PlayerAction::Attack(current_pos.clone());
        }

        // Choose the closest target, breaking ties with
        // readability order.
        // TODO: can we do this without a forloop?
        let mut target = Coordinate {
            x: std::u16::MAX,
            y: std::u16::MAX,
        };
        let mut steps = std::u16::MAX;
        for curr_target in attack_coords {
            if let Some(Some(link)) = pathfinder.get(&curr_target) {
                if link.steps < steps || (link.steps == steps && curr_target < target) {
                    target = curr_target;
                    steps = link.steps;
                }
            }
        }

        // Unwrap the path to the target, and return the first step to take towards the chosen
        // opponent.

        // println!("target: {:?}", target);
        while let Some(Some(link)) = pathfinder.get(&target) {
            if link.steps == 0 {
                // We have reached the link of our starting point, thus the target is only 1 step
                // away.

                break;
            }
            target = link.coord.clone();
        }

        PlayerAction::Move(target)
    }
}

#[derive(Eq, PartialEq)]
enum Cell {
    Space,
    Wall,
}

struct Arena {
    grid: Vec<Vec<Cell>>,
}

impl Arena {
    // Returns the adjacent cells for a coord which are not Walls.
    // Panics if the coord is at the boundary of a grid (which must all be Walls)

    fn get_adjacent(&self, coord: &Coordinate) -> Vec<Coordinate> {
        let (x, y) = (coord.x as usize, coord.y as usize);
        if y == 0 || self.grid.len() - 1 <= y || x == 0 || x >= self.grid[y].len() {
            panic!("cannot get_adjacent, coord out of bounds: {:?}", coord)
        }

        vec![
            Coordinate {
                x: coord.x,
                y: coord.y - 1,
            },
            Coordinate {
                x: coord.x,
                y: coord.y + 1,
            },
            Coordinate {
                x: coord.x - 1,
                y: coord.y,
            },
            Coordinate {
                x: coord.x + 1,
                y: coord.y,
            },
        ]
        .into_iter()
        .filter(|c| self.grid[c.y as usize][c.x as usize] == Cell::Space)
        .collect()
    }
}

struct Simulation {
    players: BTreeMap<Coordinate, Player>,
    arena: Arena,
}

impl Simulation {
    fn tick(&mut self) {
        // traverse each of the player's coordinates in reading order, and have each player take a step

        // TODO: implement player attacks

        self.players
            .keys()
            .cloned()
            .collect::<Vec<Coordinate>>()
            .into_iter()
            .for_each(|player_coord| {
                let player = self.players.remove(&player_coord).unwrap();
                use PlayerAction::*;
                match player.step(&player_coord, &self.arena, &self.players) {
                    Move(next_coord) => {
                        self.players.insert(next_coord, player);
                    }
                    Stay => {
                        self.players.insert(player_coord, player);
                    }
                    Attack(coord) => {
                        self.players.insert(player_coord, player);
                        // TODO: attack the opponent at the provided coord
                    }
                };
                println!();
            });
    }

    fn run(&mut self) {
        // run self.tick until one team has won!
    }
}

impl FromStr for Simulation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut players = BTreeMap::<Coordinate, Player>::new();

        let grid = s
            .lines()
            .enumerate()
            .map(|(y, line)| {
                line.as_bytes()
                    .into_iter()
                    .enumerate()
                    .map(|(x, c)| {
                        use Cell::*;
                        match c {
                            b'.' => Ok(Space),
                            b'#' => Ok(Wall),
                            b'E' => {
                                players.insert(
                                    Coordinate {
                                        y: y as u16,
                                        x: x as u16,
                                    },
                                    Player::new(PlayerKind::Elf),
                                );
                                Ok(Space)
                            }
                            b'G' => {
                                players.insert(
                                    Coordinate {
                                        y: y as u16,
                                        x: x as u16,
                                    },
                                    Player::new(PlayerKind::Goblin),
                                );
                                Ok(Space)
                            }
                            _ => Err(Error::from(format!("invalid character: {}", c))),
                        }
                    })
                    .collect::<Result<Vec<Cell>>>()
            })
            .collect::<Result<Vec<Vec<Cell>>>>()?;

        Ok(Simulation {
            players,
            arena: Arena { grid },
        })
    }
}

impl Display for Simulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.arena
            .grid
            .iter()
            .enumerate()
            .map(move |(y, v)| {
                v.iter().enumerate().map(move |(x, c)| {
                    use Cell::*;
                    let coord = Coordinate {
                        x: x as u16,
                        y: y as u16,
                    };
                    if let Some(player) = self.players.get(&coord) {
                        player.to_char()
                    } else {
                        match c {
                            Space => '.',
                            Wall => '#',
                        }
                    }
                })
            })
            .map(|s| writeln!(f, "{}", s.collect::<String>()))
            .collect::<std::fmt::Result>()
    }
}

#[test]
fn test_player_step() -> Result<()> {
    let s = "\
        #######\n\
        #.E...#\n\
        #.....#\n\
        #...G.#\n\
        #######\n\
    ";

    let mut sim = s.parse::<Simulation>()?;

    let current = Coordinate { x: 2, y: 1 };
    let player = sim.players.remove(&current).unwrap();
    assert_eq!(
        player.step(&current, &sim.arena, &sim.players),
        PlayerAction::Move(Coordinate { x: 3, y: 1 })
    );
    println!("test_player_step passed!");
    Ok(())
}

#[test]
fn test_ticks() -> Result<()> {
    let round_1 = "\
        #########\n\
        #G..G..G#\n\
        #.......#\n\
        #.......#\n\
        #G..E..G#\n\
        #.......#\n\
        #.......#\n\
        #G..G..G#\n\
        #########\n\
    ";
    let mut sim = round_1.parse::<Simulation>()?;
    println!("round 1:\n{}", sim);
    sim.tick();
    let round_2 = "\
        #########\n\
        #.G...G.#\n\
        #...G...#\n\
        #...E..G#\n\
        #.G.....#\n\
        #.......#\n\
        #G..G..G#\n\
        #.......#\n\
        #########\n\
    ";
    println!("round 2:\n{}", sim);
    assert_eq!(format!("{}", sim), round_2);

    let round_3 = "\
        #########\n\
        #..G.G..#\n\
        #...G...#\n\
        #.G.E.G.#\n\
        #.......#\n\
        #G..G..G#\n\
        #.......#\n\
        #.......#\n\
        #########\n\
    ";

    sim.tick();
    println!("round 3:\n{}", sim);
    assert_eq!(format!("{}", sim), round_3);

    let round_4 = "\
        #########\n\
        #.......#\n\
        #..GGG..#\n\
        #..GEG..#\n\
        #G..G...#\n\
        #......G#\n\
        #.......#\n\
        #.......#\n\
        #########\n\
    ";

    sim.tick();
    println!("round 4:\n{}", sim);
    assert_eq!(format!("{}", sim), round_4);

    println!("test_ticks passed!");
    Ok(())
}

#[test]
fn test_min_heap() -> Result<()> {
    let mut heap = BinaryHeap::<Reverse<Link>>::new();
    let stub_coord = Coordinate { x: 1, y: 1 };
    let link1 = Link {
        steps: 5,
        coord: stub_coord.clone(),
    };
    let link2 = Link {
        steps: 1,
        coord: stub_coord.clone(),
    };
    let link3 = Link {
        steps: 3,
        coord: stub_coord.clone(),
    };
    heap.push(Reverse(link1.clone()));
    heap.push(Reverse(link2.clone()));
    heap.push(Reverse(link3.clone()));

    assert_eq!(heap.pop(), Some(Reverse(link2)));
    assert_eq!(heap.pop(), Some(Reverse(link3)));
    assert_eq!(heap.pop(), Some(Reverse(link1)));

    println!("test_reverse_heap passed!");
    Ok(())
}
