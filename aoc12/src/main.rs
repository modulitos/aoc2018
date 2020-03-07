use std::boxed;
use std::collections::HashSet;
use std::error;
use std::io::{Read, Write};
use std::result;
use std::str::FromStr;

type Error = boxed::Box<dyn error::Error>;
type Result<T, E = Error> = result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let simulation = input.parse::<Simulation>()?;
    writeln!(
        std::io::stdout(),
        "count after 20 generations: {}",
        simulation.run(20)
    )?;

    // after running this code:

    // let simulation = input.parse::<Simulation>()?;
    // writeln!(
    //     std::io::stdout(),
    //     "count after 50_000_000_000 generations: {}",
    //     simulation.run(50_000_000_000)
    // )?;

    // we get:
    // on generation: 500, sum is: 21684
    // on generation: 5000, sum is: 201684
    // on generation: 50000, sum is: 2001684
    // on generation: 500000, sum is: 20001684

    // the value is 2x1684 where x is a series of 0's.
    // for 500, x is 0
    // for 5000, x is 1
    // for 50_000, x is 2
    // for 500_000, x is 3

    // thus, if n is the number of zeros in our generation, x is n - 2.
    // for 50 B, n is 10, so x is 8.
    // thus, the result is 2000000001684

    Ok(())
}

type PotId = i64;

struct Simulation {
    pots: HashSet<PotId>, // a set of pot id's that have plants
    matches: HashSet<String>,
    generation: u64,
}

impl Simulation {
    // Run for a single generation.

    fn run_generation(&mut self) {
        self.generation += 1;

        // if there are no pots, there is nothing to do.
        if let (Some(left_most), Some(right_most)) =
            (self.pots.iter().min(), self.pots.iter().max())
        {
            // Iterate over all relevant pots, starting 2 pots down from the left-most planted pot,
            // ending 2 pots up from the right-most planted pot

            let mut next_pots = HashSet::new();
            for pot_id in (left_most - 2)..=(right_most + 2) {
                // build up a pattern of plant distributions for the current PotId:
                let pattern = ((pot_id - 2)..=(pot_id + 2))
                    .map(|pot_id| {
                        if self.pots.contains(&pot_id) {
                            '#'
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>();
                if self.matches.contains(&pattern) {
                    next_pots.insert(pot_id);
                }
            }
            self.pots = next_pots;
        }
    }

    // Run simulation for 20 generations, returning the score at the end of the 20th generation.

    fn run(mut self, generations: u64) -> i64 {
        while self.generation < generations {
            if self.generation == 500 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            if self.generation == 5000 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            if self.generation == 50_000 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            if self.generation == 500_000 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            if self.generation == 5_000_000 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            if self.generation == 50_000_000 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            if self.generation == 500_000_000 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            if self.generation == 5_000_000_000 {
                println!(
                    "on generation: {}, sum is: {}",
                    self.generation,
                    self.pots.iter().sum::<i64>()
                );
            }
            self.run_generation();
        }
        self.pots.iter().sum()
    }

    // For testing only.
    // Returns a string representing the generation

    fn generation_to_str(&self) -> String {
        // if there are no pots, return an empty string
        if let (Some(&left_most), Some(&right_most)) =
            (self.pots.iter().min(), self.pots.iter().max())
        {
            (left_most..=right_most)
                .map(|pot_id| {
                    if self.pots.contains(&pot_id) {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect::<String>()
        } else {
            "".to_string()
        }
    }
}

impl FromStr for Simulation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.lines();
        let initial_state = match iter.next() {
            None => return Err(Self::Err::from("invalid string")),
            Some(s) => s,
        };
        let prefix = "initial state: ";
        iter.next();

        let pots = initial_state[prefix.len()..]
            .as_bytes()
            .iter()
            .enumerate()
            .filter(|(i, &c)| c == b'#')
            .map(|(i, _)| i as PotId)
            .collect::<HashSet<PotId>>();

        let matches = iter
            .filter_map(|line| {
                if line.as_bytes()[9] == b'#' {
                    Some(line[0..5].to_string())
                } else {
                    None
                }
            })
            .collect::<HashSet<String>>();
        Ok(Simulation {
            pots,
            matches,
            generation: 0,
        })
    }
}

#[test]
fn test_count_plants() -> Result<()> {
    let input = "\
    initial state: #..#.#..##......###...###\n\
    \n\
    ...## => #\n\
    ..#.. => #\n\
    .#... => #\n\
    .#.#. => #\n\
    .#.## => #\n\
    .##.. => #\n\
    .#### => #\n\
    #.#.# => #\n\
    #.### => #\n\
    ##.#. => #\n\
    ##.## => #\n\
    ###.. => #\n\
    ###.# => #\n\
    ####. => #\
    ";

    let mut simulation = input.parse::<Simulation>()?;
    assert_eq!(simulation.matches.len(), 14);
    assert_eq!(simulation.generation_to_str(), "#..#.#..##......###...###");
    simulation.run_generation();
    assert_eq!(simulation.generation_to_str(), "#...#....#.....#..#..#..#");

    assert_eq!(simulation.run(20), 325);

    println!("places counted pass!");
    Ok(())
}

#[test]
fn test_str_slice() {
    assert_eq!("asdf", "asdf");
    assert_eq!("asdf"[1..3], "asdf"[1..3]);
    println!("slices equal!");
}
