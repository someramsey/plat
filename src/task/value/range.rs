use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct RangeValue(pub i32, pub i32);

impl Display for RangeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "range {}-{}", self.0, self.1)
    }
}