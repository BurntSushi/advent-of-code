use std::cmp;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::ops::RangeInclusive;
use std::result;
use std::str::{self, FromStr};

use lazy_static::lazy_static;
use regex::Regex;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut scans: Vec<ClayScan> = vec![];
    for line in input.lines() {
        let scan = line.parse().or_else(|err| {
            err!("failed to parse '{:?}': {}", line, err)
        })?;
        scans.push(scan);
    }

    let mut ground = Ground::new();
    ground.add_clay_scans(&scans);
    while ground.add_water() {}

    writeln!(io::stdout(), "reachable tiles: {}", ground.water_in_bounds())?;
    writeln!(io::stdout(), "remaining water: {}", ground.water_at_rest())?;
    Ok(())
}

#[derive(Clone, Debug)]
struct Ground {
    spring: Coordinate,
    clay: HashSet<Coordinate>,
    water: HashMap<Coordinate, Water>,
    // When determing the next downward coordinate, we use this set to avoid
    // searching for a spot among settled water. If we know that the entire
    // next row is settled, then just move on---there is no place for the water
    // to go. This substantially speeds up this particular implementation
    // which is otherwise quite slow!
    settled: HashSet<Coordinate>,
    min: Coordinate,
    max: Coordinate,
}

impl Ground {
    fn new() -> Ground {
        Ground {
            spring: Coordinate { x: 500, y: 0 },
            clay: HashSet::new(),
            water: HashMap::new(),
            settled: HashSet::new(),
            min: Coordinate { x: 0, y: 0 },
            max: Coordinate { x: 0, y: 0 },
        }
    }

    fn water_in_bounds(&self) -> usize {
        self.water
            .keys()
            .filter(|&&c| self.min.y <= c.y && c.y <= self.max.y)
            .count()
    }

    fn water_at_rest(&self) -> usize {
        self.water.values().filter(|&&w| w == Water::Rest).count()
    }

    fn add_water(&mut self) -> bool {
        let mut rested = false;
        let mut stack = vec![self.spring];
        let mut seen = HashSet::new();
        while let Some(c) = stack.pop() {
            if seen.contains(&c) {
                continue;
            }
            seen.insert(c);

            if let Some(down) = self.down(c) {
                if down.y <= self.max.y {
                    stack.push(down);
                    self.water.insert(down, Water::Flow);
                }
                continue;
            }

            let mut blocked = true;
            let mut c2 = c;
            while let Some(left) = self.left(c2) {
                c2 = left;
                self.water.insert(c2, Water::Flow);
                if self.down(c2).is_some() {
                    stack.push(c2);
                    blocked = false;
                    break;
                }
            }
            c2 = c;
            while let Some(right) = self.right(c2) {
                c2 = right;
                self.water.insert(c2, Water::Flow);
                if self.down(c2).is_some() {
                    stack.push(c2);
                    blocked = false;
                    break;
                }
            }
            if blocked {
                self.water.insert(c, Water::Rest);
                rested = true;
            }
        }
        rested
    }

    fn down(&mut self, c: Coordinate) -> Option<Coordinate> {
        let down = Coordinate { x: c.x, y: c.y + 1 };
        if self.is_clay(down) {
            return None;
        }
        if !self.is_settled(down) {
            return Some(down);
        }

        let mut left = Coordinate { x: down.x - 1, y: down.y };
        if !self.settled.contains(&left) {
            let start = left;
            while !self.is_clay(left) {
                if !self.is_settled(left) {
                    return Some(left);
                }
                left.x -= 1;
            }
            self.settled.insert(start);
        }

        let mut right = Coordinate { x: down.x + 1, y: down.y };
        if !self.settled.contains(&right) {
            let start = right;
            while !self.is_clay(right) {
                if !self.is_settled(right) {
                    return Some(right);
                }
                right.x += 1;
            }
            self.settled.insert(start);
        }

        None
    }

    fn left(&self, c: Coordinate) -> Option<Coordinate> {
        let left = Coordinate { x: c.x - 1, y: c.y };
        if self.is_clay(left) || self.is_settled(left) {
            None
        } else {
            Some(left)
        }
    }

    fn right(&self, c: Coordinate) -> Option<Coordinate> {
        let right = Coordinate { x: c.x + 1, y: c.y };
        if self.is_clay(right) || self.is_settled(right) {
            None
        } else {
            Some(right)
        }
    }

    fn is_clay(&self, c: Coordinate) -> bool {
        self.clay.contains(&c)
    }

    fn is_settled(&self, c: Coordinate) -> bool {
        self.water.get(&c).map_or(false, |&w| w == Water::Rest)
    }

    fn add_clay_scans(&mut self, scans: &[ClayScan]) {
        if scans.is_empty() {
            return;
        }
        self.min = Coordinate {
            x: *scans[0].x.start(),
            y: *scans[0].y.start(),
        };
        self.max = self.min;
        for scan in scans {
            for x in scan.x.clone() {
                for y in scan.y.clone() {
                    let c = Coordinate { x, y };
                    self.clay.insert(c);
                    self.min.x = cmp::min(self.min.x, c.x);
                    self.min.y = cmp::min(self.min.y, c.y);
                    self.max.x = cmp::max(self.max.x, c.x);
                    self.max.y = cmp::max(self.max.y, c.y);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Water {
    Flow,
    Rest,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coordinate {
    x: i64,
    y: i64,
}

impl Coordinate {
}

#[derive(Clone, Debug)]
struct ClayScan {
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
}

impl FromStr for ClayScan {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<ClayScan> {
        lazy_static! {
            static ref RE1: Regex = Regex::new(r"(?x)
                x=(?P<x>[0-9]+),\sy=(?P<y1>[0-9]+)\.\.(?P<y2>[0-9]+)
            ").unwrap();

            static ref RE2: Regex = Regex::new(r"(?x)
                y=(?P<y>[0-9]+),\sx=(?P<x1>[0-9]+)\.\.(?P<x2>[0-9]+)
            ").unwrap();
        }

        if let Some(caps) = RE1.captures(s) {
            let x = caps["x"].parse()?;
            let (y1, y2) = (caps["y1"].parse()?, caps["y2"].parse()?);
            return Ok(ClayScan {
                x: x..=x,
                y: y1..=y2,
            });
        }
        if let Some(caps) = RE2.captures(s) {
            let (x1, x2) = (caps["x1"].parse()?, caps["x2"].parse()?);
            let y = caps["y"].parse()?;
            return Ok(ClayScan {
                x: x1..=x2,
                y: y..=y,
            });
        }
        err!("unrecognized clay scan: {:?}", s)
    }
}

impl fmt::Display for Ground {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in self.spring.y..=self.max.y {
            for x in (self.min.x - 1)..=(self.max.x + 1) {
                let c = Coordinate { x, y };
                if c == self.spring {
                    write!(f, "+")?;
                } else if self.clay.contains(&c) {
                    write!(f, "#")?;
                } else if let Some(&w) = self.water.get(&c) {
                    write!(f, "{}", w)?;
                } else {
                    write!(f, ".")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl fmt::Display for Water {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Water::Flow => write!(f, "|"),
            Water::Rest => write!(f, "~"),
        }
    }
}
