mod error;

use error::{Error, Result};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::slice::Iter;

fn main() -> Result<()> {
    // puzzle input:
    // depth: 3339
    // target: 10,715
    let cave = Cave::new(3339, Coordinate { x: 10, y: 715 });
    writeln!(std::io::stdout(), "risk level: {}", cave.calc_risk_level())?;
    let fastest_time = find_fastest_time_to_target(&cave)?;
    writeln!(std::io::stdout(), "min target time: {}", fastest_time)?;
    Ok(())
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Ord, PartialOrd)]
struct Coordinate {
    x: u16,
    y: u16,
}

impl Coordinate {
    fn get_adjacent(&self) -> Vec<Coordinate> {
        let mut adjacent = vec![];
        let x = self.x;
        let y = self.y;
        if x > 0 {
            adjacent.push(Coordinate { x: x - 1, y })
        }
        if y > 0 {
            adjacent.push(Coordinate { x, y: y - 1 })
        }
        adjacent.push(Coordinate { x: x + 1, y });
        adjacent.push(Coordinate { x, y: y + 1 });

        adjacent
    }
}

type CaveValue = u64;

enum Region {
    Rocky,
    Narrow,
    Wet,
}

impl Region {
    fn from_erosion_level(level: CaveValue) -> Self {
        use Region::*;
        match level % 3 {
            0 => Rocky,
            1 => Wet,
            2 => Narrow,
            _ => panic!("impossible state of level % 3: {}", level % 3),
        }
    }
    fn to_risk_level(&self) -> u8 {
        use Region::*;
        match self {
            Rocky => 0,
            Wet => 1,
            Narrow => 2,
        }
    }
    fn to_char(&self) -> char {
        use Region::*;
        match &self {
            Rocky => '.',
            Wet => '=',
            Narrow => '|',
        }
    }
}

struct Cave {
    target: Coordinate,
    regions: HashMap<Coordinate, Region>,
}

impl Cave {
    fn new(depth: u32, target: Coordinate) -> Self {
        let mut geologic_indexes = HashMap::<Coordinate, CaveValue>::new();
        let mut erosion_levels = HashMap::<Coordinate, CaveValue>::new();
        let mut regions = HashMap::new();

        // Use this buffer so that our regions have some extra space, in case we need to move beyond
        // the x and y limits of the target

        let max_y = target.y * 10;
        let max_x = target.x * 10;

        for y in 0..=max_y {
            for x in 0..=max_x {
                let geologic_index = if x == 0 && y == 0 {
                    0
                } else if x == target.x && y == target.y {
                    0
                } else if y == 0 {
                    CaveValue::from(x) * 16_807
                } else if x == 0 {
                    CaveValue::from(y) * 48_271
                } else {
                    let west = erosion_levels
                        .get(&Coordinate { x: x - 1, y })
                        .expect(&format!("erosion level must exist at ({}, {})", x - 1, y));
                    let north = erosion_levels
                        .get(&Coordinate { x, y: y - 1 })
                        .expect(&format!("erosion level must exist at ({}, {})", x, y - 1));
                    west * north
                };
                let coord = Coordinate { x, y };

                let erosion_level = (geologic_index + CaveValue::from(depth)) % 20_183;
                erosion_levels.insert(coord, erosion_level);
                regions.insert(coord, Region::from_erosion_level(erosion_level));
                geologic_indexes.insert(coord, geologic_index);
            }
        }

        Self { target, regions }
    }

    fn calc_risk_level(&self) -> u32 {
        (0..=self.target.y)
            .map(|y| {
                (0..=self.target.x)
                    .map(|x| {
                        u32::from(
                            self.regions
                                .get(&Coordinate { x, y })
                                .unwrap()
                                .to_risk_level(),
                        )
                    })
                    .sum::<u32>()
            })
            .sum()
    }
}

// Used for testing:

