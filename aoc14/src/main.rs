use std::error::Error;
use std::io::{self, Write};
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    part1(110201)?;
    part2(&[1, 1, 0, 2, 0, 1])?;
    Ok(())
}

fn part1(recipe_count: usize) -> Result<()> {
    let mut recipes = Recipes::new();
    while recipes.scores.len() < recipe_count + 10 {
        recipes.step();
    }

    let scores = recipes.scores[recipe_count..recipe_count+10]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .concat();
    writeln!(io::stdout(), "scores of next ten recipes: {}", scores)?;
    Ok(())
}

fn part2(digits: &[u32]) -> Result<()> {
    let mut recipes = Recipes::new();
    let ends_at;
    loop {
        if recipes.scores.ends_with(&digits) {
            ends_at = recipes.scores.len() - digits.len();
            break;
        } else if recipes.scores[..recipes.scores.len()-1].ends_with(&digits) {
            ends_at = recipes.scores.len() - digits.len() - 1;
            break;
        }
        recipes.step();
    }

    writeln!(io::stdout(), "recipes to the left: {}", ends_at)?;
    Ok(())
}

#[derive(Clone, Debug)]
struct Recipes {
    elves: Vec<usize>,
    scores: Vec<u32>,
}

impl Recipes {
    fn new() -> Recipes {
        Recipes { scores: vec![3, 7], elves: vec![0, 1] }
    }

    fn step(&mut self) {
        let new_recipe: u32 = self.elves
            .iter()
            .map(|&e| self.scores[e])
            .sum();
        for &digit in new_recipe.to_string().as_bytes() {
            let digit_value = digit - b'0';
            self.scores.push(digit_value as u32);
        }
        for e in &mut self.elves {
            *e = (*e + self.scores[*e] as usize + 1) % self.scores.len();
        }
    }
}
