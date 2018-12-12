use std::cmp;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::result;
use std::str::FromStr;

use fnv::FnvHashMap as HashMap;
use lazy_static::lazy_static;
use regex::Regex;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let pots: Pots = input.parse()?;
    run(pots.clone(), 20)?;
    run(pots.clone(), 500)?;
    run(pots.clone(), 5_000)?;
    run(pots.clone(), 50_000)?;
    // After running the above, there is an obvious pattern. The result is
    // always 4x..x866 where x=0 and is repeated N-1 times where N is the
    // number of zeros in the generation count. 50_000_000_000 has 10 zeros,
    // which means our answer is 4000000000866.
    //
    // Given the implementation here, it would take about a month for it to run
    // over 50 billion generations. There's likely a more clever solution
    // that detects convergence of the pot states?
    Ok(())
}

fn run(mut pots: Pots, generations: usize) -> Result<()> {
    for i in 0..generations {
        pots = pots.step();
        if i % 100_000 == 0 {
            println!("gen: {}, min: {}, max: {}, size: {}",
                     i, pots.min, pots.max, pots.pots.len());
        }
    }
    writeln!(
        io::stdout(),
        "sum of pots with plants after {} generations: {}",
        generations, pots.sum_plant(),
    )?;
    Ok(())
}

#[derive(Clone)]
pub struct Pots {
    pots: HashMap<i32, Pot>,
    transitions: Vec<Transition>,
    min: i32,
    max: i32,
}

impl Pots {
    fn sum_plant(&self) -> i32 {
        self.pots
            .iter()
            .filter(|&(_, pot)| pot.has_plants())
            .map(|(&i, _)| i)
            .sum()
    }

    fn step(&self) -> Pots {
        let mut new = self.fresh();
        for &i in self.pots.keys() {
            for j in i-2..=i+2 {
                new.set_pot(j, self.next_state(&self.current_state(j)));
            }
        }
        new
    }

    fn fresh(&self) -> Pots {
        Pots {
            pots: HashMap::default(),
            transitions: self.transitions.clone(),
            min: self.min,
            max: self.max,
        }
    }

    fn next_state(&self, current: &[Pot]) -> Pot {
        for t in &self.transitions {
            if t.is_match(current) {
                return t.to;
            }
        }
        Pot::Empty
    }

    fn current_state(&self, at: i32) -> Vec<Pot> {
        let mut state = vec![];
        for i in at-2..=at+2 {
            state.push(self.pot(i));
        }
        state
    }

    fn pot(&self, i: i32) -> Pot {
        self.pots.get(&i).map(|&pot| pot).unwrap_or(Pot::Empty)
    }

    fn set_pot(&mut self, i: i32, pot: Pot) {
        if pot.has_plants() {
            self.min = cmp::min(self.min, i - 2);
            self.max = cmp::max(self.max, i + 2);
            self.pots.insert(i, pot);
        }
    }
}

impl FromStr for Pots {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Pots> {
        let mut lines = s.lines();
        let first = match lines.next() {
            None => return err!("empty input for pots"),
            Some(first) => first,
        };

        let prefix = "initial state: ";
        if !first.starts_with(prefix) {
            return err!("unexpected prefix for first line: {:?}", first);
        }
        let pots: HashMap<i32, Pot> = first[prefix.len()..]
            .char_indices()
            .map(|(i, _)| s[prefix.len() + i..].parse())
            .collect::<Result<Vec<Pot>>>()?
            .into_iter()
            .enumerate()
            .map(|(i, pot)| (i as i32, pot))
            .collect();

        match lines.next() {
            None => return err!("missing empty line separating transitions"),
            Some(second) => {
                if !second.is_empty() {
                    return err!("second line is not empty: {:?}", second);
                }
            }
        }

        let transitions = lines
            .map(|line| line.parse())
            .collect::<Result<Vec<Transition>>>()?
            // Drop transitions to empty pots.
            .into_iter()
            .filter(|t| t.to.has_plants())
            .collect::<Vec<Transition>>();

        let (min, max) = (-2, pots.len() as i32 + 2);
        Ok(Pots { pots, transitions, min, max })
    }
}

impl fmt::Debug for Pots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.min..=self.max {
            if self.pot(i).has_plants() {
                write!(f, "#")?;
            } else {
                write!(f, ".")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pot {
    Plants,
    Empty,
}

impl Pot {
    fn has_plants(&self) -> bool {
        *self == Pot::Plants
    }
}

impl FromStr for Pot {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Pot> {
        if s.is_empty() {
            err!("no pot in empty string")
        } else if &s[0..1] == "#" {
            Ok(Pot::Plants)
        } else if &s[0..1] == "." {
            Ok(Pot::Empty)
        } else {
            err!("unrecognized pot state: {:?}", s)
        }
    }
}

#[derive(Clone, Debug)]
struct Transition {
    from: Vec<Pot>,
    to: Pot,
}

impl Transition {
    fn is_match(&self, state: &[Pot]) -> bool {
        self.from == state
    }
}

impl FromStr for Transition {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Transition> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^(?P<from>[#.]{5}) => (?P<to>[#.])$",
            ).unwrap();
        }

        let caps = match RE.captures(s) {
            None => return err!("unrecognized transition"),
            Some(caps) => caps,
        };
        let from = caps["from"]
            .char_indices()
            .map(|(i, _)| s[i..].parse())
            .collect::<Result<Vec<Pot>>>()?;
        Ok(Transition {
            from: from,
            to: caps["to"].parse()?,
        })
    }
}
