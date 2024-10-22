use std::{fmt::Display, ops::AddAssign, thread};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OutputSpec {
    Step,
    Temp,
    KineticE,
    PotentialE,
    TotalE,
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
pub enum Operation {
    First,
    Max,
    Min,
    Sum,
}
pub struct OutputMessage {
    pub id: thread::ThreadId,
    pub value: Value,
    pub op: Operation,
}
impl OutputMessage {
    pub fn new(value: Value, op: Operation) -> Self {
        Self {
            id: thread::current().id(),
            value,
            op,
        }
    }
}
