use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

use crate::{compute::Compute, traits::Named};

#[derive(Clone, Debug, PartialEq)]
pub enum OutputSpec {
    Step,
    Compute(Compute),
}
impl Display for OutputSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OutputSpec::Step => "step",
            OutputSpec::Compute(c) => c.name(),
        };
        String::from(s).fmt(f)
    }
}

#[derive(Clone)]
pub struct Output {
    pub every: usize,
    pub values: Vec<OutputSpec>,
}
impl Output {
    pub fn new() -> Self {
        Self {
            every: 100,
            values: Vec::new(),
        }
    }
}

// After init run, set_output should send one message to manager
// On output, send thread_id w/ nlocal, then thread_id with each value

#[derive(Clone, Debug)]
pub enum Value {
    Int(i32),
    Usize(usize),
    Float(f64),
}
impl Value {
    pub fn default(&self, op: Operation) -> Self {
        match op {
            Operation::Sum | Operation::First => match self {
                Value::Float(_) => Value::Float(0.0),
                Value::Int(_) => Value::Int(0),
                Value::Usize(_) => Value::Usize(0),
            },
            Operation::Max => match self {
                Value::Float(_) => Value::Float(f64::MIN),
                Value::Int(_) => Value::Int(i32::MIN),
                Value::Usize(_) => Value::Usize(usize::MIN),
            },
            Operation::Min => match self {
                Value::Float(_) => Value::Float(f64::MAX),
                Value::Int(_) => Value::Int(i32::MAX),
                Value::Usize(_) => Value::Usize(usize::MAX),
            },
        }
    }
    pub fn max(self, other: Self) -> Self {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => Value::Float(f64::max(f1, f2)),
            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1.max(i2)),
            (Value::Usize(u1), Value::Usize(u2)) => Value::Usize(u1.max(u2)),
            _ => panic!("Mismatched types"),
        }
    }
    pub fn min(self, other: Self) -> Self {
        match (self, other) {
            (Value::Float(f1), Value::Float(f2)) => Value::Float(f64::min(f1, f2)),
            (Value::Int(i1), Value::Int(i2)) => Value::Int(i1.min(i2)),
            (Value::Usize(u1), Value::Usize(u2)) => Value::Usize(u1.min(u2)),
            _ => panic!("Mismatched types"),
        }
    }
}
impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => Value::Float(i + j),
            (Value::Int(i), Value::Int(j)) => Value::Int(i + j),
            (Value::Usize(i), Value::Usize(j)) => Value::Usize(i + j),
            _ => panic!("Mismatched types"),
        }
    }
}
impl AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Value::Float(i), Value::Float(j)) => *i += j,
            (Value::Int(i), Value::Int(j)) => *i += j,
            (Value::Usize(i), Value::Usize(j)) => *i += j,
            _ => panic!("Mismatched types"),
        }
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(v) => v.fmt(f),
            Value::Int(v) => v.fmt(f),
            Value::Usize(v) => v.fmt(f),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    First,
    Max,
    Min,
    Sum,
}

pub trait Operatable {
    fn op(&self) -> Operation;
}
