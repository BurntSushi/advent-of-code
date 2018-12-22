use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::io::{self, Write};
use std::result;

const DEPTH: usize = 9171;
const TARGET: Coordinate = Coordinate { x: 7, y: 721 };

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let cave = Cave::new(DEPTH, TARGET)?;
    writeln!(io::stdout(), "risk level: {}", cave.risk_level())?;
    writeln!(io::stdout(), "time to target: {}", cave.shortest_time()?)?;
    Ok(())
}

#[derive(Clone, Debug)]
struct Cave {
    depth: usize,
    target: Coordinate,
    bound: Coordinate,
    regions: Vec<Vec<Region>>,
}

impl Cave {
    fn new(depth: usize, target: Coordinate) -> Result<Cave> {
        let mut scanner = CaveScanner::new(depth, target);
        scanner.scan();
        scanner.cave()
    }

    fn risk_level(&self) -> usize {
        let mut risk_level = 0;
        for y in 0..=self.target.y {
            for x in 0..=self.target.x {
                risk_level += self.regions[y][x].risk_level();
            }
        }
        risk_level
    }

    fn shortest_time(&self) -> Result<usize> {
        type Time = usize; // minutes
        type PriorityQueue = BinaryHeap<Reverse<(Time, Coordinate, Equip)>>;

        let mut queue: PriorityQueue = BinaryHeap::new();
        let mut best: HashMap<(Coordinate, Equip), Time> = HashMap::new();

        queue.push(Reverse((0, Coordinate { x: 0, y: 0 }, Equip::Torch)));
        while let Some(Reverse((time, c, equip))) = queue.pop() {
            if best.contains_key(&(c, equip)) && best[&(c, equip)] <= time {
                continue;
            }
            best.insert((c, equip), time);
            if c == self.target && equip == Equip::Torch {
                return Ok(time);
            }

            // Try equipping different tools.
            for &e in &[Equip::Torch, Equip::Gear, Equip::Neither] {
                if self.regions[c.y][c.x].can_equip(e) {
                    queue.push(Reverse((time + 7, c, e)));
                }
            }
            // Try visiting each neighbor.
            for &(x, y) in &[(0, -1), (1, 0), (0, 1), (-1, 0)] {
                if (x < 0 && c.x == 0) || (y < 0 && c.y == 0) {
                    continue;
                }

                let x = (c.x as i64 + x) as usize;
                let y = (c.y as i64 + y) as usize;
                if x > self.bound.x || y > self.bound.y {
                    continue;
                }
                if self.regions[y][x].can_equip(equip) {
                    let neighbor = Coordinate { x, y };
                    queue.push(Reverse((time + 1, neighbor, equip)));
                }
            }
        }
        err!("could not find a path to {:?}", self.target)
    }
}

#[derive(Clone, Debug)]
struct CaveScanner {
    depth: usize,
    target: Coordinate,
    bound: Coordinate,
    regions: Vec<Vec<Option<Region>>>,
}

impl CaveScanner {
    fn new(depth: usize, target: Coordinate) -> CaveScanner {
        // In part 2, we might need to travel outside the rectangle created
        // by the mouth and the target. We heuristic expand the bounds by a
        // factor of 2 in both directions. I don't think there is any guarantee
        // that this works in general, but ¯\_(ツ)_/¯.
        //
        // Actually, a factor of 2 wasn't enough! It gave us an answer of 1009,
        // which was too high. Bumping this up to a factor of 10 gave us the
        // correct answer of 986. Oof.
        let bound = Coordinate { x: target.x * 10, y: target.y * 10 };
        let regions = vec![vec![None; bound.x + 1]; bound.y + 1];
        CaveScanner { depth, target, bound, regions }
    }

    fn scan(&mut self) {
        self.regions[0][0] = Some(Region::new(self.depth, 0));
        self.regions[self.target.y][self.target.x] =
            Some(Region::new(self.depth, 0));
        for x in 0..=self.bound.x {
            self.regions[0][x] = Some(Region::new(self.depth, x * 16_807));
        }
        for y in 0..=self.bound.y {
            self.regions[y][0] = Some(Region::new(self.depth, y * 48_271));
        }
        for y in 1..=self.bound.y {
            for x in 1..=self.bound.x {
                if x == self.target.x && y == self.target.y {
                    continue;
                }

                // These unwraps are OK because we are guaranteed to have
                // computed the region for left and above in a prior iteration.
                let left = self.regions[y][x-1].as_ref().unwrap();
                let above = self.regions[y-1][x].as_ref().unwrap();
                let geologic_index = left.erosion_level * above.erosion_level;
                let region = Region::new(self.depth, geologic_index);
                self.regions[y][x] = Some(region);
            }
        }
    }

    fn cave(&self) -> Result<Cave> {
        let mut cave = Cave {
            depth: self.depth,
            target: self.target,
            bound: self.bound,
            regions: vec![],
        };
        for y in 0..=self.bound.y {
            let mut row = vec![];
            for x in 0..=self.bound.x {
                let region = match self.regions[y][x].clone() {
                    None => return err!("unknown region at ({}, {})", x, y),
                    Some(region) => region,
                };
                row.push(region);
            }
            cave.regions.push(row);
        }
        Ok(cave)
    }
}

#[derive(Clone, Debug)]
struct Region {
    typ: RegionType,
    geologic_index: usize,
    erosion_level: usize,
}

#[derive(Clone, Copy, Debug)]
enum RegionType {
    Rocky,
    Wet,
    Narrow,
}

impl Region {
    fn new(cave_depth: usize, geologic_index: usize) -> Region {
        let erosion_level = (geologic_index + cave_depth) % 20183;
        let typ = match erosion_level % 3 {
            0 => RegionType::Rocky,
            1 => RegionType::Wet,
            2 => RegionType::Narrow,
            _ => unreachable!(),
        };
        Region { typ, geologic_index, erosion_level }
    }

    fn risk_level(&self) -> usize {
        match self.typ {
            RegionType::Rocky => 0,
            RegionType::Wet => 1,
            RegionType::Narrow => 2,
        }
    }

    fn can_equip(&self, equip: Equip) -> bool {
        use self::RegionType::*;
        use self::Equip::*;

        match (self.typ, equip) {
            (Rocky, Torch) | (Rocky, Gear) => true,
            (Wet, Gear) | (Wet, Neither) => true,
            (Narrow, Torch) | (Narrow, Neither) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
enum Equip {
    Torch,
    Gear,
    Neither,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct Coordinate {
    x: usize,
    y: usize,
}