impl Display for Cave {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (0..=self.target.y)
            .map(|y| {
                let row = (0..=self.target.x)
                    .map(|x| {
                        if x == 0 && y == 0 {
                            'M'
                        } else if x == self.target.x && y == self.target.y {
                            'T'
                        } else {
                            self.regions.get(&Coordinate { x, y }).unwrap().to_char()
                        }
                    })
                    .collect::<String>();
                writeln!(f, "{}", row)?;
                Ok(())
            })
            .collect::<std::fmt::Result>()?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Ord, PartialOrd)]
enum Tool {
    ClimbingGear,
    Torch,
    Neither,
}

impl Tool {
    fn can_access(&self, region: &Region) -> bool {
        use Region::*;
        use Tool::*;

        match (region, &self) {
            (Rocky, Torch) | (Rocky, ClimbingGear) => true,
            (Wet, ClimbingGear) | (Wet, Neither) => true,
            (Narrow, Torch) | (Narrow, Neither) => true,
            _ => false,
        }
    }
    fn iter() -> Iter<'static, Tool> {
        use Tool::*;
        static TOOLS: [Tool; 3] = [ClimbingGear, Torch, Neither];
        TOOLS.iter()
    }
}

type Time = u32; // time, in minutes

fn find_fastest_time_to_target(cave: &Cave) -> Result<Time> {
    // This is our cache of explored locations:
    let mut best_times = HashMap::<(Coordinate, Tool), Time>::new();

    // Using dijkstra's algorithm:
    let mut p_queue = BinaryHeap::<Reverse<(Time, Coordinate, Tool)>>::new();
    // start at the cave mouth:
    let coord = Coordinate { x: 0, y: 0 };

    use Tool::*;
    p_queue.push(Reverse((0, coord, Torch)));

    while let Some(Reverse((curr_time, curr_coord, curr_tool))) = p_queue.pop() {
        if let Some(prev_time) = best_times.get(&(curr_coord, curr_tool)) {
            if prev_time <= &curr_time {
                // skip exploring the current coord/tool combo if it's already accessible in a faster
                continue;
            }
        }

        // save our new best time:
        best_times.insert((curr_coord, curr_tool), curr_time);

        if curr_coord == cave.target && curr_tool == Torch {
            return Ok(curr_time);
        }

        // Explore new paths were we equip a different tool at this location:
        // Try equipping with new tools:
        // let tools_to_explore = Tool::iter()
        Tool::iter()
            .filter(|tool| tool != &&curr_tool)
            .filter(|tool| tool.can_access(cave.regions.get(&curr_coord).unwrap()))
            .for_each(|&tool| p_queue.push(Reverse((curr_time + 7, curr_coord, tool))));

        // Explore the adjacent coordinates that are accessible with our current tool.
        curr_coord
            .get_adjacent()
            .into_iter()
            .filter(|coord| {
                // coord is accessible by the tool
                // If the adjacent coord is off the map, then let's not explore it.
                // Note this map includes buffered regions beyond the extent of the target coord.
                cave.regions.contains_key(&coord)
                    && curr_tool.can_access(cave.regions.get(&coord).unwrap())
            })
            .for_each(|coord| p_queue.push(Reverse((curr_time + 1, coord, curr_tool))));
    }
    Err(Error::from("unable to reach the target within the Cave."))
}

#[test]
fn test_cave() -> Result<()> {
    let cave = Cave::new(510, Coordinate { x: 10, y: 10 });
    assert_eq!(cave.calc_risk_level(), 114);
    println!("test_cave risk level passed.");
    let display = "\
        M=.|=.|.|=.\n\
        .|=|=|||..|\n\
        .==|....||=\n\
        =.|....|.==\n\
        =|..==...=.\n\
        =||.=.=||=|\n\
        |.=.===|||.\n\
        |..==||=.|=\n\
        .=..===..=|\n\
        .======|||=\n\
        .===|=|===T\n\
    ";
    let cave_regions = format!("{}", cave);
    println!("cave_regions:\n{}", cave_regions);
    assert_eq!(cave_regions, display);

    assert_eq!(find_fastest_time_to_target(&cave)?, 45);
    println!("test_cave fastest_time_to_target passed.");

    Ok(())
}
