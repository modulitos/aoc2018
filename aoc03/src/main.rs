#[macro_use]
extern crate lazy_static;
use regex::Regex;
use std::io::{self, Read, Write};
use std::str::FromStr;

type Error = Box<dyn ::std::error::Error>;
type Result<T> = ::std::result::Result<T, Error>;

const GRID_SIZE: usize = 1000;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    let claims = claims_from_str(&input)?;

    // TODO: Program hangs if we use u32. Why??
    let mut grid = [[u8::from(0); GRID_SIZE]; GRID_SIZE];

    writeln!(
        io::stdout(),
        "overlaps: {}",
        count_overlaps(&claims, &mut grid)?
    )?;

    writeln!(
        io::stdout(),
        "non-overlapping claim: {}",
        get_non_overlapping(&claims, &grid)?
    )?;
    Ok(())
}

fn claims_from_str(input: &str) -> Result<Vec<Claim>> {
    let mut claims: Vec<Claim> = Vec::new();
    for line in input.lines() {
        claims.push(Claim::from_str(line)?);
    }
    Ok(claims)
}

fn count_overlaps(claims: &Vec<Claim>, grid: &mut [[u8; 1000]; 1000]) -> Result<i32> {
    claims.iter().for_each(|c| {
        c.iter_points().for_each(|(x, y)| {
            // TODO: usize doesn't have try_from on a u8. How to avoid type casting here?

            grid[x as usize][y as usize] += 1;
        })
    });

    // TODO: How to avoid the for loops here? Perhaps we would need to bring in
    // https://crates.io/crates/ndarray ?

    let mut counts = 0;
    for i in 0..GRID_SIZE {
        for j in 0..GRID_SIZE {
            if grid[i][j] > 1 {
                counts += 1;
            }
        }
    }

    Ok(counts)
}

fn get_non_overlapping(claims: &Vec<Claim>, grid: &[[u8; 1000]; 1000]) -> Result<u32> {
    Ok(claims
        .iter()
        .find(|claim| {
            claim
                .iter_points()
                .all(|(x, y)| grid[x as usize][y as usize] < 2)
        })
        .unwrap()
        .id)
}

struct Claim {
    id: u32,
    x: u32,
    y: u32,
    dx: u32,
    dy: u32,
}

impl Claim {
    fn iter_points(&self) -> IterPoints {
        IterPoints {
            claim: &self,
            px: self.x,
            py: self.y,
        }
    }
}

struct IterPoints<'c> {
    claim: &'c Claim,
    px: u32,
    py: u32,
}

impl<'c> Iterator for IterPoints<'c> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.px >= self.claim.x + self.claim.dx {
            // We need to increment y
            self.py += 1;
            self.px = self.claim.x;
        }

        if self.py >= self.claim.y + self.claim.dy {
            // y has exceeded the bounds
            return None;
        }

        let (px, py) = (self.px, self.py);
        self.px += 1;
        Some((px, py))
    }
}

#[test]
fn test_claim_iterator() -> Result<()> {
    let claim = Claim {
        id: 0,
        x: 4,
        y: 0,
        dx: 2,
        dy: 2,
    };
    let mut iter = claim.iter_points();
    assert_eq!(iter.next(), Some((4, 0)));
    assert_eq!(iter.next(), Some((5, 0)));
    assert_eq!(iter.next(), Some((4, 1)));
    assert_eq!(iter.next(), Some((5, 1)));
    assert_eq!(iter.next(), None);

    println!("itereator passes!");
    Ok(())
}

impl FromStr for Claim {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
            \#(?P<id>[0-9]+)\s+@\s+  # the id
            (?P<x>[0-9]+),(?P<y>[0-9]+):\s+  # the x,y offset from the top left
            (?P<dx>[0-9]+)x(?P<dy>[0-9]+)  # the distance in the x and y dimensions
            "
            )
            .unwrap();
        }
        // TODO: avoid unwrap and throw a specific error. Might require updating our error type.
        let caps = RE.captures(s).unwrap();

        let claim = Claim {
            id: caps["id"].parse()?,
            x: caps["x"].parse()?,
            y: caps["y"].parse()?,
            dx: caps["dx"].parse()?,
            dy: caps["dy"].parse()?,
        };
        Ok(claim)
    }
}

#[test]
fn test_overlaps() -> Result<()> {
    let s = "#1 @ 1,3: 4x4\n#2 @ 3,1: 4x4\n#3 @ 5,5: 2x2\n";

    let claims: Vec<Claim> = claims_from_str(&s)?;
    assert_eq!(claims[0].id, 1);

    // TODO: Program hangs if we use u32. Why??
    let mut grid = [[u8::from(0); GRID_SIZE]; GRID_SIZE];

    assert_eq!(count_overlaps(&claims, &mut grid)?, 4);

    assert_eq!(get_non_overlapping(&claims, &grid)?, 3);

    println!("overlaps passed!");
    Ok(())
}
