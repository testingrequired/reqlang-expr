use std::ops::Range;

pub type Span = Range<usize>;
pub type Spanned<T> = (T, Span);
