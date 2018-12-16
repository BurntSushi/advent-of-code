#![allow(dead_code)]

use std::cmp;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::result;
use std::str::FromStr;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let caves: Caves = input.parse()?;

    writeln!(io::stdout(), "part 1, outcome: {}", caves.clone().outcome()?)?;

    for power in 4..100 {
        let mut caves = caves.clone();
        caves.set_elf_attack_power(power);

        let initial_elves = caves.remaining_elves();
        let outcome = caves.outcome()?;
        if initial_elves == caves.remaining_elves() {
            writeln!(
                io::stdout(),
                "part 2, elves at power {}, outcome: {}",
                power, outcome,
            )?;
            break;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct Caves {
    grid: BTreeMap<Coordinate, Cell>,
    units: BTreeMap<Coordinate, Unit>,
    max: Coordinate,
}

#[derive(Clone, Debug)]
enum Cell {
    Wall,
    Open,
}

impl Caves {
    fn outcome(&mut self) -> Result<usize> {
        const LIMIT: usize = 500;

        for i in 0..LIMIT {
            let moved = self.step();
            if !moved {
                let hp = self.hp();
                return Ok(i * hp);
            }
        }
        err!("no outcome after {} iterations", LIMIT)
    }

    fn remaining_elves(&self) -> usize {
        self.units.values().filter(|u| u.is_elf()).count()
    }

    fn set_elf_attack_power(&mut self, power: usize) {
        for unit in self.units.values_mut() {
            if unit.is_elf() {
                unit.attack = power;
            }
        }
    }

    fn hp(&self) -> usize {
        self.units.values().map(|u| u.hp).sum()
    }

    fn step(&mut self) -> bool {
        let mut any_move = false;
        let unit_coordinates: Vec<_> = self.units.keys().cloned().collect();
        for c in unit_coordinates.into_iter() {
            if !self.units.contains_key(&c) {
                continue;
            }
            if !self.any_enemies(c) {
                return false;
            }
            if let Some(attack) = self.best_attack_unit(c) {
                self.attack(c, attack);
                any_move = true;
                continue;
            }

            let nextc = match self.next_step(c) {
                None => continue,
                Some(nextc) => nextc,
            };
            any_move = true;

            let unit = self.units.remove(&c).unwrap();
            self.units.insert(nextc, unit);
            if let Some(attack) = self.best_attack_unit(nextc) {
                self.attack(nextc, attack);
            }
        }
        any_move
    }

    fn next_step(&self, unit: Coordinate) -> Option<Coordinate> {
        self.nearest_target(unit).and_then(|t| self.nearest_step(unit, t))
    }

    fn nearest_step(
        &self,
        unit: Coordinate,
        target: Coordinate,
    ) -> Option<Coordinate> {
        let dists = self.distances(target);
        self.neighbors(unit)
            .filter_map(|c| dists.get(&c).map(|dist| (c, dist)))
            .min_by_key(|&(_, dist)| dist)
            .map(|(c, _)| c)
    }

    fn nearest_target(&self, unit: Coordinate) -> Option<Coordinate> {
        let dists = self.distances(unit);
        self.targets(unit)
            .into_iter()
            .filter_map(|c| dists.get(&c).map(|dist| (c, dist)))
            .min_by_key(|&(_, dist)| dist)
            .map(|(c, _)| c)
    }

    fn distances(&self, origin: Coordinate) -> BTreeMap<Coordinate, usize> {
        let mut d = BTreeMap::new();
        d.insert(origin, 0);

        // let mut todo = vec![origin];
        let mut todo = VecDeque::new();
        todo.push_front(origin);
        let mut todo_set = BTreeSet::new();
        let mut visited = BTreeSet::new();
        while let Some(c) = todo.pop_front() {
            visited.insert(c);
            todo_set.remove(&c);
            for neighbor in self.neighbors(c) {
                if visited.contains(&neighbor) {
                    continue;
                }
                if !todo_set.contains(&neighbor) {
                    todo.push_back(neighbor);
                    todo_set.insert(neighbor);
                }

                let candidate_dist = 1 + *d.get(&c).unwrap_or(&0);
                if !d.contains_key(&neighbor) || candidate_dist < d[&neighbor]
                {
                    d.insert(neighbor, candidate_dist);
                }
            }
        }
        d
    }

    fn targets(&self, origin: Coordinate) -> BTreeSet<Coordinate> {
        let unit = &self.units[&origin];
        let mut targets = BTreeSet::new();
        for (&c, candidate) in &self.units {
            if unit.is_enemy(candidate) {
                targets.extend(self.neighbors(c));
            }
        }
        targets
    }

    fn any_enemies(&self, unit: Coordinate) -> bool {
        for candidate in self.units.values() {
            if self.units[&unit].is_enemy(candidate) {
                return true;
            }
        }
        false
    }

    fn attack(&mut self, attacker: Coordinate, victim: Coordinate) {
        let power = self.units[&attacker].attack;
        if self.units.get_mut(&victim).unwrap().absorb(power) {
            self.units.remove(&victim);
        }
    }

    fn best_attack_unit(&self, c: Coordinate) -> Option<Coordinate> {
        let unit = &self.units[&c];
        c.neighbors(self.max)
            .into_iter()
            .filter(|c| self.units.contains_key(c))
            .filter(|c| unit.is_enemy(&self.units[c]))
            .min_by_key(|c| (self.units[c].hp, *c))
    }

    fn neighbors<'a>(
        &'a self,
        origin: Coordinate,
    ) -> impl Iterator<Item=Coordinate> + 'a {
        origin
            .neighbors(self.max)
            .into_iter()
            .filter(move |&c| self.is_open(c))
    }

    fn is_open(&self, c: Coordinate) -> bool {
        !self.units.contains_key(&c) && self.grid[&c].is_open()
    }
}

