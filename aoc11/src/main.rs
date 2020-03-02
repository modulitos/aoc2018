use std::cmp;
use std::error;
use std::io::Write;
use std::result;

type Error = std::boxed::Box<dyn error::Error>;
type Result<R, E = Error> = result::Result<R, E>;

fn main() -> Result<()> {
    let grid = Grid::new(4455);

    let (x, y) = grid.find_largest();
    writeln!(
        std::io::stdout(),
        "3x3 coordinate of max fuel cells for grid: ({}, {})",
        x,
        y
    )?;
    Ok(())
}

type PowerLevel = i32;

struct Grid {
    cells: [[PowerLevel; 300]; 300],
}

impl Grid {
    fn new(serial_number: u16) -> Self {
        let mut cells = [[0; 300]; 300];

        for y in 1..=300 {
            for x in 1..=300 {
                cells[x - 1][y - 1] = Grid::get_power_level(serial_number, x as u16, y as u16);
            }
        }
        Grid { cells }
    }

    // scan the grid to find the 3x3 grid with the largest sum

    fn find_largest(&self) -> (usize, usize) {
        // calculate the value of the current 3x3 grid.
        // let mut max = (0..3).fold(0, |sum, y| {
        //     sum + (0..3).fold(0, |sum, x| sum + self.cells[x][y])
        // });
        let mut max = std::i32::MIN;
        // let mut curr_sum = max;
        let mut max_coords = (0, 0);

        for y in 0..=297 {
            let mut curr_sum = (y..=y + 2).fold(0, |sum, y| {
                sum + (0..=2).fold(0, |sum, x| sum + self.cells[x][y])
            });
            for x in 1..=297 {
                // subtract the value of the left-most col
                curr_sum -= (y..=y + 2).fold(0, |sum, y| sum + self.cells[x - 1][y]);

                // add the value of the right-most col
                curr_sum += (y..=y + 2).fold(0, |sum, y| sum + self.cells[x + 2][y]);

                if curr_sum > max {
                    max_coords = (x, y);
                    max = curr_sum;
                }
            }
        }

        (max_coords.0 + 1, max_coords.1 + 1)
    }

    fn get_power_level(serial_number: u16, x: u16, y: u16) -> i32 {
        let rack_id = i32::from(x) + 10;
        let power_level = ((rack_id * i32::from(y)) + i32::from(serial_number)) * rack_id;

        // Keep only the hundreds digit of the power level (so 12345 becomes 3; numbers with no
        // hundreds digit become 0)
        (power_level / 100) % 10 - 5
    }
}

#[test]
fn test_power_cells() -> Result<()> {
    assert_eq!(Grid::get_power_level(8, 3, 5), 4);

    assert_eq!(Grid::get_power_level(57, 122, 79), -5);
    assert_eq!(Grid::get_power_level(39, 217, 196), 0);
    assert_eq!(Grid::get_power_level(71, 101, 153), 4);

    println!("test power cells passed.");
    Ok(())
}

#[test]
fn test_grid_find() -> Result<()> {
    let grid = Grid::new(18);
    assert_eq!(grid.find_largest(), (33, 45));

    let grid = Grid::new(42);
    assert_eq!(grid.find_largest(), (21, 61));
    println!("test grid find passed.");
    Ok(())
}

// #[test]
// fn test_grid_find_any_size() -> Result<()> {
//     // For grid serial number 18, the largest total square (with a total power of 113) is 16x16 and
//     // has a top-left corner of 90,269, so its identifier is 90,269,16.
//     let grid = Grid::new(18);
//     assert_eq!(grid.find_largest_any_size(), (90, 269, 16));
//
//     // For grid serial number 42, the largest total square (with a total power of 119) is 12x12 and
//     // has a top-left corner of 232,251, so its identifier is 232,251,12.
//
//     Ok(())
// }
