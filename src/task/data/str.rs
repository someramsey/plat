use std::sync::Arc;

pub type Str = Arc<str>;

#[macro_export]
macro_rules! str {
    ($($arg:tt)*) => {
        Arc::from(format!($($arg)*))
    };
}

pub fn ch_to_str(c: char) -> Arc<str> {
    let mut buffer = [0; 4];
    let s = c.encode_utf8(&mut buffer);
    Arc::from(s as &str)
}

pub fn concat_str(vec: Vec<Arc<str>>) -> Arc<str> {
    let total_len: usize = vec.iter().map(|s| s.len()).sum();
    let mut buffer = Vec::with_capacity(total_len);

    for arc_str in vec {
        buffer.extend_from_slice(arc_str.as_bytes());
    }

    let concatenated_str = unsafe { std::str::from_utf8_unchecked(&buffer) };
    Arc::from(concatenated_str)
}