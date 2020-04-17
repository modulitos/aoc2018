mod error;

use error::{Error, Result};
use std::collections::HashMap;
use std::io::{Read, Write};

fn main() -> Result<()> {
    // puzzle input:
    // depth: 3339
    // target: 10,715
    let depth = 3_339;
    let target = Coordinate { x: 10, y: 715};
    let cave = Cave::new(depth, target);
    writeln!(std::io::stdout(), "risk level: {}", cave.calc_risk_level())?;
    Ok(())
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Coordinate {
    y: u16,
    x: u16,
}

type CaveValue = u64;

enum Type {
    Rocky,
    Narrow,
    Wet,
}

impl Type {
    fn from_erosion_level(level: CaveValue) -> Self {
        use Type::*;
        match level % 3 {
            0 => Rocky,
            1 => Wet,
            2 => Narrow,
            _ => panic!("impossible state of level % 3: {}", level % 3),
        }
    }
    fn to_risk_level(&self) -> u8 {
        use Type::*;
        match self {
            Rocky => 0,
            Wet => 1,
            Narrow => 2,
        }
    }
}

struct Cave {
    depth: u32,
    target: Coordinate,
    regions: HashMap<Coordinate, Type>,
    // erosion_level: HashMap<Coordinate, u32>,
    // geologic_index: HashMap<Coordinate, u32>
}

impl Cave {
    fn new(depth: u32, target: Coordinate) -> Self {
        let mut geologic_indexes = HashMap::<Coordinate, CaveValue>::new();
        let mut erosion_levels = HashMap::<Coordinate, CaveValue>::new();
        let mut regions = HashMap::new();

        for y in 0..=target.y {
            for x in 0..=target.x {
                let geologic_index = if x == 0 && y == 0 {
                    0
                } else if x == target.x && y == target.y {
                    0
                } else if y == 0 {
                    CaveValue::from(x) * 16_807
                } else if x == 0 {
                    CaveValue::from(y) * 48_271
                } else {
                    if let (Some(west), Some(north)) = (
                        erosion_levels.get(&Coordinate { x: x - 1, y }),
                        erosion_levels.get(&Coordinate { x, y: y - 1 }),
                    ) {
                        west * north
                    } else {
                        // guaranteed to have erosion levels at these points.
                        panic!("wtf");
                    }
                };
                let coord = Coordinate { x, y };

                let erosion_level = (geologic_index + CaveValue::from(depth)) % 20_183;
                erosion_levels.insert(coord.clone(), erosion_level);
                regions.insert(coord.clone(), Type::from_erosion_level(erosion_level));
                geologic_indexes.insert(coord, geologic_index);
            }
        }

        Self {
            depth,
            target,
            regions,
        }
    }

    fn calc_risk_level(&self) -> u32 {
        (0..=self.target.y).map(|y| {
            (0..=self.target.x)
                .map(|x| {
                    u32::from(self.regions
                        .get(&Coordinate { x, y })
                        .unwrap()
                        .to_risk_level())
                })
                .sum::<u32>()
        }).sum()
    }
}

#[test]
fn test_cave() -> Result<()> {
    let cave = Cave::new(510, Coordinate { x: 10, y: 10 });
    assert_eq!(cave.calc_risk_level(), 114);
    let display = "\
        M=.|=.|.|=.|=|=.\n\
        .|=|=|||..|.=...\n\
        .==|....||=..|==\n\
        =.|....|.==.|==.\n\
        =|..==...=.|==..\n\
        =||.=.=||=|=..|=\n\
        |.=.===|||..=..|\n\
        |..==||=.|==|===\n\
        .=..===..=|.|||.\n\
        .======|||=|=.|=\n\
        .===|=|===T===||\n\
        =|||...|==..|=.|\n\
        =.=|=.=..=.||==|\n\
        ||=|=...|==.=|==\n\
        |=.=||===.|||===\n\
        ||.|==.|.|.||=||\n\
    ";
    println!("test_cave passed.");

    Ok(())
}
