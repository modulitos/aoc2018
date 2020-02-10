#[macro_use]
extern crate lazy_static;
use std::str::FromStr;

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

type Error = std::boxed::Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let coords = parse_coordinates(&input)?;
    let locations = parse_locations(&coords);
    writeln!(
        std::io::stdout(),
        "largest finite area size: {}",
        find_largest_finite_area(&locations, &coords)
    )?;
    Ok(())
}

fn find_largest_finite_area(locations: &Vec<Location>, coords: &Vec<Coordinate>) -> u32 {
    let bounding_coord_ids = Coordinate::get_bounding_coord_ids(coords);
    let map = locations
        .iter()
        .filter(|location| {
            if let Some(closest_coordinate_id) = location.closest_coordinate {
                !bounding_coord_ids.contains(&closest_coordinate_id)
            } else {
                false
            }
        })
        .fold(HashMap::<CoordinateId, u32>::new(), |mut map, location| {
            if let Some(closest_coordinate_id) = location.closest_coordinate {
                *map.entry(closest_coordinate_id).or_default() += 1;
            }
            map
        });
    //    assert_eq!(map.keys().len(), 2);
    println!("map: {:?}", map);
    map.iter()
        .max_by_key(|(_, &freq)| freq)
        .map(|(_, freq)| *freq)
        .expect("there must be enough locations!")
}

fn parse_coordinates(input: &str) -> Result<Vec<Coordinate>> {
    input
        .lines()
        .enumerate()
        .map(|(id, line)| {
            Ok(Coordinate {
                id: id as CoordinateId,
                point: line.parse()?,
            })
        })
        .collect::<Result<Vec<Coordinate>>>()
}

#[derive(Debug)]
struct Location {
    closest_coordinate: Option<CoordinateId>,
    point: Point,
}

fn parse_locations(coords: &Vec<Coordinate>) -> Vec<Location> {
    let (upper_left, lower_right) = Coordinate::get_grid_bounds(coords);
    (upper_left.x..=lower_right.x)
        .flat_map(|x| {
            (upper_left.y..=lower_right.y).map(move |y| {
                let point = Point { x, y };
                Location {
                    closest_coordinate: point
                        .get_closest_coordinate(coords)
                        .and_then(|coordinate| Some(coordinate.id)),
                    point,
                }
            })
        })
        .collect::<Vec<Location>>()
}

#[derive(Debug)]
struct Point {
    x: u32,
    y: u32,
}

impl FromStr for Point {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                # x, y coordinates, separated by a ', '
                (?P<x>[0-9]+),\s{1}(?P<y>[0-9]+)
                "
            )
            .unwrap();
        }

        let caps = RE.captures(s).unwrap();

        let x = caps["x"].parse()?;
        let y = caps["y"].parse()?;
        Ok(Point { x, y })
    }
}

impl Point {
    // Returns the coordinate that is closest to this point.
    // If more than one coordinate is tied for being closer, returns None
    fn get_closest_coordinate<'a>(&self, coords: &'a Vec<Coordinate>) -> Option<&'a Coordinate> {
        let mut closest_coord = None;
        let mut shortest_distance = std::u32::MAX;
        for coord in coords {
            let distance = self.get_distance(&coord.point);
            if distance < shortest_distance {
                shortest_distance = distance;
                closest_coord = Some(coord);
            } else if distance == shortest_distance {
                closest_coord = None;
            }
        }
        closest_coord
    }

    // Returns the Manhattan Distance between two Points
    fn get_distance(&self, other: &Point) -> u32 {
        let d_x = if self.x > other.x {
            self.x.saturating_sub(other.x)
        } else {
            other.x.saturating_sub(self.x)
        };
        let d_y = if self.y > other.y {
            self.y.saturating_sub(other.y)
        } else {
            other.y.saturating_sub(self.y)
        };

        d_x.saturating_add(d_y)
    }
}

type CoordinateId = u8;

struct Coordinate {
    id: CoordinateId,
    point: Point,
}

impl Coordinate {
    // Returns a tuple representing the top-left, and bottom-right of the grid.
    fn get_grid_bounds(coords: &Vec<Coordinate>) -> (Point, Point) {
        let (min_x, min_y, max_x, max_y) = coords.iter().fold(
            (std::u32::MAX, std::u32::MAX, 0, 0),
            |(min_x, min_y, max_x, max_y), coord| {
                (
                    std::cmp::min(min_x, coord.point.x),
                    std::cmp::min(min_y, coord.point.y),
                    std::cmp::max(max_x, coord.point.x),
                    std::cmp::max(max_y, coord.point.y),
                )
            },
        );

        (Point { x: min_x, y: min_y }, Point { x: max_x, y: max_y })
    }

    // TODO: bug here! don't just get the corners, but get anything that is on an outer edge!
    fn get_bounding_coord_ids(coords: &Vec<Coordinate>) -> HashSet<CoordinateId> {
        let (upper_left, lower_right) = Coordinate::get_grid_bounds(coords);
        coords.iter().fold(HashSet::new(), |mut set, coord| {
            let point = &coord.point;
            if point.x == upper_left.x
                || point.x == lower_right.x
                || point.y == upper_left.y
                || point.y == lower_right.y
            {
                set.insert(coord.id);
            }
            set
        })
    }
}

#[test]
fn test_find_largest_finite_area() -> Result<()> {
    let s = "\
        1, 1\n\
        1, 6\n\
        8, 3\n\
        3, 4\n\
        5, 5\n\
        8, 9\
    ";
    let coords = parse_coordinates(&s)?;
    let locations = parse_locations(&coords);
    assert_eq!(find_largest_finite_area(&locations, &coords), 17);
    println!("find_largest_finite_area passed!");
    Ok(())
}
