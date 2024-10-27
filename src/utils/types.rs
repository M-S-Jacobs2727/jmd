use std::ops::Range;

pub enum Types {
    One(usize),
    Range(Range<usize>),
}
impl Types {
    pub fn to_vec(&self) -> Vec<usize> {
        match self {
            Types::One(i) => vec![*i],
            Types::Range(r) => r.to_owned().collect(),
        }
    }
    pub fn to_range(&self) -> Range<usize> {
        match self {
            Types::One(i) => *i..*i + 1,
            Types::Range(r) => r.clone(),
        }
    }
}
impl From<Range<usize>> for Types {
    fn from(value: Range<usize>) -> Self {
        Types::Range(value)
    }
}
impl From<usize> for Types {
    fn from(value: usize) -> Self {
        Self::One(value)
    }
}
