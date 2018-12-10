use std::cmp;
use std::error::Error;
use std::io::{self, Read, Write};
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

    let mut points: Vec<Point> = vec![];
    for line in input.lines() {
        let point = line.parse().or_else(|err| {
            err!("failed to parse '{:?}': {}", line, err)
        })?;
        points.push(point);
    }
    if points.is_empty() {
        return err!("no points given");
    }
    let mut points = Points::new(points);

    for _ in 0..1_000_000 {
        points.step();
        let (w, h) = points.dimensions();
        if w <= 80 && h <= 80 {
            writeln!(io::stdout(), "seconds: {}", points.seconds)?;
            writeln!(io::stdout(), "{}", points.grid_string().trim())?;
            writeln!(io::stdout(), "{}", "~".repeat(79))?;
        }
    }
    writeln!(io::stdout(), "message should be in one of the above grids")?;
    Ok(())
}

#[derive(Clone, Debug)]
struct Points {
    points: Vec<Point>,
    seconds: u32,
}

impl Points {
    fn new(points: Vec<Point>) -> Points {
        assert!(!points.is_empty());
        Points { points, seconds: 0 }
    }

    fn step(&mut self) {
        for p in &mut self.points {
            p.x += p.vx;
            p.y += p.vy;
        }
        self.seconds += 1;
    }

    fn bounds(&self) -> Bounds {
        let mut b = Bounds {
            minx: self.points[0].x,
            maxx: self.points[0].x,
            miny: self.points[0].y,
            maxy: self.points[0].y,
        };
        for p in &self.points {
            b.minx = cmp::min(b.minx, p.x);
            b.maxx = cmp::max(b.maxx, p.x);
            b.miny = cmp::min(b.miny, p.y);
            b.maxy = cmp::max(b.maxy, p.y);
        }
        b
    }

    fn dimensions(&self) -> (usize, usize) {
        let b = self.bounds();
        (b.width(), b.height())
    }

    fn grid_string(&self) -> String {
        let bounds = self.bounds();
        let mut grid = vec![vec![b'.'; bounds.width()]; bounds.height()];
        for p in &self.points {
            let x = bounds.normal_x(p.x);
            let y = bounds.normal_y(p.y);
            grid[y as usize][x as usize] = b'#';
        }

        let mut buf = String::new();
        for row in grid {
            buf.push_str(str::from_utf8(&row).unwrap());
            buf.push('\n');
        }
        buf
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Bounds {
    minx: i32,
    maxx: i32,
    miny: i32,
    maxy: i32,
}

impl Bounds {
    fn normal_x(&self, x: i32) -> u32 {
        if self.minx >= 0 {
            (x - self.minx) as u32
        } else {
            (x + self.minx.abs()) as u32
        }
    }

    fn normal_y(&self, y: i32) -> u32 {
        if self.miny >= 0 {
            (y - self.miny) as u32
        } else {
            (y + self.miny.abs()) as u32
        }
    }

    fn width(&self) -> usize {
        (self.maxx - self.minx + 1) as usize
    }

    fn height(&self) -> usize {
        (self.maxy - self.miny + 1) as usize
    }
}

#[derive(Clone, Debug)]
struct Point {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
}

impl FromStr for Point {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Point> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                position=<\s*(?P<x>[-0-9]+),\s*(?P<y>[-0-9]+)>
                \s+
                velocity=<\s*(?P<vx>[-0-9]+),\s*(?P<vy>[-0-9]+)>
            ").unwrap();
        }

        let caps = match RE.captures(s) {
            None => return err!("unrecognized position/velocity"),
            Some(caps) => caps,
        };
        Ok(Point {
            x: caps["x"].parse()?,
            y: caps["y"].parse()?,
            vx: caps["vx"].parse()?,
            vy: caps["vy"].parse()?,
        })
    }
}
