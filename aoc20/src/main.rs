use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Read, Write};
use std::result;

use regex_syntax::ParserBuilder;
use regex_syntax::hir::{self, Hir, HirKind};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let expr = ParserBuilder::new()
        .nest_limit(1000)
        .build()
        .parse(input.trim())?;

    let mut dists = Distances::new();
    let origin = Coordinate { x: 0, y: 0 };
    dists.insert(origin, 0);
    distances(&expr, &mut dists, origin)?;

    let largest = dists.values().max().unwrap();
    writeln!(io::stdout(), "largest number of doors: {}", largest)?;
    let atleast = dists.values().filter(|&&d| d >= 1000).count();
    writeln!(io::stdout(), "pass through at least 1000 doors: {}", atleast)?;
    Ok(())
}

type Distances = HashMap<Coordinate, usize>;

fn distances(
    expr: &Hir,
    dists: &mut Distances,
    c: Coordinate,
) -> Result<Coordinate> {
    match *expr.kind() {
        | HirKind::Empty
        | HirKind::Literal(hir::Literal::Byte(_))
        | HirKind::Class(_)
        | HirKind::Anchor(_)
        | HirKind::WordBoundary(_)
        | HirKind::Repetition(_) => Ok(c),
        HirKind::Literal(hir::Literal::Unicode(ch)) => {
            let nextc = c.mv(ch)?;
            let mut dist = dists[&c] + 1;
            if dists.contains_key(&nextc) {
                dist = cmp::min(dist, dists[&nextc])
            }
            dists.insert(nextc, dist);
            Ok(nextc)
        }
        HirKind::Group(ref g) => {
            distances(&g.hir, dists, c)
        }
        HirKind::Concat(ref exprs) => {
            let mut nextc = c;
            for e in exprs {
                nextc = distances(e, dists, nextc)?;
            }
            Ok(nextc)
        }
        HirKind::Alternation(ref exprs) => {
            for e in exprs {
                distances(e, dists, c)?;
            }
            Ok(c)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Coordinate {
    x: i64,
    y: i64,
}

impl Coordinate {
    fn mv(self, direction: char) -> Result<Coordinate> {
        match direction {
            'N' => Ok(Coordinate { x: self.x, y: self.y - 1 }),
            'S' => Ok(Coordinate { x: self.x, y: self.y + 1 }),
            'W' => Ok(Coordinate { x: self.x - 1, y: self.y }),
            'E' => Ok(Coordinate { x: self.x + 1, y: self.y }),
            _ => err!("unknown direction: {:?}", direction),
        }
    }
}
