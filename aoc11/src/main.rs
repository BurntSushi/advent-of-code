use std::error::Error;
use std::io::{self, Write};
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

const SERIAL_NUMBER: i32 = 2866;
const GRID_SIZE: i32 = 300;

fn main() -> Result<()> {
    assert!(GRID_SIZE > 0);

    let mut grid = Grid::new(GRID_SIZE);
    for x in 1..=GRID_SIZE {
        for y in 1..=GRID_SIZE {
            grid.set(x, y, fuel_cell_power(x, y));
        }
    }

    part1(&grid)?;
    part2(&grid)?;
    Ok(())
}

fn part1(grid: &Grid) -> Result<()> {
    let (mut max_x, mut max_y, mut max_power) =
        (1, 1, grid.square_power(3, 1, 1));
    for x in 1..=GRID_SIZE {
        for y in 1..=GRID_SIZE {
            let power = grid.square_power(3, x, y);
            if power > max_power {
                max_x = x;
                max_y = y;
                max_power = power;
            }
        }
    }
    writeln!(io::stdout(), "most powerful 3x3 square: {},{}", max_x, max_y)?;
    Ok(())
}

fn part2(grid: &Grid) -> Result<()> {
    let (mut max_size, mut max_x, mut max_y, mut max_power) =
        (1, 1, 1, grid.square_power(1, 1, 1));

    // This smells like a problem that can reuse results to make it faster,
    // but didn't have time to think through that. This brute force approach
    // is slow, but simple. In particular, we are bailed out by the fact that
    // we do not check squares that contain a cell outside of the grid. In
    // practice, this makes checking squares that are large with respect to
    // the full grid very fast.
    for size in 1..=GRID_SIZE {
        for x in 1..=GRID_SIZE {
            for y in 1..=GRID_SIZE {
                let power = grid.square_power(size, x, y);
                if power > max_power {
                    max_size = size;
                    max_x = x;
                    max_y = y;
                    max_power = power;
                }
                if y + size > GRID_SIZE {
                    break;
                }
            }
            if x + size > GRID_SIZE {
                break;
            }
        }
    }
    writeln!(
        io::stdout(),
        "most powerful square: {},{},{}",
        max_x, max_y, max_size,
    )?;
    Ok(())
}

struct Grid {
    power: Vec<Vec<i32>>,
}

impl Grid {
    fn new(size: i32) -> Grid {
        Grid { power: vec![vec![0; size as usize]; size as usize] }
    }

    fn set(&mut self, x: i32, y: i32, power: i32) {
        self.power[x as usize - 1][y as usize - 1] = power;
    }

    fn get(&self, x: i32, y: i32) -> Option<i32> {
        let (x, y) = (x - 1, y - 1);
        if 0 <= x && x < GRID_SIZE && 0 <= y && y < GRID_SIZE {
            Some(self.power[x as usize][y as usize])
        } else {
            None
        }
    }

    fn square_power(
        &self,
        size: i32,
        top_left_x: i32,
        top_left_y: i32,
    ) -> i32 {
        let mut power = 0;
        for x in top_left_x..top_left_x + size {
            for y in top_left_y..top_left_y + size {
                power += self.get(x, y).unwrap_or(0);
            }
        }
        power
    }
}

fn fuel_cell_power(x: i32, y: i32) -> i32 {
    let rack_id = x + 10;
    let mut power = rack_id * y;
    power += SERIAL_NUMBER;
    power *= rack_id;
    power = (power / 100) % 10;
    power -= 5;
    power
}
