use std::sync::Arc;

pub type Str = Arc<str>;

#[macro_export]
macro_rules! str {
    ($($arg:tt)*) => {
        Arc::from(format!($($arg)*))
    };
}