impl Cell {
    fn is_open(&self) -> bool {
        match *self {
            Cell::Open => true,
            Cell::Wall => false,
        }
    }
}

impl FromStr for Caves {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Caves> {
        if !s.is_ascii() {
            return err!("only ASCII caves are supported");
        }

        let mut caves = Caves::default();
        caves.max.x = s.lines().next().unwrap_or("").len() - 1;
        caves.max.y = s.lines().count() - 1;
        if !s.lines().all(|line| line.len() == caves.max.x + 1) {
            return err!("all lines in input must have the same length");
        }

        for (y, line) in s.lines().enumerate() {
            for x in line.char_indices().map(|(x, _)| x) {
                let c = Coordinate { x, y };
                let cell = &line[x..x+1];
                if ["E", "G"].contains(&cell) {
                    let unit = cell.parse()?;
                    caves.grid.insert(c, Cell::Open);
                    caves.units.insert(c, unit);
                } else {
                    caves.grid.insert(c, cell.parse()?);
                }
            }
        }
        Ok(caves)
    }
}

impl FromStr for Cell {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Cell> {
        match s.as_bytes().get(0) {
            None => err!("cannot deserialize empty string into cell"),
            Some(&b'#') => Ok(Cell::Wall),
            Some(&b'.') => Ok(Cell::Open),
            Some(&b) => err!("unrecognized cell: 0x{:X}", b),
        }
    }
}

impl fmt::Display for Caves {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (c, cell) in &self.grid {
            if let Some(ref unit) = self.units.get(c) {
                write!(f, "{}", unit)?;
            } else {
                write!(f, "{}", cell)?;
            }
            if c.x == self.max.x {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Cell::Wall => write!(f, "#"),
            Cell::Open => write!(f, "."),
        }
    }
}

#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl Coordinate {
    fn with_x(self, x: usize) -> Coordinate {
        Coordinate { x, ..self }
    }

    fn with_y(self, y: usize) -> Coordinate {
        Coordinate { y, ..self }
    }

    fn distance(&self, other: Coordinate) -> usize {
        let x = (self.x as isize - other.x as isize).abs();
        let y = (self.y as isize - other.y as isize).abs();
        (x + y) as usize
    }

    fn neighbors(self, max: Coordinate) -> Vec<Coordinate> {
        assert!(self <= max, "{:?} should be <= than the max {:?}", self, max);

        let mut coords = vec![];
        if self.y >= 1 {
            coords.push(self.with_y(self.y - 1));
        }
        if self.x >= 1 {
            coords.push(self.with_x(self.x - 1));
        }
        if self.x + 1 <= max.x {
            coords.push(self.with_x(self.x + 1));
        }
        if self.y + 1 <= max.y {
            coords.push(self.with_y(self.y + 1));
        }
        coords
    }
}

impl fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Coordinate) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Coordinate) -> Option<cmp::Ordering> {
        Some((self.y, self.x).cmp(&(other.y, other.x)))
    }
}

#[derive(Clone, Debug)]
struct Unit {
    attack: usize,
    hp: usize,
    kind: UnitKind,
}

#[derive(Clone, Copy, Debug)]
enum UnitKind {
    Elf,
    Goblin,
}

impl Unit {
    fn is_enemy(&self, candidate: &Unit) -> bool {
        match (self.kind, candidate.kind) {
            (UnitKind::Elf, UnitKind::Goblin) => true,
            (UnitKind::Goblin, UnitKind::Elf) => true,
            _ => false,
        }
    }

    fn is_elf(&self) -> bool {
        match self.kind {
            UnitKind::Elf => true,
            UnitKind::Goblin => false,
        }
    }

    fn absorb(&mut self, power: usize) -> bool {
        self.hp = self.hp.saturating_sub(power);
        self.is_dead()
    }

    fn is_dead(&self) -> bool {
        self.hp == 0
    }
}

impl FromStr for Unit {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Unit> {
        let kind = match s.as_bytes().get(0) {
            None => return err!("cannot deserialize empty string into unit"),
            Some(&b'E') => UnitKind::Elf,
            Some(&b'G') => UnitKind::Goblin,
            Some(&b) => return err!("unrecognized unit kind: 0x{:X}", b),
        };
        Ok(Unit { attack: 3, hp: 200, kind })
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            UnitKind::Elf => write!(f, "E"),
            UnitKind::Goblin => write!(f, "G"),
        }
    }
}
