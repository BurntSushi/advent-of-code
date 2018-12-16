use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::result;
use std::str::{self, FromStr};

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let sample_input = match env::args_os().nth(1) {
        None => return err!("Usage: aoc16 <sample-path> <program-path>"),
        Some(p) => fs::read_to_string(p)?,
    };
    let program_input = match env::args_os().nth(2) {
        None => return err!("Usage: aoc16 <sample-path> <program-path>"),
        Some(p) => fs::read_to_string(p)?,
    };
    let samples: Samples = sample_input.parse()?;

    part1(&samples)?;
    part2(&samples, &program_input)?;
    Ok(())
}

fn part1(samples: &Samples) -> Result<()> {
    let mut count = 0;
    for s in &samples.0 {
        if s.similar_to()?.len() >= 3 {
            count += 1;
        }
    }
    writeln!(io::stdout(), "samples similar to 3+ ops: {}", count)?;
    Ok(())
}

fn part2(samples: &Samples, program: &str) -> Result<()> {
    let opmap = InstructionMapping::derive(samples)?;
    let prog = Program::parse(|n| opmap[&n], program)?;

    let mut regs = Registers::default();
    prog.exec(&mut regs);

    let output = regs.get(Register::R1);
    writeln!(io::stdout(), "result in register 0: {}", output)?;
    Ok(())
}

#[derive(Clone, Debug)]
struct Program(Vec<Op>);

impl Program {
    fn parse(
        mut opmap: impl FnMut(OpNumber) -> OpNumber,
        s: &str,
    ) -> Result<Program> {
        let mut ops = vec![];
        for line in s.lines() {
            let inst: UnknownInstruction = line.parse()?;
            ops.push(inst.to_op(&mut opmap)?);
        }
        Ok(Program(ops))
    }

    fn exec(&self, regs: &mut Registers) {
        for op in &self.0 {
            op.exec(regs);
        }
    }
}

#[derive(Clone, Debug)]
struct Op {
    output: Register,
    kind: OpKind,
}

#[derive(Clone, Debug)]
enum OpKind {
    Addr { a: Register, b: Register },
    Addi { a: Register, b: Immediate },
    Mulr { a: Register, b: Register },
    Muli { a: Register, b: Immediate },
    Banr { a: Register, b: Register },
    Bani { a: Register, b: Immediate },
    Borr { a: Register, b: Register },
    Bori { a: Register, b: Immediate },
    Setr { a: Register },
    Seti { a: Immediate },
    Gtir { a: Immediate, b: Register },
    Gtri { a: Register, b: Immediate },
    Gtrr { a: Register, b: Register },
    Eqir { a: Immediate, b: Register },
    Eqri { a: Register, b: Immediate },
    Eqrr { a: Register, b: Register },
}

impl Op {
    fn exec(&self, regs: &mut Registers) {
        use self::OpKind::*;

        let value = match self.kind {
            Addr { a, b } => regs.get(a) + regs.get(b),
            Addi { a, b } => regs.get(a) + b,
            Mulr { a, b } => regs.get(a) * regs.get(b),
            Muli { a, b } => regs.get(a) * b,
            Banr { a, b } => regs.get(a) & regs.get(b),
            Bani { a, b } => regs.get(a) & b,
            Borr { a, b } => regs.get(a) | regs.get(b),
            Bori { a, b } => regs.get(a) | b,
            Setr { a } => regs.get(a),
            Seti { a } => a,
            Gtir { a, b } => if a > regs.get(b) { 1 } else { 0 },
            Gtri { a, b } => if regs.get(a) > b { 1 } else { 0 },
            Gtrr { a, b } => if regs.get(a) > regs.get(b) { 1 } else { 0 },
            Eqir { a, b } => if a == regs.get(b) { 1 } else { 0 },
            Eqri { a, b } => if regs.get(a) == b { 1 } else { 0 },
            Eqrr { a, b } => if regs.get(a) == regs.get(b) { 1 } else { 0 },
        };
        regs.set(self.output, value);
    }
}

type Immediate = i64;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Registers([i64; 4]);

#[derive(Clone, Copy, Debug)]
enum Register {
    R1,
    R2,
    R3,
    R4,
}

impl Registers {
    fn from_slice(registers: &[i64]) -> Result<Registers> {
        if registers.len() != 4 {
            return err!("regs must have len == 4, got {}", registers.len());
        }
        Ok(Registers([registers[0], registers[1], registers[2], registers[3]]))
    }

    fn get(&self, r: Register) -> i64 {
        match r {
            Register::R1 => self.0[0],
            Register::R2 => self.0[1],
            Register::R3 => self.0[2],
            Register::R4 => self.0[3],
        }
    }

    fn set(&mut self, r: Register, v: i64) {
        match r {
            Register::R1 => self.0[0] = v,
            Register::R2 => self.0[1] = v,
            Register::R3 => self.0[2] = v,
            Register::R4 => self.0[3] = v,
        }
    }
}

