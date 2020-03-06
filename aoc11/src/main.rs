use std::cmp;
use std::error;
use std::io::Write;
use std::result;

type Error = std::boxed::Box<dyn error::Error>;
type Result<R, E = Error> = result::Result<R, E>;

fn main() -> Result<()> {
    let grid = Grid::new(4455);

    let (x, y) = grid.find_largest_3x3();
    writeln!(
        std::io::stdout(),
        "3x3 coordinate of max fuel cells for grid: ({}, {})",
        x,
        y
    )?;

    writeln!(
        std::io::stdout(),
        "location and size of largest grid: {:?}",
        grid.find_largest()?
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

        for y in 0..300 {
            for x in 0..300 {
                cells[x][y] = Grid::get_power_level(serial_number, x as u16, y as u16);
            }
        }
        Grid { cells }
    }

    // Scan the grid to find the 3x3 sub-grid with the largest sum.
    // Returns the coordinates of the sub-grid's top left corner.

    fn find_largest_3x3(&self) -> (usize, usize) {
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

    // Scan the grid to find the square sub-grid with the largest sum.
    // Returns the coordinates of the sub-grid's top left corner, along with the size of the
    // sub-grid.

    fn find_largest(&self) -> Result<(usize, usize, usize)> {
        // create a summed area table: https://en.wikipedia.org/wiki/Summed-area_table
        let mut sums: [[i32; 300]; 300] = [[0; 300]; 300];
        for y in 0..300 {
            for x in 0..300 {
                let top = if y == 0 { 0 } else { sums[x][y - 1] };
                let left = if x == 0 { 0 } else { sums[x - 1][y] };
                let top_left = if x == 0 || y == 0 {
                    0
                } else {
                    sums[x - 1][y - 1]
                };
                sums[x][y] = i32::from(self.cells[x][y]) + top + left - top_left;
            }
        }

        let mut max_sum = std::i32::MIN;
        let mut results = (0, 0, 0);
        let mut found_dupe = false;
        for ymin in 0..300 {
            for xmin in 0..300 {
                for (xmax, ymax) in ((xmin + 1)..300).zip((ymin + 1)..300) {
                    let length = xmax - xmin;
                    // calculates the grid's sum, leveraging properties of the summed area table:
                    let curr_sum =
                        sums[xmax][ymax] - sums[xmin][ymax] - sums[xmax][ymin] + sums[xmin][ymin];
                    if curr_sum > max_sum {
                        // Add 1 to account for the 1-based indexing expected from the results
                        // Add another 1 to account for xmin and ymin not being in the bounds of the sub-grid.
                        results = (xmin + 2, ymin + 2, length);
                        max_sum = curr_sum;
                        found_dupe = false;
                    } else if curr_sum == max_sum {
                        found_dupe = true;
                    }
                }
            }
        }

        if found_dupe {
            return Err(Error::from(format!(
                "Not supposed to have more than one sum {:?}",
                results
            )));
        }

        Ok(results)
    }

    fn get_power_level(serial_number: u16, x: u16, y: u16) -> i32 {
        // add 1 to x and y to account for 1-based indexing
        let rack_id = i32::from(x + 1) + 10;
        let power_level = ((rack_id * i32::from(y + 1)) + i32::from(serial_number)) * rack_id;

        // Keep only the hundreds digit of the power level (so 12345 becomes 3; numbers with no
        // hundreds digit become 0)
        (power_level / 100) % 10 - 5
    }
}

#[test]
fn test_power_cells() -> Result<()> {
    assert_eq!(Grid::get_power_level(8, 2, 4), 4);

    assert_eq!(Grid::get_power_level(57, 121, 78), -5);
    assert_eq!(Grid::get_power_level(39, 216, 195), 0);
    assert_eq!(Grid::get_power_level(71, 100, 152), 4);

    println!("test power cells passed.");
    Ok(())
}

#[test]
fn test_grid_find_3x3() -> Result<()> {
    let grid = Grid::new(18);
    assert_eq!(grid.find_largest_3x3(), (33, 45));

    let grid = Grid::new(42);
    assert_eq!(grid.find_largest_3x3(), (21, 61));
    println!("test grid find 3x3 passed.");
    Ok(())
}

#[test]
fn test_grid_find_largest() -> Result<()> {
    // For grid serial number 18, the largest total square (with a total power of 113) is 16x16 and
    // has a top-left corner of 90,269, so its identifier is 90,269,16.
    let grid = Grid::new(18);
    assert_eq!(grid.find_largest()?, (90, 269, 16));

    // For grid serial number 42, the largest total square (with a total power of 119) is 12x12 and
    // has a top-left corner of 232,251, so its identifier is 232,251,12.
    let grid = Grid::new(42);
    assert_eq!(grid.find_largest()?, (232, 251, 12));
    println!("test find_largest passed.");
    Ok(())
}

// This function borrows a slice
fn analyze_slice(slice: &[i32]) {
    println!("first element of the slice: {}", slice[0]);
    println!("the slice has {} elements", slice.len());
}

#[test]
fn test_array_slicing() {
    // Fixed-size array (type signature is superfluous)
    let xs: [i32; 5] = [1, 2, 3, 4, 5];

    // All elements can be initialized to the same value
    // let ys: [i32; 500] = [0; 500];
    let ys = [[0; 10]; 10];

    assert_eq!(ys.len(), 10);
    assert_eq!(ys[0].len(), 10);

    let ys_2 = &ys[1..];
    assert_eq!(ys_2.len(), 9);
    assert_eq!(ys_2[0].len(), 10);
    assert_eq!(ys.len(), 10);
    assert_eq!(ys[0].len(), 10);

    let ys_3 = &ys[1..][1..];
    assert_eq!(ys_3.len(), 8);
    assert_eq!(ys_3[0].len(), 10);
    assert_eq!(ys.len(), 10);
    assert_eq!(ys[0].len(), 10);


    // Indexing starts at 0
    // println!("first element of the array: {}", xs[0]);
    // println!("second element of the array: {}", xs[1]);

    // `len` returns the size of the array
    // println!("array size: {}", xs.len());

    // Arrays are stack allocated
    // println!("array occupies {} bytes", std::mem::size_of_val(&xs));

    // Arrays can be automatically borrowed as slices
    // println!("borrow the whole array as a slice");
    analyze_slice(&xs);

    // Slices can point to a section of an array
    // They are of the form [starting_index..ending_index]
    // starting_index is the first position in the slice
    // ending_index is one more than the last position in the slice
    // println!("borrow a section of the array as a slice");
    analyze_slice(&xs[1..4]);

    println!("array slicing tests passed");
}
