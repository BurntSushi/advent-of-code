use std::io::{self, Read, Write};
use std::mem;

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input = input.trim();

    part1(input)?;
    part2(input)?;
    Ok(())
}

fn part1(polymer: &str) -> Result<()> {
    writeln!(io::stdout(), "inert length: {}", react(polymer).len())?;
    Ok(())
}

fn part2(polymer: &str) -> Result<()> {
    let mut best = polymer.len();
    for b in b'A'..=b'Z' {
        let unit1 = b as char;
        let unit2 = (b + 32) as char;
        let cleansed = polymer.replace(unit1, "").replace(unit2, "");
        let reacted = react(&cleansed);
        if reacted.len() < best {
            best = reacted.len();
        }
    }
    writeln!(io::stdout(), "best inert length: {}", best)?;
    Ok(())
}

/// Reacts the given polymer and returns the final inert form.
fn react(polymer_string: &str) -> String {
    let mut polymer = polymer_string.as_bytes().to_vec();
    let mut reacted_polymer = vec![];
    loop {
        let mut reacted = false;
        let mut i = 1;
        while i < polymer.len() {
            if reacts(polymer[i-1], polymer[i]) {
                reacted = true;
                i += 2;
                continue;
            }
            reacted_polymer.push(polymer[i-1]);
            i += 1;
        }
        if i == polymer.len() {
            reacted_polymer.push(polymer[i-1]);
        }

        mem::swap(&mut polymer, &mut reacted_polymer);
        reacted_polymer.clear();
        if !reacted {
            break;
        }
    }
    // We only ever remove ASCII bytes, which is guaranteed to preserve the
    // UTF-8 validity of `polymer`.
    String::from_utf8(polymer).unwrap()
}

/// Returns true if and only if the given bytes correspond to types that
/// react with one another.
fn reacts(b1: u8, b2: u8) -> bool {
    if b1 < b2 {
        b2 - b1 == 32
    } else {
        b1 - b2 == 32
    }
}
