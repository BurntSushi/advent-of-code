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
    let prog: Program = input.parse()?;

    part1(&prog)?;
    part2(&prog)?;
    Ok(())
}

fn part1(prog: &Program) -> Result<()> {
    let mut vm = VM::default();
    writeln!(io::stdout(), "result in register 0: {}", vm.exec(prog)?)?;
    Ok(())
}

fn part2(prog: &Program) -> Result<()> {
    let mut vm = VM::default();
    vm.registers.set(Register::R0, 1);
    writeln!(io::stdout(), "result in register 0, redux: {}", vm.exec(prog)?)?;
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct VM {
    registers: Registers,
    ip: usize,
}

impl VM {
    fn exec(&mut self, prog: &Program) -> Result<i64> {
        while let Some(op) = prog.ops.get(self.ip) {
            if self.ip == 3 {
                self.ip = self.fast();
                continue;
            }
            self.registers.set(prog.ipreg, self.ip as i64);
            op.exec(&mut self.registers);
            self.ip = self.registers.get(prog.ipreg) as usize + 1;
        }
        Ok(self.registers.get(Register::R0))
    }

    fn fast(&mut self) -> usize {
        use self::Register::*;

        // The code below optimizes this loop:
        //
        // R2 = ...  # invariant below
        //
        // R3 = R4 * R1
        // if R3 == R2:
        //   R3 = 1
        //   R0 = R4 + R0
        // else:
        //   R3 = 0
        // R1 = R1 + 1
        // if R1 > R2:
        //   R3 = 1
        //   goto beginning
        // else:
        //   R3 = 0
        //   continue to ip=12
        //
        // The above appears to be a very inefficient way of determining
        // whether R4 divides R2.

        if self.registers.get(R2) % self.registers.get(R4) == 0 {
            let sum = self.registers.get(R4) + self.registers.get(R0);
            self.registers.set(R0, sum);
        }

        let r2 = self.registers.get(R2);
        self.registers.set(R1, r2);
        self.registers.set(R3, 0);
        12
    }

}

#[derive(Clone, Debug)]
struct Program {
    ipreg: Register,
    ops: Vec<Op>,
}

impl FromStr for Program {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Program> {
        let mut prog = Program {
            ipreg: Register::R1,
            ops: vec![],
        };
        for line in s.lines() {
            if line.starts_with("#ip ") {
                let bound: i64 = line[4..].parse()?;
                prog.ipreg = Register::from_number(bound)?;
            } else {
                prog.ops.push(line.parse()?);
            }
        }
        Ok(prog)
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
struct Registers([i64; 6]);

#[derive(Clone, Copy, Debug)]
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
}

impl Registers {
    fn get(&self, r: Register) -> i64 {
        match r {
            Register::R0 => self.0[0],
            Register::R1 => self.0[1],
            Register::R2 => self.0[2],
            Register::R3 => self.0[3],
            Register::R4 => self.0[4],
            Register::R5 => self.0[5],
        }
    }

    fn set(&mut self, r: Register, v: i64) {
        match r {
            Register::R0 => self.0[0] = v,
            Register::R1 => self.0[1] = v,
            Register::R2 => self.0[2] = v,
            Register::R3 => self.0[3] = v,
            Register::R4 => self.0[4] = v,
            Register::R5 => self.0[5] = v,
        }
    }
}

impl Register {
    fn from_number(n: i64) -> Result<Register> {
        match n {
            0 => Ok(Register::R0),
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            5 => Ok(Register::R5),
            _ => err!("invalid register number: {}", n),
        }
    }
}

impl FromStr for Op {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Op> {
        use self::OpKind::*;

        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?P<name>[a-z]+) (?P<a>[0-9]+) (?P<b>[0-9]+) (?P<c>[0-9]+)"
            ).unwrap();
        }

        let caps = match RE.captures(s) {
            None => return err!("invalid instruction: '{:?}'", s),
            Some(caps) => caps,
        };
        let (a, b) = (caps["a"].parse()?, caps["b"].parse()?);
        let mkreg = Register::from_number;
        let kind = match &caps["name"] {
            "addr" => Addr { a: mkreg(a)?, b: mkreg(b)? },
            "addi" => Addi { a: mkreg(a)?, b },
            "mulr" => Mulr { a: mkreg(a)?, b: mkreg(b)? },
            "muli" => Muli { a: mkreg(a)?, b },
            "banr" => Banr { a: mkreg(a)?, b: mkreg(b)? },
            "bani" => Bani { a: mkreg(a)?, b },
            "borr" => Borr { a: mkreg(a)?, b: mkreg(b)? },
            "bori" => Bori { a: mkreg(a)?, b },
            "setr" => Setr { a: mkreg(a)? },
            "seti" => Seti { a },
            "gtir" => Gtir { a, b: mkreg(b)? },
            "gtri" => Gtri { a: mkreg(a)?, b },
            "gtrr" => Gtrr { a: mkreg(a)?, b: mkreg(b)? },
            "eqir" => Eqir { a, b: mkreg(b)? },
            "eqri" => Eqri { a: mkreg(a)?, b },
            "eqrr" => Eqrr { a: mkreg(a)?, b: mkreg(b)? },
            unk => return err!("unknown instruction name: {:?}", unk),
        };
        Ok(Op {
            output: Register::from_number(caps["c"].parse()?)?,
            kind: kind,
        })
    }
}
