use std::boxed::Box;
use std::cmp::{Ordering, Reverse};
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};
use std::error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::result;
use std::str::FromStr;

type Error = Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let mut sim = input.parse::<Simulation>()?;
    writeln!(std::io::stdout(), "result of simulation: {:?}", sim.run())?;

    writeln!(std::io::stdout(), "result of elf power: {:?}", Simulation::find_elf_power(&input)?)?;
    Ok(())
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Debug)]
struct Coordinate {
    y: u16,
    x: u16,
}

#[derive(PartialEq, Hash, Eq)]
enum PlayerKind {
    Elf,
    Goblin,
}

#[derive(Debug, Eq, PartialEq)]
enum PlayerAction {
    Stay,               // No accessible enemies to attack, so stay in place
    Move(Coordinate),   // Move towards an opponent
    Attack(Coordinate), // Attacks the player at the given coordinate
    // Moves next to the opponent (first coord), and attacks the player at the second coordinate
    MoveAndAttack(Coordinate, Coordinate),
}

type PlayerId = u8;

struct Player {
    id: PlayerId,
    health: u16,
    power: u16, // damage done for each attack
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

struct PathFinder {
    map: HashMap<Coordinate, Option<Link>>,
}

impl PathFinder {
    // Use Dijkstra's algorithm to track each accessible cell in the arena, and map how many
    // steps it takes to get there, along with a link to the existing point to get to that point

    // If the map's value is None, then there are no steps required to get to that coord.
    // let mut pathfinder = HashMap::<Coordinate, Option<Link>>::new();
    // pathfinder.insert(current_pos.clone(), None);

    fn new(
        current_pos: &Coordinate,
        arena: &Arena,
        players: &BTreeMap<Coordinate, Player>,
    ) -> Self {
        let mut map = HashMap::<Coordinate, Option<Link>>::new();
        map.insert(current_pos.clone(), None);

        let mut heap = BinaryHeap::<Reverse<Link>>::new();
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
                    if let Some(Some(existing_link)) = map.get(&next_link.coord) {
                        if link.steps < existing_link.steps
                            || (link.steps == existing_link.steps
                                && link.coord < existing_link.coord)
                        {
                            // only insert if new link is using fewer steps, or if steps are equal
                            // and the new coord has a lower reading order:

                            map.insert(next_link.coord.clone(), Some(link.clone()));
                            heap.push(Reverse(next_link));
                        }
                    } else {
                        // the link doesn't already exist
                        map.insert(next_link.coord.clone(), Some(link.clone()));
                        heap.push(Reverse(next_link));
                    }
                });
        }
        PathFinder { map }
    }

    fn get_first_step_toward_target(&self, mut target: Coordinate) -> Coordinate {
        while let Some(Some(link)) = self.map.get(&target) {
            if link.steps == 0 {
                // We have reached the link of our starting point, thus the target is only 1 step
                // away.

                break;
            }
            target = link.coord.clone()
        }
        target
    }
}

impl Player {
    fn new(kind: PlayerKind, id: PlayerId) -> Self {
        Player {
            health: 200,
            kind,
            power: 3,
            id,
        }
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

    fn step<'a>(
        &self,
        current_pos: &Coordinate,
        arena: &Arena,
        players: &BTreeMap<Coordinate, Player>, // list of all players, except this player
    ) -> PlayerAction {
        let pathfinder = PathFinder::new(current_pos, arena, players);

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

                !players.contains_key(attack_coord) && pathfinder.map.contains_key(attack_coord)
            })
            .collect::<Vec<Coordinate>>();

        if attack_coords.len() == 0 {
            return PlayerAction::Stay;
        }

        if attack_coords.contains(&current_pos) {
            let target = self.get_opponent_target_from_attack_range(arena, players, &current_pos);
            return PlayerAction::Attack(target);
        }

        // Choose the closest target, breaking ties with
        // readability order.
        let target = attack_coords.iter().filter(|coord| {
            pathfinder.map.get(&coord).is_some()
        }).min_by_key(|coord| {
            if let Some(Some(link)) = pathfinder.map.get(coord) {
                (link.steps, coord.clone())
            } else {
                // TODO: how can we avoid the if/let here, and unwrap/expect directly?
                panic!("player's coord should not be within attacking range at this point.")
            }
        }).unwrap().clone();


        // Unwrap the path to the target, and return the first step to take towards the chosen
        // opponent.

        let target = pathfinder.get_first_step_toward_target(target);

        if attack_coords.contains(&target) {
            let opponent_target =
                self.get_opponent_target_from_attack_range(arena, players, &target);
            return PlayerAction::MoveAndAttack(target, opponent_target);
        } else {
            PlayerAction::Move(target)
        }
    }

    // Given a location where we can attack an opponent, find our opponent's coords by getting our
    // adjacent coords, filtering for opponent occupied coords, and return the coord of the opponent
    // with min health. Break ties with coord in reading order.

    fn get_opponent_target_from_attack_range(
        &self,
        arena: &Arena,
        players: &BTreeMap<Coordinate, Player>,
        cell_in_range: &Coordinate,
    ) -> Coordinate {
        arena
            .get_adjacent(cell_in_range) // variable here!
            .into_iter()
            .filter(|coord| {
                if let Some(player) = players.get(&coord) {
                    self.is_opponent(player)
                } else {
                    false
                }
            })
            .min_by_key(|coord| {
                // select the opponent with the lowest health, breaking ties by reading order
                let player = players.get(&coord).unwrap();
                (player.health, coord.clone())
            })
            .expect("invalid state: attack_coords misaligned with players")
    }

    // Attack an opposing player. Returns whether or not the opponent dies.

    fn attack(&self, opponent: &mut Player) -> bool {
        if !self.is_opponent(opponent) {
            panic!("attempting to attack own opponent!");
        }
        opponent.health = opponent.health.saturating_sub(self.power);
        opponent.health == 0
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
    rounds: u16, // number of rounds that have passed
}

