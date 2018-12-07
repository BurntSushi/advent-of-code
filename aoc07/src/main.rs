#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{self, Read, Write};
use std::result;
use std::str::FromStr;

use regex::Regex;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

/// A map from step to all of its required dependency steps. The set of
/// required dependency sets may be empty.
type RequiredFor = HashMap<Step, HashSet<Step>>;

type Step = char;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut deps: Vec<Dependency> = vec![];
    for line in input.lines() {
        let dep = line.parse().or_else(|err| {
            err!("failed to parse '{:?}': {}", line, err)
        })?;
        deps.push(dep);
    }

    let mut required_for: RequiredFor = HashMap::new();
    for dep in deps {
        required_for.entry(dep.step).or_default().insert(dep.required);
        required_for.entry(dep.required).or_default();
    }

    part1(&required_for)?;
    part2(&required_for)?;
    Ok(())
}

fn part1(required_for: &RequiredFor) -> Result<()> {
    let mut taken: HashSet<Step> = HashSet::new();
    let mut order: Vec<Step> = vec![];
    let mut next: Vec<Step> = vec![];
    loop {
        find_next_steps(&required_for, &taken, &taken, &mut next);
        let next_step = match next.pop() {
            None => break,
            Some(next_step) => next_step,
        };
        taken.insert(next_step);
        order.push(next_step);
    }

    let answer: String = order.iter().cloned().collect();
    writeln!(io::stdout(), "step order: {}", answer)?;
    Ok(())
}

fn part2(required_for: &RequiredFor) -> Result<()> {
    let mut workers = Workers::new(5);
    let mut assigned: HashSet<Step> = HashSet::new();
    let mut done: HashSet<Step> = HashSet::new();
    let mut order: Vec<Step> = vec![];
    let mut next: Vec<Step> = vec![];

    let mut seconds = 0;
    loop {
        workers.run_one_step(&mut order, &mut done);

        find_next_steps(&required_for, &assigned, &done, &mut next);
        if next.is_empty() && workers.all_idle() {
            break;
        }
        for worker in workers.available() {
            let next_step = match next.pop() {
                None => break,
                Some(next_step) => next_step,
            };
            assigned.insert(next_step);
            workers.work_on(worker, next_step);
        }
        seconds += 1;
    }

    let answer: String = order.iter().cloned().collect();
    writeln!(io::stdout(), "step order (part 2): {}", answer)?;
    writeln!(io::stdout(), "total seconds: {}", seconds)?;
    Ok(())
}

/// Populate `next_stack` with next steps such that the steps are sorted in
/// reverse lexicographically with no duplicates.
///
/// Steps in `taken` are never added to the stack.
///
/// Steps in `done` signify which steps have already been completed. Only steps
/// with all dependencies completed will be put on to the stack.
fn find_next_steps(
    required_for: &RequiredFor,
    taken: &HashSet<Step>,
    done: &HashSet<Step>,
    next_stack: &mut Vec<Step>,
) {
    for (&step, dependencies) in required_for {
        if taken.contains(&step) {
            continue;
        }
        if dependencies.iter().all(|s| done.contains(s)) {
            next_stack.push(step);
        }
    }
    next_stack.sort();
    next_stack.dedup();
    next_stack.reverse();
}

/// Workers manages the simulation of a fixed size worker pool. This tracks
/// the status of each worker, whether idle or active. When active, we record
/// how much and what work remains until that worker is idle again.
#[derive(Debug)]
struct Workers {
    status: Vec<Status>,
}

type WorkerID = usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Idle,
    Working { step: Step, remaining: u32 }
}

impl Workers {
    fn new(count: usize) -> Workers {
        Workers { status: vec![Status::Idle; count] }
    }

    fn available(&self) -> Vec<WorkerID> {
        let mut available = vec![];
        for (worker, &status) in self.status.iter().enumerate() {
            if status == Status::Idle {
                available.push(worker);
            }
        }
        available
    }

    fn all_idle(&self) -> bool {
        self.status.iter().all(|s| *s == Status::Idle)
    }

    fn work_on(&mut self, worker: WorkerID, step: Step) {
        let status = &mut self.status[worker];
        assert!(*status == Status::Idle, "worker {} is not available", worker);

        let remaining = (step as u32) - b'A' as u32 + 1 + 60;
        *status = Status::Working { step, remaining }
    }

    /// Run one step in the simulation. Workers that have finished their work
    /// are transitioned to idle status.
    fn run_one_step(&mut self, order: &mut Vec<Step>, done: &mut HashSet<Step>) {
        for worker in 0..self.status.len() {
            let mut is_done = false;
            match self.status[worker] {
                Status::Idle => {}
                Status::Working { step, ref mut remaining } => {
                    *remaining -= 1;
                    if *remaining == 0 {
                        is_done = true;
                        order.push(step);
                        done.insert(step);
                    }
                }
            }
            if is_done {
                self.status[worker] = Status::Idle;
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Dependency {
    step: Step,
    required: Step,
}

impl FromStr for Dependency {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Dependency> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
              r"Step ([A-Z]) must be finished before step ([A-Z]) can begin."
            ).unwrap();
        }

        let caps = match RE.captures(s) {
            None => return err!("unrecognized dependency"),
            Some(caps) => caps,
        };
        Ok(Dependency {
            step: caps[2].as_bytes()[0] as Step,
            required: caps[1].as_bytes()[0] as Step,
        })
    }
}
