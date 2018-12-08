use std::error::Error;
use std::io::{self, Read, Write};
use std::result;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<Error>::from(format!($($tt)*))) }
}

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut flat = vec![];
    for number in input.split_whitespace() {
        flat.push(number.parse()?);
    }
    let root = Node::from_flat(&flat)?;

    part1(&root)?;
    part2(&root)?;
    Ok(())
}

fn part1(root: &Node) -> Result<()> {
    writeln!(io::stdout(), "{}", root.sum_all_metadata())?;
    Ok(())
}

fn part2(root: &Node) -> Result<()> {
    writeln!(io::stdout(), "{}", root.value())?;
    Ok(())
}

#[derive(Debug, Default)]
struct Node {
    metadata: Vec<i32>,
    children: Vec<Node>,
    // Total count of numbers in this node. For the root node, this corresponds
    // to the total count of all numbers in the tree.
    len: usize,
}

impl Node {
    fn from_flat(flat: &[i32]) -> Result<Node> {
        if flat.len() < 2 {
            return err!("invalid header for node");
        }

        let (child_count, meta_count) = (flat[0], flat[1]);
        let mut node = Node { len: 2, ..Node::default() };
        for _ in 0..child_count {
            let child = Node::from_flat(&flat[node.len..])?;
            node.len += child.len;
            node.children.push(child);
        }
        for _ in 0..meta_count {
            let meta = match flat.get(node.len) {
                None => return err!("no meta data matching header"),
                Some(&i) if i < 1 => return err!("invalid meta data"),
                Some(&i) => i,
            };
            node.metadata.push(meta);
            node.len += 1;
        }
        Ok(node)
    }

    fn sum_all_metadata(&self) -> i32 {
        let mut sum = self.metadata.iter().cloned().sum();
        for child in &self.children {
            sum += child.sum_all_metadata();
        }
        sum
    }

    fn value(&self) -> i32 {
        if self.children.is_empty() {
            return self.metadata.iter().cloned().sum::<i32>();
        }

        let mut sum = 0;
        for &i in &self.metadata {
            let child = match self.children.get(i as usize - 1) {
                None => continue,
                Some(child) => child,
            };
            sum += child.value();
        }
        sum
    }
}