impl Simulation {
    fn tick(&mut self) {
        // traverse each of the player's coordinates in reading order, and have each player take a
        // step

        // assign "round_completed" to indicate whether all players cycled through with opponents on
        // the other team still available.

        let round_completed = self
            .players
            .values()
            .map(|player| player.id)
            .clone()
            .collect::<Vec<PlayerId>>()
            .into_iter()
            .map(|id| {
                // returns a bool indicating whether the round exited early.

                // Iterating over id's instead of coords here to cover the edge case where a player
                // moves into another player's coordinate

                if self
                    .players
                    .values()
                    .find(|player| player.id == id)
                    .is_none()
                {
                    // the player has since died this round, so skip them
                    return false;
                }

                if self
                    .players
                    .values()
                    .map(|player| &player.kind)
                    .collect::<HashSet<&PlayerKind>>()
                    .len()
                    <= 1
                {
                    // If all opponents have been eliminated, then exit the round early.
                    return true;
                }
                let player_coord = self
                    .players
                    .iter()
                    .find(|(_, player)| player.id == id)
                    .unwrap()
                    .0
                    .clone();
                let player = self.players.remove(&player_coord).expect(&format!(
                    "player not found in BTreeMap at coord: {:?}, on round: {:?}",
                    player_coord, self.rounds
                ));
                match player.step(&player_coord, &self.arena, &self.players) {
                    PlayerAction::Move(next_coord) => {
                        self.players.insert(next_coord, player);
                    }
                    PlayerAction::Stay => {
                        self.players.insert(player_coord.clone(), player);
                    }
                    PlayerAction::MoveAndAttack(player_coord, target) => {
                        if let Entry::Occupied(mut opponent) = self.players.entry(target.clone()) {
                            if player.attack(opponent.get_mut()) {
                                self.players.remove(&target.clone());
                            }
                        } else {
                            panic!("opponent not found when attacking target: {:?}", target);
                        };
                        self.players.insert(player_coord.clone(), player);
                    }
                    PlayerAction::Attack(target) => {
                        // TODO: is there a way to write in this form, without the "closure required
                        // unique access to self, but it is already borrowed" error?

                        // self.players.entry(target.clone()).and_modify(|opponent| {
                        //     if player.attack(opponent) {
                        //         self.players.remove(&target);
                        //     }
                        // });

                        if let Entry::Occupied(mut opponent) = self.players.entry(target.clone()) {
                            if player.attack(opponent.get_mut()) {
                                self.players.remove(&target.clone());
                            }
                        } else {
                            panic!("opponent not found when attacking target: {:?}", target);
                        };
                        self.players.insert(player_coord.clone(), player);
                    }
                };
                false
            })
            .find(|&result| result)
            .is_none();
        if round_completed {
            self.rounds += 1;
        }
    }

    fn run(&mut self) -> u32 {
        // run self.tick until one team has won!
        while self
            .players
            .values()
            .map(|player| &player.kind)
            .collect::<HashSet<&PlayerKind>>()
            .len()
            > 1
        {
            self.tick();
        }
        u32::from(
            self.players
                .values()
                .map(|player| player.health)
                .sum::<u16>(),
        ) * u32::from(self.rounds)
    }

    fn set_elf_power(&mut self, power: u16) {
        self.players
            .values_mut()
            .filter(|player| player.kind == PlayerKind::Elf)
            .for_each(|elf| elf.power = power);
    }

