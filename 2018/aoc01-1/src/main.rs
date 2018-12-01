use std::io::{self, BufRead, Write};

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn main() -> Result<()> {
    let mut freq = 0;
    let stdin = io::BufReader::new(io::stdin());
    for result in stdin.lines() {
        let line = result?;
        let change: i32 = line.parse()?;
        freq += change;
    }
    writeln!(io::stdout(), "{}", freq)?;
    Ok(())
}
