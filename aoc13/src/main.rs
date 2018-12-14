use std::cmp;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::mem;
use std::result;
use std::str::FromStr;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut transport: Transport = input.parse()?;
    if transport.carts.is_empty() {
        return err!("found no carts in input");
    }
    loop {
        let crashes = transport.step()?;
        if !crashes.is_empty() {
            let c = crashes[0];
            writeln!(io::stdout(), "first crash at: {},{}", c.x, c.y)?;
            break;
        }
    }
    loop {
        transport.step()?;
        let uncrashed = transport.uncrashed();
        if uncrashed.is_empty() {
            writeln!(io::stdout(), "mutually assured destruction")?;
            break;
        }
        if uncrashed.len() == 1 {
            let c = uncrashed[0];
            writeln!(io::stdout(), "last cart standing at: {},{}", c.x, c.y)?;
            break;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl Coordinate {
    fn up(self) -> Result<Coordinate> {
        if self.y == 0 {
            err!("cannot move up")
        } else {
            Ok(Coordinate { y: self.y - 1, ..self })
        }
    }

    fn down(self) -> Result<Coordinate> {
        Ok(Coordinate { y: self.y + 1, ..self })
    }

    fn left(self) -> Result<Coordinate> {
        if self.x == 0 {
            err!("cannot move left")
        } else {
            Ok(Coordinate { x: self.x - 1, ..self })
        }
    }

    fn right(self) -> Result<Coordinate> {
        Ok(Coordinate { x: self.x + 1, ..self })
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

#[derive(Clone)]
struct Transport {
    carts: BTreeMap<Coordinate, Cart>,
    grid: Grid,
}

impl Transport {
    fn new() -> Transport {
        Transport { carts: BTreeMap::new(), grid: Grid::new() }
    }

    fn step(&mut self) -> Result<Vec<Coordinate>> {
        let mut crashes = HashSet::new();
        let mut previous_carts = mem::replace(
            &mut self.carts,
            BTreeMap::new(),
        );
        for (c, cart) in previous_carts.clone() {
            if crashes.contains(&c) {
                continue;
            }

            let (next_cart, next_c) = self.grid.step(cart, c)?;
            assert!(!cart.is_crashed());
            assert!(!next_cart.is_crashed());

            if previous_carts.contains_key(&next_c)
                || self.carts.contains_key(&next_c)
            {
                self.carts.remove(&next_c);
                crashes.insert(next_c);
            } else {
                assert!(!self.carts.contains_key(&next_c));
                self.carts.insert(next_c, next_cart);
            }
            previous_carts.remove(&c);
        }
        Ok(crashes.into_iter().collect())
    }

    fn uncrashed(&self) -> Vec<Coordinate> {
        self.carts
            .iter()
            .filter(|&(_, cart)| !cart.is_crashed())
            .map(|(&c, _)| c)
            .collect()
    }
}

impl FromStr for Transport {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Transport> {
        if !s.is_ascii() {
            return err!("expected initial transport grid to be ASCII");
        }

        let mut trans = Transport::new();
        for (y, line) in s.lines().enumerate() {
            for x in line.char_indices().map(|(i, _)| i) {
                let c = Coordinate { x, y };
                let cell = &line[x..x+1];
                if !"<>^v".contains(cell) {
                    trans.grid.set(c, cell.parse()?);
                    continue;
                }
                let cart = cell.parse()?;
                trans.carts.insert(c, cart);
                trans.grid.set(c, cart.initial_track()?);
            }
        }
        Ok(trans)
    }
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..=self.grid.max_y {
            for x in 0..=self.grid.max_x {
                let c = Coordinate { x, y };
                if let Some(&cart) = self.carts.get(&c) {
                    write!(f, "{:?}", cart)?;
                } else {
                    write!(f, "{:?}", self.grid.get(c))?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Cart {
    intersections: usize,
    kind: CartKind,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CartKind {
    Up,
    Down,
    Left,
    Right,
    Crashed,
}

impl Cart {
    fn initial_track(&self) -> Result<Track> {
        match self.kind {
            CartKind::Up | CartKind::Down => Ok(Track::Vertical),
            CartKind::Left | CartKind::Right => Ok(Track::Horizontal),
            CartKind::Crashed => err!("unknown track for crashed cart"),
        }
    }

    fn is_crashed(&self) -> bool {
        self.kind == CartKind::Crashed
    }

    fn direction(self, kind: CartKind) -> Cart {
        Cart { kind, ..self }
    }

    fn intersection(mut self) -> Cart {
        let which = self.intersections % 3;
        self.intersections += 1;
        match which {
            0 => self.turn_left(),
            1 => self,
            2 => self.turn_right(),
            _ => unreachable!(),
        }
    }

    fn turn_left(self) -> Cart {
        use self::CartKind::*;

        let kind = match self.kind {
            Up => Left,
            Down => Right,
            Left => Down,
            Right => Up,
            Crashed => Crashed,
        };
        Cart { kind, ..self }
    }

    fn turn_right(self) -> Cart {
        use self::CartKind::*;

        let kind = match self.kind {
            Up => Right,
            Down => Left,
            Left => Up,
            Right => Down,
            Crashed => Crashed,
        };
        Cart { kind, ..self }
    }
}

impl FromStr for Cart {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Cart> {
        let kind = match s.as_bytes().get(0) {
            None => return err!("no cart available in empty string"),
            Some(&b'^') => CartKind::Up,
            Some(&b'v') => CartKind::Down,
            Some(&b'<') => CartKind::Left,
            Some(&b'>') => CartKind::Right,
            Some(&b'X') => CartKind::Crashed,
            Some(&b) => return err!("unrecognized cart: 0x{:X}", b),
        };
        Ok(Cart { intersections: 0, kind })
    }
}

impl fmt::Debug for Cart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            CartKind::Up => write!(f, "^"),
            CartKind::Down => write!(f, "v"),
            CartKind::Left => write!(f, "<"),
            CartKind::Right => write!(f, ">"),
            CartKind::Crashed => write!(f, "X"),
        }
    }
}

#[derive(Clone)]
struct Grid {
    tracks: HashMap<Coordinate, Track>,
    max_x: usize,
    max_y: usize,
}

impl Grid {
    fn new() -> Grid {
        Grid { tracks: HashMap::new(), max_x: 0, max_y: 0 }
    }

    fn get(&self, c: Coordinate) -> Track {
        self.tracks.get(&c).map(|&c| c).unwrap_or(Track::Empty)
    }

    fn set(&mut self, c: Coordinate, track: Track) {
        self.tracks.insert(c, track);
        self.max_x = cmp::max(self.max_x, c.x);
        self.max_y = cmp::max(self.max_y, c.y);
    }

    // /// Given a cart and its position in the grid, return the next position
    // /// for the cart.
    fn step(
        &self,
        mut cart: Cart,
        c: Coordinate,
    ) -> Result<(Cart, Coordinate)> {
        use self::CartKind::*;
        use self::Track::*;

        let next_coord = match (cart.kind, self.get(c)) {
            (_, Empty) => return err!("invalid transition on empty"),
            (Crashed, _) => c,
            (Up, Horizontal) => return err!("cannot go up on horizontal"),
            (Up, _) => c.up()?,
            (Down, Horizontal) => return err!("cannot go down on horizontal"),
            (Down, _) => c.down()?,
            (Left, Vertical) => return err!("cannot go left on vertical"),
            (Left, _) => c.left()?,
            (Right, Vertical) => return err!("cannot go right on vertical"),
            (Right, _) => c.right()?,
        };
        cart = match (cart.kind, self.get(next_coord)) {
            (_, Empty) => return err!("cannot move to empty coordinate"),
            (Crashed, _) => cart,
            (Up, Vertical) => cart.direction(Up),
            (Up, Horizontal) => cart.direction(Up),
            (Up, Intersection) => cart.intersection(),
            (Up, CurveForward) => cart.direction(Right),
            (Up, CurveBackward) => cart.direction(Left),
            (Down, Vertical) => cart.direction(Down),
            (Down, Horizontal) => cart.direction(Down),
            (Down, Intersection) => cart.intersection(),
            (Down, CurveForward) => cart.direction(Left),
            (Down, CurveBackward) => cart.direction(Right),
            (Left, Vertical) => cart.direction(Left),
            (Left, Horizontal) => cart.direction(Left),
            (Left, Intersection) => cart.intersection(),
            (Left, CurveForward) => cart.direction(Down),
            (Left, CurveBackward) => cart.direction(Up),
            (Right, Vertical) => cart.direction(Right),
            (Right, Horizontal) => cart.direction(Right),
            (Right, Intersection) => cart.intersection(),
            (Right, CurveForward) => cart.direction(Up),
            (Right, CurveBackward) => cart.direction(Down),
        };
        Ok((cart, next_coord))
    }
}

#[derive(Clone, Copy)]
enum Track {
    Empty,
    Vertical,
    Horizontal,
    Intersection,
    CurveForward,
    CurveBackward,
}

impl FromStr for Track {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Track> {
        match s.as_bytes().get(0) {
            None => err!("no track available in empty string"),
            Some(&b' ') => Ok(Track::Empty),
            Some(&b'|') => Ok(Track::Vertical),
            Some(&b'-') => Ok(Track::Horizontal),
            Some(&b'+') => Ok(Track::Intersection),
            Some(&b'/') => Ok(Track::CurveForward),
            Some(&b'\\') => Ok(Track::CurveBackward),
            Some(&b) => err!("unrecognized track: 0x{:X}", b),
        }
    }
}

impl fmt::Debug for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..=self.max_y {
            for x in 0..=self.max_x {
                write!(f, "{:?}", self.get(Coordinate { x, y }))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Track::Empty => write!(f, " "),
            Track::Vertical => write!(f, "|"),
            Track::Horizontal => write!(f, "-"),
            Track::Intersection => write!(f, "+"),
            Track::CurveForward => write!(f, "/"),
            Track::CurveBackward => write!(f, "\\"),
        }
    }
}