    fn get_elf_counts(&self) -> u8 {
        self.players
            .values()
            .filter(|player| player.kind == PlayerKind::Elf)
            .count() as u8
    }

    // runs the simulation over and over until we find the minimum elf power required to defeat all
    // Goblins without losing a single elf.

    fn find_elf_power(input: &str) -> Result<u32> {
        for power in 4..200 {  // power of 4 is the minimum
            let mut sim = input.parse::<Simulation>()?;
            sim.set_elf_power(power);
            let starting_elves = sim.get_elf_counts();
            let result = sim.run();
            if starting_elves == sim.get_elf_counts() {
                return Ok(result);
            }
        }
        Err(Error::from(
            "failed to find elf power after 200 iterations.",
        ))
    }

    // for debugging/testing only
    fn get_player_healths(&self) -> Vec<u16> {
        self.players
            .values()
            .map(|player| player.health)
            .collect::<Vec<u16>>()
    }
}

impl FromStr for Simulation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut players = BTreeMap::<Coordinate, Player>::new();
        let mut curr_id = 0;

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
                                    Player::new(PlayerKind::Elf, curr_id),
                                );
                                curr_id += 1;
                                Ok(Space)
                            }
                            b'G' => {
                                players.insert(
                                    Coordinate {
                                        y: y as u16,
                                        x: x as u16,
                                    },
                                    Player::new(PlayerKind::Goblin, curr_id),
                                );
                                curr_id += 1;
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
            rounds: 0,
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
    println!("test_player_step passed.");
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
    assert_eq!(format!("{}", sim), round_4);

    println!("test_ticks passed.");
    Ok(())
}

#[test]
fn test_attacks() -> Result<()> {
    let round_0 = "\
        #######\n\
        #.G...#\n\
        #...EG#\n\
        #.#.#G#\n\
        #..G#E#\n\
        #.....#\n\
        #######\n\
    ";

    let mut sim = round_0.parse::<Simulation>()?;
    sim.tick();
    let round_1 = "\
        #######\n\
        #..G..#\n\
        #...EG#\n\
        #.#G#G#\n\
        #...#E#\n\
        #.....#\n\
        #######\n\
    ";
    assert_eq!(format!("{}", sim), round_1);
    assert_eq!(
        sim.players
            .values()
            .map(|player| player.health)
            .collect::<Vec<u16>>(),
        vec![200, 197, 197, 200, 197, 197]
    );

    sim.tick();
    let round_2 = "\
        #######\n\
        #...G.#\n\
        #..GEG#\n\
        #.#.#G#\n\
        #...#E#\n\
        #.....#\n\
        #######\n\
    ";
    assert_eq!(format!("{}", sim), round_2);
    assert_eq!(
        sim.players
            .values()
            .map(|player| player.health)
            .collect::<Vec<u16>>(),
        vec![200, 200, 188, 194, 194, 194]
    );

    (0..21).for_each(|_| sim.tick());

    let round_23 = "\
        #######\n\
        #...G.#\n\
        #..G.G#\n\
        #.#.#G#\n\
        #...#E#\n\
        #.....#\n\
        #######\n\
    ";
    assert_eq!(format!("{}", sim), round_23);
    assert_eq!(
        sim.players
            .values()
            .map(|player| player.health)
            .collect::<Vec<u16>>(),
        vec![200, 200, 131, 131, 131]
    );

    let round_47 = "\
        #######\n\
        #G....#\n\
        #.G...#\n\
        #.#.#G#\n\
        #...#.#\n\
        #....G#\n\
        #######\n\
    ";
    let result = sim.run();
    assert_eq!(format!("{}", sim), round_47);
    assert_eq!(sim.get_player_healths(), vec![200, 131, 59, 200]);
    assert_eq!(result, 27730);

    println!("test_attacks passed.");
    Ok(())
}

#[test]
fn test_run_simulation_1() -> Result<()> {
    let input = "\
        #######\n\
        #G..#E#\n\
        #E#E.E#\n\
        #G.##.#\n\
        #...#E#\n\
        #...E.#\n\
        #######\n\
    ";
    let mut sim = input.parse::<Simulation>()?;
    let end = "\
        #######\n\
        #...#E#\n\
        #E#...#\n\
        #.E##.#\n\
        #E..#E#\n\
        #.....#\n\
        #######\n\
    ";
    let result = sim.run();
    assert_eq!(format!("{}", sim), end);
    assert_eq!(sim.rounds, 37);
    assert_eq!(sim.get_player_healths(), vec![200, 197, 185, 200, 200]);
    assert_eq!(result, 36334);

    println!("test_run_simulation_1 passed.");
    Ok(())
}

