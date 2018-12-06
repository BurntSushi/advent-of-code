use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self, Read, Write};
use std::result;
use std::str::FromStr;

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let coordinates = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<Coordinate>>>()?;
    if coordinates.is_empty() {
        return Err(From::from("no coordinates given"));
    }

    let mut grid = Grid::new(coordinates);
    grid.find_finite();

    part1(&grid)?;
    part2(&grid)?;
    Ok(())
}

fn part1(grid: &Grid) -> Result<()> {
    let mut biggest_area = 0;
    for &loc in &grid.finite {
        let mut candidate_area = 0;
        for &loc2 in grid.table.values() {
            if loc == loc2 {
                candidate_area += 1;
            }
        }
        if candidate_area > biggest_area {
            biggest_area = candidate_area;
        }
    }
    writeln!(io::stdout(), "biggest area: {}", biggest_area)?;
    Ok(())
}

fn part2(grid: &Grid) -> Result<()> {
    // Similar to part 1, we simply choose a bounding box. Experimentation
    // indicates convergence.
    let bound = 500;
    let mut size = 0;
    for x in -bound..=bound {
        for y in -bound..=bound {
            if grid.distance_sum(Coordinate { x, y }) < 10000 {
                size += 1;
            }
        }
    }
    writeln!(io::stdout(), "size: {}", size)?;
    Ok(())
}

#[derive(Debug)]
struct Grid {
    // all coordinates given in the input
    locations: Vec<Coordinate>,
    // all known finite coordinates
    finite: HashSet<Coordinate>,
    // a map from an arbitrary coordinate to its closest location
    table: HashMap<Coordinate, Coordinate>,
}

impl Grid {
    fn new(locations: Vec<Coordinate>) -> Grid {
        assert!(!locations.is_empty());
        Grid { locations, finite: HashSet::new(), table: HashMap::new() }
    }

    fn find_finite(&mut self) {
        // This isn't actually guaranteed to be correct. We simply assert that
        // after some fixed number of iterations, our set of finite locations
        // converges.
        //
        // I started this trying for a solution that didn't assume a bounding
        // box size, which would have made this much simpler. At the end of
        // the day, we're still not fully general because there is no logic
        // for detecting convergence.
        for step in 0..100 {
            for loc in &self.locations {
                if self.finite.contains(&loc) {
                    continue;
                }
                for c in loc.border(step) {
                    let closest = match self.closest_location(c) {
                        None => continue,
                        Some(closest) => closest,
                    };
                    self.table.insert(c, closest);
                }
            }
            for &loc in &self.locations {
                if !loc.border(step).any(|c| self.table.get(&c) == Some(&loc)) {
                    self.finite.insert(loc);
                }
            }
        }
    }

    /// Returns the sum of distances between the given coordinate and all
    /// locations.
    fn distance_sum(&self, c: Coordinate) -> i32 {
        self.locations.iter().map(|&loc| loc.distance(c)).sum()
    }

    /// Returns a unique location with minimum distance to the given
    /// coordinate. If no such unique minimum exists, then None is returned.
    fn closest_location(&self, c: Coordinate) -> Option<Coordinate> {
        let (mut min, mut unique) = (self.locations[0], true);
        for &loc in &self.locations[1..] {
            if loc.distance(c) == min.distance(c) {
                unique = false;
            } else if loc.distance(c) < min.distance(c) {
                min = loc;
                unique = true;
            }
        }
        if !unique {
            None
        } else {
            Some(min)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    fn distance(self, other: Coordinate) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn border(self, step: i32) -> impl Iterator<Item=Coordinate> {
        (self.x - step..=self.x + step)
            .flat_map(move |x| {
                (self.y - step..=self.y + step)
                .map(move |y| Coordinate { x, y })
            })
            .filter(move |&c2| self.distance(c2) == step)
    }
}

impl FromStr for Coordinate {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Coordinate> {
        let comma = match s.find(",") {
            None => return Err(From::from("could not find comma")),
            Some(i) => i,
        };
        let (pos1, pos2) = (&s[..comma].trim(), s[comma + 1..].trim());
        Ok(Coordinate { x: pos1.parse()?, y: pos2.parse()? })
    }
}
