#![allow(warnings)]

use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Read, Write};
use std::result;
use std::str::{self, FromStr};

use lazy_static::lazy_static;
use rand::Rng;
use rand::seq::SliceRandom;
use regex::Regex;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let bots: Bots = input.parse()?;

    let largest = bots.largest_radius();
    let in_range = bots.in_range_of_bot(&largest);
    writeln!(io::stdout(), "nanobots in range: {}", in_range)?;

    // The solution to part 2 is very dissatisfying. We use a cobbled
    // together version of simulated annealing and combine it with a somewhat
    // intelligent initial sample of points. Specifically, for each bot, we
    // sample points along the edge of its sphere of influence. The thinking
    // here is that optimal coordinate is probably close to the edge of at
    // least one sphere.
    //
    // Running this program does not guarantee the correct answer each time,
    // and it probably takes too long to run to completion anyway. I guessed
    // a few numbers as the distance appeared to stabilize and eventually got
    // it right.
    //
    // guessed: 111_851_609 (too low), 832 in range
    // 111_789_973 also has 832 in range.
    // 111_770_929 also has 832 in range.
    // guessed: 118_995_681
    // guessed: 121_493_970 (853 in range)
    // guessed: 121_493_971 (correct)
    let best = search(&bots);
    writeln!(io::stdout(), "BEST: {:?}", best);
    let dist = Coordinate::origin().distance(&best);
    writeln!(io::stdout(), "shortest distance: {}", dist)?;
    Ok(())
}

fn search(bots: &Bots) -> Coordinate {
    const INIT_TEMPERATURE: f64 = 1_000.0;
    const COOLING_FACTOR: f64 = 0.9999;
    const ITERS: usize = 1_000;

    fn prob(iter: usize, in_range_old: u64, in_range_new: u64) -> f64 {
        let temp = COOLING_FACTOR.powi(iter as i32) * INIT_TEMPERATURE;
        ((in_range_new as f64 - in_range_old as f64) / temp).exp()
    }

    let mut rng = rand::thread_rng();
    let mut origins = vec![];
    for bot in bots.bots.iter() {
        for _ in 0..10000 {
            origins.push(bot.random_surface_coordinate(&mut rng));
        }
    }
    origins.shuffle(&mut rng);

    let mut best_in_range = bots.in_range(&origins[0]);
    let mut best: HashSet<Coordinate> = HashSet::new();
    best.insert(origins[0]);

    for (i, &o) in origins.iter().enumerate() {
        let mut cur_pos = o;
        let mut cur_in_range = bots.in_range(&cur_pos);

        for i in 0..ITERS {
            let new_pos = cur_pos.random_neighbor(&mut rng);
            let new_in_range = bots.in_range(&new_pos);
            let p = prob(i, cur_in_range, new_in_range);
            if p >= 1.0 || rng.gen_bool(p) {
                cur_pos = new_pos;
                cur_in_range = new_in_range;
            }
            if new_in_range == best_in_range {
                best.insert(new_pos);
            } else if new_in_range > best_in_range {
                best.clear();
                best.insert(new_pos);
                best_in_range = new_in_range;
            }
        }

        // print out progress
        if i % 100 == 0 {
            let zzz = best.iter()
                .cloned()
                .min_by_key(|c| Coordinate::origin().distance(&c))
                .unwrap();
            println!(
                "origin ({}/{}): {:?} => {:?} (in range: {}, dist: {})",
                i, origins.len(), o, zzz, best_in_range,
                Coordinate::origin().distance(&zzz),
            );
        }
    }
    best.iter()
        .cloned()
        .min_by_key(|c| Coordinate::origin().distance(&c))
        .unwrap()
}

#[derive(Clone, Debug)]
struct Bots {
    bots: Vec<Bot>,
}

impl Bots {
    fn largest_radius(&self) -> &Bot {
        self.bots
            .iter()
            .max_by_key(|b| b.radius)
            .unwrap()
    }

    fn in_range_of_bot(&self, bot: &Bot) -> u64 {
        self.bots.iter().filter(|b| bot.in_range_of_bot(b)).count() as u64
    }

    fn in_range(&self, c: &Coordinate) -> u64 {
        self.bots.iter().filter(|b| b.in_range(c)).count() as u64
    }

    fn total_dist(&self, c: &Coordinate) -> i64 {
        self.bots.iter().map(|b| b.pos.distance(c) as i64).sum()
    }
}

impl FromStr for Bots {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Bots> {
        let mut bots: Vec<Bot> = vec![];
        for line in s.lines() {
            let bot = line.parse().or_else(|err| {
                err!("failed to parse '{:?}': {}", line, err)
            })?;
            bots.push(bot);
        }
        if bots.is_empty() {
            return err!("found no bots in input");
        }
        Ok(Bots { bots })
    }
}

#[derive(Clone, Debug)]
struct Bot {
    pos: Coordinate,
    radius: i64,
}

impl Bot {
    fn in_range_of_bot(&self, other: &Bot) -> bool {
        self.pos.distance(&other.pos) <= self.radius
    }

    fn in_range(&self, c: &Coordinate) -> bool {
        self.pos.distance(c) <= self.radius
    }

    fn random_surface_coordinate<R: Rng>(&self, mut rng: R) -> Coordinate {
        loop {
            let (x, y, z): (f64, f64, f64) = rng.gen();
            if x == 0.0 && y == 0.0 && z == 0.0 {
                continue;
            }
            let normal = 1.0 / (x*x + y*y + z*z).sqrt();
            let (x, y, z) = (x * normal, y * normal, z * normal);
            let radius = self.radius as f64;
            return Coordinate {
                x: (x * radius) as i32,
                y: (y * radius) as i32,
                z: (z * radius) as i32,
            };
        }
    }
}

impl FromStr for Bot {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Bot> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                pos=<(?P<x>-?[0-9]+),(?P<y>-?[0-9]+),(?P<z>-?[0-9]+)>,
                \s
                r=(?P<radius>[0-9]+)
            ").unwrap();
        }

        let caps = match RE.captures(s) {
            None => return err!("unrecognized position/radius"),
            Some(caps) => caps,
        };
        let pos = Coordinate {
            x: caps["x"].parse()?,
            y: caps["y"].parse()?,
            z: caps["z"].parse()?,
        };
        let radius = caps["radius"].parse()?;
        Ok(Bot { pos, radius })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32,
}

impl Coordinate {
    fn origin() -> Coordinate {
        Coordinate { x: 0, y: 0, z: 0 }
    }

    fn distance(&self, other: &Coordinate) -> i64 {
        (self.x as i64 - other.x as i64).abs()
        + (self.y as i64 - other.y as i64).abs()
        + (self.z as i64 - other.z as i64).abs()
    }

    fn random<R: Rng>(mut rng: R) -> Coordinate {
        Coordinate { x: rng.gen(), y: rng.gen(), z: rng.gen() }
    }

    fn random_neighbor<R: Rng>(&self, mut rng: R) -> Coordinate {
        // The commented out lines are for the test input, which has a
        // considerably smaller grid.
        // let dx = rng.gen_range(-1, 2);
        // let dy = rng.gen_range(-1, 2);
        // let dz = rng.gen_range(-1, 2);
        let dx = rng.gen_range(-10_000, 10_000);
        let dy = rng.gen_range(-10_000, 10_000);
        let dz = rng.gen_range(-10_000, 10_000);
        Coordinate { x: self.x + dx, y: self.y + dy, z: self.z + dz }
    }
}