#[test]
fn test_simulation_2() -> Result<()> {
    let input = "\
        #######\n\
        #E..EG#\n\
        #.#G.E#\n\
        #E.##E#\n\
        #G..#.#\n\
        #..E#.#\n\
        #######\n\
    ";
    let end = "\
        #######\n\
        #.E.E.#\n\
        #.#E..#\n\
        #E.##.#\n\
        #.E.#.#\n\
        #...#.#\n\
        #######\n\
    ";

    let mut sim = input.parse::<Simulation>()?;
    let result = sim.run();
    assert_eq!(format!("{}", sim), end);
    assert_eq!(sim.get_player_healths(), vec![164, 197, 200, 98, 200]); // [65, 200, 101, 98, 200, 200]
    assert_eq!(sim.rounds, 46); // actual: 45
    assert_eq!(sim.get_player_healths().iter().sum::<u16>(), 859); // 864
    assert_eq!(result, 39514);

    // Find min elf power:
    assert_eq!(Simulation::find_elf_power(&input)?, 31284);

    println!("test_simulation_2 passed.");
    Ok(())
}

#[test]
fn test_run_simulation_3() -> Result<()> {
    let input = "\
        #######\n\
        #E.G#.#\n\
        #.#G..#\n\
        #G.#.G#\n\
        #G..#.#\n\
        #...E.#\n\
        #######\n\
    ";
    let mut sim = input.parse::<Simulation>()?;
    let result = sim.run();
    assert_eq!(sim.rounds, 35);
    assert_eq!(sim.get_player_healths(), vec![200, 98, 200, 95, 200]);
    assert_eq!(result, 27755);

    // Find min elf power:
    assert_eq!(Simulation::find_elf_power(&input)?, 3478);

    println!("test_run_simulation_3 passed.");
    Ok(())
}

#[test]
fn test_run_simulation_4() -> Result<()> {
    let input = "\
        #######\n\
        #.E...#\n\
        #.#..G#\n\
        #.###.#\n\
        #E#G#G#\n\
        #...#G#\n\
        #######\n\
    ";

    let mut sim = input.parse::<Simulation>()?;

    let result = sim.run();
    assert_eq!(sim.rounds, 54);
    assert_eq!(sim.get_player_healths(), vec![200, 98, 38, 200]);
    assert_eq!(result, 28944);

    // Find min elf power:
    assert_eq!(Simulation::find_elf_power(&input)?, 6474);

    println!("test_run_simulation_4 passed.");
    Ok(())
}

#[test]
fn test_run_simulation_5() -> Result<()> {
    let input = "\
        #########\n\
        #G......#\n\
        #.E.#...#\n\
        #..##..G#\n\
        #...##..#\n\
        #...#...#\n\
        #.G...G.#\n\
        #.....G.#\n\
        #########\n\
    ";

    let mut sim = input.parse::<Simulation>()?;

    let result = sim.run();
    assert_eq!(sim.rounds, 20);
    assert_eq!(sim.get_player_healths(), vec![137, 200, 200, 200, 200]);
    assert_eq!(result, 18740);

    // Find min elf power:
    assert_eq!(Simulation::find_elf_power(&input)?, 1140);

    println!("test_run_simulation_5 passed.");
    Ok(())
}

#[test]
fn test_part_1() -> Result<()> {
    // testing this because it has some edge cases that weren't covered in the existing unit tests.

    let input = "\
        ################################\n\
        ###############..........#######\n\
        ######.##########G.......#######\n\
        #####..###..######...G...#######\n\
        #####..#...G..##........########\n\
        #####..G......#GG.......########\n\
        ######..G..#G.......G....#######\n\
        ########...###...#........######\n\
        ######....G###.GG#.........#####\n\
        ######G...####...#..........####\n\
        ###.##.....G................####\n\
        ###.......................#.####\n\
        ##.......G....#####.......E.####\n\
        ###.......G..#######....##E.####\n\
        ####........#########..G.#.#####\n\
        #.#..##.....#########..#..######\n\
        #....####.G.#########......#####\n\
        #.##G#####..#########.....###.E#\n\
        ###########.#########...E.E....#\n\
        ###########..#######..........##\n\
        ###########..E#####.......######\n\
        ###########............E.#######\n\
        #########.E.....E..##.#..#######\n\
        #######.G.###.......E###########\n\
        ######...#######....############\n\
        ################...#############\n\
        ###############....#############\n\
        ###############...##############\n\
        #################.##############\n\
        #################.##############\n\
        #################.##############\n\
        ################################\n\
    ";

    let mut sim = input.parse::<Simulation>()?;

    let result = sim.run();
    assert_eq!(result, 319410);

    println!("test_part_1 passed.");
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

    println!("test_reverse_heap passed.");
    Ok(())
}
