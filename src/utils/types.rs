use std::ops::Range;

pub enum Types {
    One(u32),
    Range(Range<u32>),
}
impl Types {
    pub fn to_vec(&self) -> Vec<u32> {
        match self {
            Types::One(i) => vec![*i],
            Types::Range(r) => r.to_owned().collect(),
        }
    }
    pub fn to_range(&self) -> Range<u32> {
        match self {
            Types::One(i) => *i..*i + 1,
            Types::Range(r) => r.clone(),
        }
    }
}
impl From<u32> for Types {
    fn from(value: u32) -> Self {
        Self::One(value)
    }
}