impl Register {
    fn from_number(n: i64) -> Result<Register> {
        match n {
            0 => Ok(Register::R1),
            1 => Ok(Register::R2),
            2 => Ok(Register::R3),
            3 => Ok(Register::R4),
            _ => err!("invalid register number: {}", n),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct OpNumber(u8);

impl OpNumber {
    fn new(n: i64) -> Result<OpNumber> {
        if n <= 15 {
            Ok(OpNumber(n as u8))
        } else {
            err!("op number {} is out of bounds", n)
        }
    }
}

#[derive(Clone, Debug, Default)]
struct InstructionMapping {
    unknown_to_known: HashMap<OpNumber, OpNumber>,
    known_mapped: HashSet<OpNumber>,
}

impl InstructionMapping {
    /// Upon success, the map returned has length exactly equal to 16 and
    /// uniquely maps every opcode number from the unknown encoding to the
    /// known encoding. The known encoding numbers the ops 0 through 16, in
    /// order of declaration.
    fn derive(samples: &Samples) -> Result<HashMap<OpNumber, OpNumber>> {
        let mut m = InstructionMapping::default();
        for _ in 0..16 {
            for s in &samples.0 {
                if m.unknown_to_known.contains_key(&s.instruction.op) {
                    continue;
                }

                let similar = s
                    .similar_to()?
                    .into_iter()
                    .filter(|n| !m.known_mapped.contains(n))
                    .collect::<Vec<OpNumber>>();
                if similar.len() == 1 {
                    m.unknown_to_known.insert(s.instruction.op, similar[0]);
                    m.known_mapped.insert(similar[0]);
                    break;
                }
            }
        }
        if m.unknown_to_known.len() != 16 {
            err!("samples do no lead to a unique mapping")
        } else {
            Ok(m.unknown_to_known)
        }
    }
}

#[derive(Clone, Debug)]
struct Samples(Vec<Sample>);

#[derive(Clone, Debug, Default)]
struct Sample {
    before: Registers,
    after: Registers,
    instruction: UnknownInstruction,
}

impl Sample {
    fn similar_to(&self) -> Result<Vec<OpNumber>> {
        let mut similar = vec![];
        for i in 0..16 {
            let opnum = OpNumber::new(i).unwrap();
            let op = self.instruction.to_op(|_| opnum)?;
            let mut regs = self.before.clone();
            op.exec(&mut regs);
            if regs == self.after {
                similar.push(opnum);
            }
        }
        Ok(similar)
    }
}

#[derive(Clone, Debug, Default)]
struct UnknownInstruction {
    op: OpNumber,
    a: i64,
    b: i64,
    c: i64,
}

impl UnknownInstruction {
    fn to_op(&self, mut to: impl FnMut(OpNumber) -> OpNumber) -> Result<Op> {
        use self::OpKind::*;

        let mkreg = Register::from_number;
        let kind = match to(self.op).0 {
            0 => Addr { a: mkreg(self.a)?, b: mkreg(self.b)? },
            1 => Addi { a: mkreg(self.a)?, b: self.b },
            2 => Mulr { a: mkreg(self.a)?, b: mkreg(self.b)? },
            3 => Muli { a: mkreg(self.a)?, b: self.b },
            4 => Banr { a: mkreg(self.a)?, b: mkreg(self.b)? },
            5 => Bani { a: mkreg(self.a)?, b: self.b },
            6 => Borr { a: mkreg(self.a)?, b: mkreg(self.b)? },
            7 => Bori { a: mkreg(self.a)?, b: self.b },
            8 => Setr { a: mkreg(self.a)? },
            9 => Seti { a: self.a },
            10 => Gtir { a: self.a, b: mkreg(self.b)? },
            11 => Gtri { a: mkreg(self.a)?, b: self.b },
            12 => Gtrr { a: mkreg(self.a)?, b: mkreg(self.b)? },
            13 => Eqir { a: self.a, b: mkreg(self.b)? },
            14 => Eqri { a: mkreg(self.a)?, b: self.b },
            15 => Eqrr { a: mkreg(self.a)?, b: mkreg(self.b)? },
            n => return err!("unrecognized op number: {}", n),
        };
        Ok(Op { output: mkreg(self.c)?, kind })
    }
}

impl FromStr for Samples {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Samples> {
        let mut samples = vec![];

        let mut cur = Sample::default();
        for line in s.lines() {
            let is_before = line.starts_with("Before:");
            let is_after = line.starts_with("After:");

            if is_before || is_after {
                let start = match line.find("[") {
                    None => return err!("invalid start: {}", line),
                    Some(start) => start,
                };
                let end = match line.find("]") {
                    None => return err!("invalid end: {}", line),
                    Some(end) => end,
                };
                let regs: Registers = line[start+1..end].parse()?;
                if is_before {
                    cur.before = regs;
                } else {
                    assert!(is_after);
                    cur.after = regs;
                    samples.push(cur);
                    cur = Sample::default();
                }
            } else if !line.is_empty() {
                cur.instruction = line.parse()?;
            }
        }
        Ok(Samples(samples))
    }
}

impl FromStr for Registers {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Registers> {
        let regs = s
            .split(",")
            .map(|s| s.trim().parse::<i64>().map_err(From::from))
            .collect::<Result<Vec<i64>>>()?;
        Registers::from_slice(&regs)
    }
}

impl FromStr for UnknownInstruction {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<UnknownInstruction> {
        let numbers = s
            .split_whitespace()
            .map(|s| s.trim().parse::<i64>().map_err(From::from))
            .collect::<Result<Vec<i64>>>()?;
        if numbers.len() != 4 {
            err!("expected 4 numbers for unk inst, but got {}", numbers.len())
        } else {
            Ok(UnknownInstruction {
                op: OpNumber::new(numbers[0])?,
                a: numbers[1],
                b: numbers[2],
                c: numbers[3],
            })
        }
    }
}
