//! RocStr errors type : insufficient capacity error type used with `try_...` functions

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;

use crate::RocStr;

const DEFAULT_MESSAGE: &str = "CAPACITY ERROR : this RocStr cannot contains this string.";

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InsufficientCapacity<const SIZE: usize>(RocStr<SIZE>);

impl<const SIZE: usize> From<RocStr<SIZE>> for InsufficientCapacity<SIZE> {
    #[inline]
    #[must_use]
    fn from(value: RocStr<SIZE>) -> Self {
        Self(value)
    }
}

impl<const SIZE: usize, T> From<T> for InsufficientCapacity<SIZE>
where
    T: AsRef<str>,
{
    #[inline]
    #[must_use]
    fn from(value: T) -> Self {
        Self(RocStr::from(value.as_ref()))
    }
}

impl<const SIZE: usize> Display for InsufficientCapacity<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl Default for InsufficientCapacity<57> {
    #[inline]
    #[must_use]
    fn default() -> Self {
        DEFAULT_MESSAGE.into()
    }
}

#[cfg(feature = "std")]
pub mod std {
    extern crate std;
    use super::*;
    impl<const SIZE: usize> std::error::Error for InsufficientCapacity<SIZE> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rocerr_should_be_the_defined_default_value() {
        let expected = InsufficientCapacity(RocStr::<57>::from(DEFAULT_MESSAGE));
        let sample = InsufficientCapacity::default();

        assert_eq!(expected, sample);
    }

    #[test]
    fn rocerr_from_str_should_contains_str_message() {
        let expected = InsufficientCapacity(RocStr::<25>::from("This is a capacity error."));
        let sample = InsufficientCapacity::from("This is a capacity error.");

        assert_eq!(expected, sample);
    }

    #[test]
    fn rocerr_from_rocstr_should_contains_rocstr_message() {
        let expected = InsufficientCapacity(RocStr::<25>::from("This is a capacity error."));
        let sample = InsufficientCapacity::from(RocStr::<25>::from("This is a capacity error."));

        assert_eq!(expected, sample);
    }

    #[test]
    fn rocerr_should_display_as_a_str_message() {
        extern crate std;
        use std::format;
        use std::string::ToString;

        let expected = "This is a capacity error.".to_string();
        let sample = format!(
            "{}",
            InsufficientCapacity::<25>::from("This is a capacity error.")
        );

        assert_eq!(expected, sample);
    }
}
