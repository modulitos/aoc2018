use std::boxed::Box;
use std::collections::{BTreeMap, HashMap};
use std::error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::result;
use std::str::FromStr;

type Error = Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

const START_PERIOD_AT_MINUTE: usize = 1_000;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let mut sim = input.parse::<Simulation>()?;

    // run until 1_000_000_000 minutes...
    let mut period_values = vec![];
    // assuming that a pattern emerges between 1_000 and 2_000 iterations...
    for i in 0..(2_000) {
        // part 1: get resource values after 10 mins:
        if i == 10 {
            writeln!(
                std::io::stdout(),
                "sim resource value after 10 minutes: {}",
                sim.get_resource_value()
            )?;
        }

        // part 2: leverage a repeating pattern to get resource value after 1_000_000_000 minutes
        if i >= START_PERIOD_AT_MINUTE {
            let resource_value = sim.get_resource_value();
            if period_values.contains(&resource_value) {
                break;
            }

            period_values.push(resource_value);
        }

        sim.run_minute();
   }

    let period_length = period_values.len();
    let offset = (1_000_000_000 - START_PERIOD_AT_MINUTE) % period_length;

    writeln!(
        std::io::stdout(),
        "sim resource value after 1_000_000_000 minutes: {}",
        period_values.get(offset).unwrap()
    )?;

    Ok(())
}

#[derive(Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
struct Coordinate {
    y: usize,
    x: usize,
}

#[derive(Debug)]
struct Player {

    // Leveraging trait objects using the state pattern here. Using enums would've been more
    // concise, but I wanted to try out trait objects as a learning experience!

    kind: Box<dyn State>,
}

impl Player {
    fn from_byte(b: &u8) -> Result<Self> {
        // TODO: Can we DRY this up?
        Ok(match b {
            b'#' => Self {
                kind: Box::new(Lumberyard {}),
            },
            b'.' => Self {
                kind: Box::new(OpenGround {}),
            },
            b'|' => Self {
                kind: Box::new(Trees {}),
            },
            _ => {
                return Err(Error::from(format!(
                    "Player::from_byte: invalid byte: {}",
                    b
                )))
            }
        })
    }

    fn transition_from_neighbors(&self, neighbors: Vec<&Player>) -> Self {
        let neighbors = neighbors
            .iter()
            .map(|player| &player.kind)
            .collect::<Vec<&Box<dyn State>>>();

        Self {
            kind: self.kind.transition_from_neighbors(neighbors),
        }
    }
}

struct OpenGround {}
struct Trees {}
struct Lumberyard {}
trait State {
    fn to_char(&self) -> char;

    // Update the player based on the state of our neighbors
    fn transition_from_neighbors(&self, neighbors: Vec<&Box<dyn State>>) -> Box<dyn State>;

    fn is_openground(&self) -> bool {
        self.to_char() == '.'
    }

    fn is_trees(&self) -> bool {
        self.to_char() == '|'
    }

    fn is_lumberyard(&self) -> bool {
        self.to_char() == '#'
    }
}

impl std::fmt::Debug for dyn State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "'{}'", self.to_char())
    }
}

impl State for OpenGround {
    fn to_char(&self) -> char {
        '.'
    }

    fn transition_from_neighbors(&self, neighbors: Vec<&Box<dyn State>>) -> Box<dyn State> {
        let count_trees = neighbors.iter().filter(|state| state.is_trees()).count();
        if count_trees >= 3 {
            Box::new(Trees {})
        } else {
            Box::new(Self {})
        }
    }
}

impl State for Trees {
    fn to_char(&self) -> char {
        '|'
    }

    fn transition_from_neighbors(&self, neighbors: Vec<&Box<dyn State>>) -> Box<dyn State> {
        let count_lumberyards = neighbors
            .iter()
            .filter(|state| state.is_lumberyard())
            .count();
        if count_lumberyards >= 3 {
            Box::new(Lumberyard {})
        } else {
            Box::new(Self {})
        }
    }
}

impl State for Lumberyard {
    fn to_char(&self) -> char {
        '#'
    }

    fn transition_from_neighbors(&self, neighbors: Vec<&Box<dyn State>>) -> Box<dyn State> {
        let count_lumberyards = neighbors
            .iter()
            .filter(|state| state.is_lumberyard())
            .count();
        let count_trees = neighbors.iter().filter(|state| state.is_trees()).count();
        if count_lumberyards >= 1 && count_trees >= 1 {
            Box::new(Self {})
        } else {
            Box::new(OpenGround {})
        }
    }
}

struct Simulation {
    players: HashMap<Coordinate, Player>,
}

