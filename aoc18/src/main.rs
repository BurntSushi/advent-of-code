use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::mem;
use std::result;
use std::str::{self, FromStr};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let minutes = 10;
    let mut area: Area = input.parse()?;
    for _ in 0..minutes {
        area.step();
    }
    writeln!(
        io::stdout(),
        "resource value after {} minutes: {}",
        minutes,
        area.resource_value(),
    )?;

    // Doing 1000000000 will take way too long. Instead, print out resource
    // values at a lower number. It is easy to notice that it is periodic.
    // Specifically, it is periodic over 28 values. Namely,
    // 1_000_000_000 % 28 == 20. The period is active, at minimum, after 1000
    // minutes. Therefore, 1028 % 28 == 20 implies that the resource value
    // after 1028 minutes is the same as the resource value after 1_000_000_000
    // minutes.
    let minutes = 1028;
    let mut area: Area = input.parse()?;
    for _ in 0..minutes {
        area.step();
    }
    writeln!(
        io::stdout(),
        "resource value after {} minutes: {}",
        minutes,
        area.resource_value(),
    )?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coordinate {
    x: i64,
    y: i64,
}

#[derive(Clone, Debug)]
struct Area {
    acres: Vec<Vec<Acre>>,
    acres2: Vec<Vec<Acre>>,
}

impl Area {
    fn resource_value(&self) -> usize {
        let (mut wooded, mut lumber) = (0, 0);
        for row in &self.acres {
            for acre in row {
                match acre {
                    Acre::Open => {}
                    Acre::Trees => wooded += 1,
                    Acre::Lumberyard => lumber += 1,
                }
            }
        }
        wooded * lumber
    }

    // I foolishly tried to optimize the code below before realizing it was
    // futile and started looking for a pattern in the output. ---AG

    fn step(&mut self) {
        let mut new = mem::replace(&mut self.acres2, vec![]);
        for y in 0..self.height() {
            for x in 0..self.width() {
                self.step_cell(x, y, &mut new);
            }
        }
        self.acres2 = mem::replace(&mut self.acres, vec![]);
        self.acres = new;
    }

    fn step_cell(
        &self,
        x: usize,
        y: usize,
        new: &mut Vec<Vec<Acre>>,
    ) {
        use self::Acre::*;

        new[y][x] = self.acres[y][x];
        match self.acres[y][x] {
            Open => {
                let count = self.neighbors(
                    x, y, 0, |count, n| {
                        if n == Trees { count + 1 } else { count }
                    },
                );
                if count >= 3 {
                    new[y][x] = Trees;
                }
            }
            Trees => {
                let count = self.neighbors(
                    x, y, 0, |count, n| {
                        if n == Lumberyard { count + 1 } else { count }
                    },
                );
                if count >= 3 {
                    new[y][x] = Lumberyard;
                }
            }
            Lumberyard => {
                let (has_lumber, has_trees) = self.neighbors(
                    x, y, (false, false),
                    |(lumber, trees), n| {
                        (lumber || n == Lumberyard, trees || n == Trees)
                    },
                );
                if !has_lumber || !has_trees {
                    new[y][x] = Open;
                }
            }
        }
    }

    fn neighbors<T>(
        &self,
        ox: usize,
        oy: usize,
        init: T,
        mut f: impl FnMut(T, Acre) -> T,
    ) -> T {
        let mut ret = init;
        for y in oy.saturating_sub(1)..=oy.saturating_add(1) {
            for x in ox.saturating_sub(1)..=ox.saturating_add(1) {
                if x == ox && y == oy {
                    continue;
                }
                if x >= self.width() || y >= self.height() {
                    continue;
                }
                ret = f(ret, self.acres[y][x]);
            }
        }
        ret
    }

    fn width(&self) -> usize {
        self.acres[0].len()
    }

    fn height(&self) -> usize {
        self.acres.len()
    }
}

impl FromStr for Area {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Area> {
        if !s.is_ascii() {
            return err!("area must be in ASCII");
        }

        let ylen = s.lines().count();
        if ylen == 0 {
            return err!("area cannot be empty");
        }

        let xlen = s.lines().next().unwrap().len();
        let mut area = Area {
            acres: vec![vec![Acre::Open; xlen]; ylen],
            acres2: vec![vec![Acre::Open; xlen]; ylen],
        };
        for (y, line) in s.lines().enumerate() {
            if line.len() != xlen {
                return err!(
                    "all rows expected to have length {}, but found {}",
                    xlen, line.len()
                );
            }
            for x in 0..line.len() {
                area.acres[y][x] = line[x..x+1].parse()?;
            }
        }
        Ok(area)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Acre {
    Open,
    Trees,
    Lumberyard,
}

impl FromStr for Acre {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Acre> {
        match s.chars().next() {
            None => err!("cannot parse acre from empty string"),
            Some('.') => Ok(Acre::Open),
            Some('|') => Ok(Acre::Trees),
            Some('#') => Ok(Acre::Lumberyard),
            Some(c) => err!("invalid acre: '{}'", c),
        }
    }
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in &self.acres {
            for col in row {
                write!(f, "{}", col)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl fmt::Display for Acre {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Acre::Open => write!(f, "."),
            Acre::Trees => write!(f, "|"),
            Acre::Lumberyard => write!(f, "#"),
        }
    }
}
