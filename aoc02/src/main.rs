use std::io::{self, Read, Write};

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    part1(&input)?;
    part2(&input)?;
    Ok(())
}

fn part1(input: &str) -> Result<()> {
    let mut frequencies = [0u8; 256];
    let (mut twos, mut threes) = (0, 0);
    for line in input.lines() {
        if !line.is_ascii() {
            return Err(From::from("part1 only supports ASCII"));
        }

        for f in frequencies.iter_mut() {
            *f = 0;
        }
        for b in line.as_bytes().iter().map(|&b| b as usize) {
            frequencies[b] = frequencies[b].saturating_add(1);
        }
        if frequencies.iter().any(|&f| f == 2) {
            twos += 1;
        }
        if frequencies.iter().any(|&f| f == 3) {
            threes += 1;
        }
    }
    writeln!(io::stdout(), "{}", twos * threes)?;
    Ok(())
}

fn part2(input: &str) -> Result<()> {
    let ids: Vec<&str> = input.lines().collect();
    for i in 0..ids.len() {
        for j in i+1..ids.len() {
            if let Some(common) = common_correct_letters(&ids[i], &ids[j]) {
                writeln!(io::stdout(), "{}", common)?;
                return Ok(());
            }
        }
    }
    Err(From::from("could not find two correct box ids"))
}

fn common_correct_letters(id1: &str, id2: &str) -> Option<String> {
    if id1.len() != id2.len() {
        return None;
    }

    let mut found_one_wrong = false;
    for (c1, c2) in id1.chars().zip(id2.chars()) {
        if c1 != c2 {
            if found_one_wrong {
                return None;
            }
            found_one_wrong = true;
        }
    }
    Some(
        id1.chars().zip(id2.chars())
         .filter(|&(c1, c2)| c1 == c2)
         .map(|(c, _)| c)
         .collect()
    )
}
