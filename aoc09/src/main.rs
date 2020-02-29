#[macro_use]
extern crate lazy_static;
use std::io::{Read, Write};
use std::str::FromStr;

use regex::Regex;
use std::collections::HashMap;

type Error = std::boxed::Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let game = input.parse::<Game>()?;

    let game_2 = Game::new(game.players.len(), game.marbles * 100);
    writeln!(
        std::io::stdout(),
        "winning score: {}",
        game.get_winning_score()
    )?;

    writeln!(
        std::io::stdout(),
        "winning score 100x: {}",
        game_2.get_winning_score()
    )?;

    Ok(())
}

type Score = u32;

struct Game {
    players: Vec<Score>,
    marbles: usize,
    circle: Circle,
}

impl Game {
    fn new(players: usize, marbles: usize) -> Self {
        Game {
            players: vec![0; players],
            marbles,
            circle: Circle::new(),
        }
    }

    fn get_winning_score(mut self) -> u32 {
        for i in 1..=self.marbles {
            let points = self.circle.turn(i as u32);
            let player_index = (i - 1) % self.players.len();
            self.players[player_index] += points;
        }
        // 8317
        *self.players.iter().max().unwrap()
    }
}

impl FromStr for Game {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?P<players>[0-9]+) players; last marble is worth (?P<marbles>[0-9]+) points"
            )
            .unwrap();
        }

        let caps = RE.captures(s).unwrap();
        let players = usize::from(caps["players"].parse::<u16>()?);
        let marbles = caps["marbles"].parse()?;
        Ok(Game::new(players, marbles))
    }
}

type MarbleId = u32;

struct Circle {
    map: HashMap<MarbleId, Marble>,
    current: MarbleId, // id of the current marble in the circle
                       // current: MarbleId, // count
}

impl Circle {
    // starts with a single marble

    fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(
            0,
            Marble {
                id: 0,
                next: 0,
                prev: 0,
            },
        );
        Circle { map, current: 0 }
    }

    // Returns a vec representing the circle of marbles. For testing only.

    fn get_vec(&self) -> Vec<MarbleId> {
        let mut curr = self.current;
        let mut vec = vec![curr];
        loop {
            curr = self.map.get(&curr).unwrap().next;
            if curr == self.current {
                break;
            }
            vec.push(curr);
        }
        vec
    }

    // Takes a turn in the game, returning the score for that turn.
    fn turn(&mut self, new: MarbleId) -> Score {
        let new_id = new;

        if new_id % 23 == 0 {
            // Remove the marble that is 7 marbles counter clockwise of the current marble.
            let remove_id = self.get_counter_clockwise(7);
            let marble_to_remove = self.map.get_mut(&remove_id).unwrap();
            self.current = marble_to_remove.next;
            self.remove_marble(&remove_id);
            new_id + remove_id
        } else {
            let prev_id = self.get_clockwise(1);
            self.insert_marble_after(prev_id, new_id);
            self.current = new_id;
            0
        }
    }

    fn get_counter_clockwise(&self, n: usize) -> MarbleId {
        let mut curr_id = self.current;
        for _ in 0..n {
            curr_id = self.map.get(&curr_id).unwrap().prev;
        }
        curr_id
    }

    fn get_clockwise(&self, n: usize) -> MarbleId {
        let mut curr_id = self.current;
        for _ in 0..n {
            curr_id = self.map.get(&curr_id).unwrap().next;
        }
        curr_id
    }

    fn insert_marble_after(&mut self, after: MarbleId, new_id: MarbleId) {
        let prev = self.map.get_mut(&after).unwrap();

        let next_id = prev.next;
        prev.next = new_id;
        let next = self.map.get_mut(&next_id).unwrap();
        next.prev = new_id;

        self.map.insert(
            new_id,
            Marble {
                id: new_id,
                prev: after,
                next: next_id,
            },
        );
    }

    fn remove_marble(&mut self, id: &MarbleId) {
        let marble_to_remove = self.map.get(&id).unwrap();
        let [prev_id, next_id] = [marble_to_remove.prev, marble_to_remove.next];

        // remove the marble:
        self.map.remove(&id);

        // update the prev/next marbles to excise the references:
        let prev = self.map.get_mut(&prev_id).unwrap();
        prev.next = next_id;
        let next = self.map.get_mut(&next_id).unwrap();
        next.prev = prev_id;
    }
}

// Alternatively, we could use a linked list. But this Marble node should be fine for our purposes

struct Marble {
    id: MarbleId, // This is also the point value of the marble
    next: MarbleId,
    prev: MarbleId,
}

#[test]
fn test_circle() -> Result<()> {
    let mut circle = Circle::new();
    for i in 1..=22 {
        circle.turn(i);
    }
    assert_eq!(
        circle.get_vec(),
        vec![22, 11, 1, 12, 6, 13, 3, 14, 7, 15, 0, 16, 8, 17, 4, 18, 9, 19, 2, 20, 10, 21, 5]
    );
    println!("1-22 test passed.");
    circle.turn(23);
    assert_eq!(
        circle.get_vec(),
        vec![19, 2, 20, 10, 21, 5, 22, 11, 1, 12, 6, 13, 3, 14, 7, 15, 0, 16, 8, 17, 4, 18]
    );
    println!("circle test passed!");
    Ok(())
}

#[test]
fn test_inputs() -> Result<()> {
    let s = "7 players; last marble is worth 25 points";
    let game = s.parse::<Game>()?;
    assert_eq!(game.get_winning_score(), 32);
    println!("passed: {}", s);

    let s = "10 players; last marble is worth 1618 points";
    let game = s.parse::<Game>()?;
    assert_eq!(game.get_winning_score(), 8317);
    println!("passed: {}", s);

    let s = "13 players; last marble is worth 7999 points";
    let game = s.parse::<Game>()?;
    assert_eq!(game.get_winning_score(), 146373);

    let s = "17 players; last marble is worth 1104 points";
    let game = s.parse::<Game>()?;
    assert_eq!(game.get_winning_score(), 2764);
    println!("passed: {}", s);

    let s = "21 players; last marble is worth 6111 points";
    let game = s.parse::<Game>()?;
    assert_eq!(game.get_winning_score(), 54718);

    let s = "30 players; last marble is worth 5807 points";
    let game = s.parse::<Game>()?;
    assert_eq!(game.get_winning_score(), 37305);

    println!("tests passed!");
    Ok(())
}
