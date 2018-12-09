use std::fmt;
use std::io::{self, Write};

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

fn main() -> Result<()> {
    part1()?;
    part2()?;
    Ok(())
}

fn part1() -> Result<()> {
    const PLAYERS: usize = 418;
    const LAST_MARBLE: u32 = 70769;

    let mut players = vec![Player::default(); PLAYERS];
    play_game(&mut players, &mut Circle::new(), LAST_MARBLE);
    writeln!(
        io::stdout(),
        "winning score (part 1): {}",
        players.iter().map(|p| p.points).max().unwrap(),
    )?;
    Ok(())
}

fn part2() -> Result<()> {
    const PLAYERS: usize = 418;
    const LAST_MARBLE: u32 = 7076900;

    let mut players = vec![Player::default(); PLAYERS];
    play_game(&mut players, &mut Circle::new(), LAST_MARBLE);
    writeln!(
        io::stdout(),
        "winning score (part 2): {}",
        players.iter().map(|p| p.points).max().unwrap(),
    )?;
    Ok(())
}

fn play_game(players: &mut [Player], circle: &mut Circle, marbles: u32) {
    let start = circle.max_marble_value() + 1; // 1 for fresh game
    let end = start + marbles;
    for (player_id, value) in (0..players.len()).cycle().zip(start..end) {
        circle.turn(&mut players[player_id], value);
    }
}

#[derive(Clone, Debug, Default)]
struct Player {
    points: u32,
}

/// Circle is a cyclic linked list of marbles. Instead of using real pointers
/// like a traditional linked list, we simply assign an incrementing ID to
/// each marble.
///
/// In this implementation, removing a marble doesn't reclaim its space, but
/// that could be implemented fairly easily with a free list of identifiers.
///
/// A traditional linked list would have been easy enough to implement with
/// Rc<RefCell<Marble>> for `next` and `Weak<RefCell<Marble>>` for prev, but
/// the circle actually causes a cycle among the `next` pointers, unlike a
/// normal linked list. We could handle it as a special case, but in the end,
/// this code is simpler.
struct Circle {
    marbles: Vec<Marble>,
    current: MarbleID,
}

type MarbleID = usize;
type MarbleValue = u32;

#[derive(Debug)]
struct Marble {
    value: MarbleValue,
    prev: MarbleID,
    next: MarbleID,
}

impl Circle {
    fn new() -> Circle {
        let first = Marble { value: 0, prev: 0, next: 0 };
        Circle { marbles: vec![first], current: 0 }
    }

    fn turn(&mut self, player: &mut Player, value: MarbleValue) {
        let marble_id = self.add_marble(value);
        if value % 23 != 0 {
            let insert_at = self.clockwise(1);
            self.insert_after(marble_id, insert_at);
            self.current = marble_id;
            return;
        }

        player.points += value;
        let remove_id = self.counter_clockwise(7);
        player.points += self.marbles[remove_id].value;
        self.remove(remove_id);
        self.current = self.counter_clockwise(6);
    }

    fn max_marble_value(&self) -> MarbleValue {
        (self.marbles.len() - 1) as MarbleValue
    }

    fn add_marble(&mut self, value: MarbleValue) -> MarbleID {
        let id = self.marbles.len();
        self.marbles.push(Marble::unlinked(value));
        id
    }

    fn insert_after(&mut self, to_insert: MarbleID, after: MarbleID) {
        let old_next = self.marbles[after].next;
        self.marbles[after].next = to_insert;
        self.marbles[old_next].prev = to_insert;
        self.marbles[to_insert].prev = after;
        self.marbles[to_insert].next = old_next;
    }

    fn remove(&mut self, id: MarbleID) {
        let (prev, next) = (self.marbles[id].prev, self.marbles[id].next);
        self.marbles[prev].next = next;
        self.marbles[next].prev = prev;
    }

    fn clockwise(&mut self, mut i: usize) -> MarbleID {
        let mut id = self.current;
        while i > 0 {
            id = self.marbles[id].next;
            i -= 1;
        }
        id
    }

    fn counter_clockwise(&mut self, mut i: usize) -> MarbleID {
        let mut id = self.current;
        while i > 0 {
            id = self.marbles[id].prev;
            i -= 1;
        }
        id
    }
}

impl Marble {
    fn unlinked(value: MarbleValue) -> Marble {
        Marble { value, prev: 0, next: 0 }
    }
}

impl fmt::Debug for Circle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut id = self.current;
        loop {
            let m = &self.marbles[id];
            write!(f, "{} ", m.value)?;
            id = m.next;
            if id == self.current {
                break;
            }
        }
        Ok(())
    }
}