impl Simulation {
    fn run_minute(&mut self) {
        let mut next_players = HashMap::new();

        self.players.iter().for_each(|(coord, player)| {
            let neighbors = self.get_neighbors(coord);
            let new_player = player.transition_from_neighbors(neighbors);
            next_players.insert(coord.clone(), new_player);
        });

        self.players = next_players;
    }

    fn get_neighbors(&self, coord: &Coordinate) -> Vec<&Player> {
        (coord.y.saturating_sub(1)..=coord.y + 1)
            .flat_map(|y| {
                (coord.x.saturating_sub(1)..=coord.x + 1).filter_map(move |x| {
                    let adjacent_coord = Coordinate { x, y };
                    match self.players.get(&adjacent_coord) {
                        Some(player) if coord != &adjacent_coord => Some(player),
                        _ => None,
                    }
                })
            })
            .collect()
    }

    fn get_resource_value(&self) -> usize {
        let count_wooded_acres = self
            .players
            .values()
            .filter(|player| player.kind.is_trees())
            .count();

        let count_lumberyards = self
            .players
            .values()
            .filter(|player| player.kind.is_lumberyard())
            .count();

        count_lumberyards * count_wooded_acres
    }
}

impl FromStr for Simulation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut row_length = None; // ensure each row has the same length
        let players = s
            .lines()
            .enumerate()
            // TODO: how to flat_map when the closure returns a Result<Iterator<_>> ?
            .map(|(y, line)| {
                let row = line
                    .as_bytes()
                    .into_iter()
                    .enumerate()
                    .map(|(x, b)| Ok((Coordinate { x, y }, Player::from_byte(b)?)))
                    .collect::<Result<Vec<(Coordinate, Player)>>>()?;

                // Verify all rows have equal length:
                match row_length {
                    Some(length) if row.len() != length => Err(Self::Err::from(format!(
                        "invalid row lengths, row {} not equal to another row length: {}",
                        row.len(),
                        row_length.unwrap()
                    ))),
                    _ => {
                        row_length = Some(row.len());
                        Ok(row)
                    }
                }
            })
            .collect::<Result<Vec<Vec<(Coordinate, Player)>>>>()?
            .into_iter()
            .fold(HashMap::new(), |mut map, row| {
                row.into_iter().for_each(|(coord, player)| {
                    map.insert(coord, player);
                });
                map
            });

        Ok(Self { players })
    }
}

impl Display for Simulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let max = self.players.keys().max().unwrap();
        (0..=max.y)
            .map(|y| {
                (0..=max.x)
                    .map(|x| {
                        write!(
                            f,
                            "{}",
                            self.players
                                .get(&Coordinate { x, y })
                                .unwrap()
                                .kind
                                .to_char()
                        )
                    })
                    .collect::<Result<(), _>>()?;
                writeln!(f, "") // newline at end of row
            })
            .collect::<Result<(), _>>()
    }
}

#[test]
fn test_simulation() -> Result<()> {
    let input = "\
        .#.#...|#.\n\
        .....#|##|\n\
        .|..|...#.\n\
        ..|#.....#\n\
        #.#|||#|#|\n\
        ...#.||...\n\
        .|....|...\n\
        ||...#|.#|\n\
        |.||||..|.\n\
        ...#.|..|.\n\
    ";

    let mut sim = input.parse::<Simulation>()?;
    println!("sim init:\n{}", sim);
    assert_eq!(format!("{}", sim), input);

    assert_eq!(
        sim.get_neighbors(&Coordinate { x: 7, y: 0 }).iter().count(),
        5
    );
    assert_eq!(
        sim.get_neighbors(&Coordinate { x: 7, y: 0 })
            .iter()
            .map(|player| player.kind.to_char())
            .collect::<Vec<char>>(),
        vec!['.', '#', '|', '#', '#']
    );

    let minute_1 = "\
        .......##.\n\
        ......|###\n\
        .|..|...#.\n\
        ..|#||...#\n\
        ..##||.|#|\n\
        ...#||||..\n\
        ||...|||..\n\
        |||||.||.|\n\
        ||||||||||\n\
        ....||..|.\n\
    ";
    sim.run_minute();

    assert_eq!(format!("{}", sim), minute_1);

    (0..9).for_each(|_| sim.run_minute());

    let minute_10 = "\
        .||##.....\n\
        ||###.....\n\
        ||##......\n\
        |##.....##\n\
        |##.....##\n\
        |##....##|\n\
        ||##.####|\n\
        ||#####|||\n\
        ||||#|||||\n\
        ||||||||||\n\
    ";

    assert_eq!(format!("{}", sim), minute_10);
    assert_eq!(sim.get_resource_value(), 1147);

    Ok(())
}
