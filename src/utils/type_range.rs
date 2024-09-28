pub enum CoeffType {
    Value(u32),
    Span,
}
pub struct TypeRange {
    begin: CoeffType,
    end: CoeffType,
}
impl TypeRange {
    pub fn new(min: Option<u32>, max: Option<u32>) -> Self {
        let begin = match min {
            Some(b) => CoeffType::Value(b),
            None => CoeffType::Span,
        };
        let end = match max {
            Some(e) => CoeffType::Value(e),
            None => CoeffType::Span,
        };
        Self { begin, end }
    }
    pub fn contains(&self, idx: u32) -> bool {
        if let CoeffType::Value(begin) = self.begin {
            if begin > idx {
                return false;
            }
        }
        if let CoeffType::Value(end) = self.end {
            if end < idx {
                return false;
            }
        }
        true
    }
    pub fn max(&self) -> Option<u32> {
        match self.end {
            CoeffType::Value(x) => Some(x),
            CoeffType::Span => None,
        }
    }
    pub fn min(&self) -> Option<u32> {
        match self.begin {
            CoeffType::Value(x) => Some(x),
            CoeffType::Span => None,
        }
    }
}
