//! An immutable fixed capacity stack based generic copy string.

use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use core::hash::Hash;
use core::hash::Hasher;
use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Neg;
use core::ops::Sub;
use core::str::from_utf8;

#[derive(Copy, Clone)]
pub struct RocStr<const SIZE: usize> {
    inner: [u8; SIZE],
    len: usize,
}

impl<const SIZE: usize> RocStr<SIZE> {
    /// Extracts a slice of bytes containing the entire [`RocStr`].
    ///
    /// # Examples
    /// ```
    /// # use rocstr::RocStr;
    /// let s = RocStr::<3>::from("foo");
    /// assert_eq!(b"foo", s.as_bytes());
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.into()
    }

    /// Extracts a string slice containing the entire [`RocStr`].
    ///
    /// # Examples
    /// ```
    /// # use rocstr::RocStr;
    /// let s = RocStr::<3>::from("foo");
    /// assert_eq!("foo", s.as_str());
    /// ```
    pub fn as_str(&self) -> &str {
        self.into()
    }

    /// Return the capacity of the [`RocStr`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use rocstr::RocStr;
    /// let string = RocStr::<16>::from("");
    /// assert_eq!(string.capacity(), 16);
    /// ```
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        SIZE
    }

    /// Returns `true` if this [`RocStr`] is an empty string.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use rocstr::RocStr;
    /// let s = RocStr::<16>::from("");
    /// assert!(s.is_empty());
    ///
    /// let s = RocStr::<16>::from("foo");
    /// assert!(!s.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the length of this [`RocStr`], in bytes, not [`char`]s or graphemes.
    ///
    /// In other words, it might not be what a human considers the length of the string.
    ///
    /// # Examples
    /// ```
    /// # use rocstr::RocStr;
    /// let s = RocStr::<16>::from("foo");
    /// assert_eq!(s.len(), 3);
    ///
    /// let fancy_f = RocStr::<16>::from("ƒoo");
    /// assert_eq!(fancy_f.len(), 4);
    /// assert_eq!(fancy_f.as_str().chars().count(), 3);
    /// ```
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Replaces all matches of a pattern with another string.
    ///
    /// `replace` creates a new [`RocStr`], and copies the data from this [`RocStr`] into it.
    /// While doing so, it attempts to find matches of a pattern.
    /// If it finds any, it replaces them with the replacement string.
    ///
    /// If replacing with the replacement string make this [`RocStr`] overflow its capacity,
    /// the string will be trim to at most the capacity.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use rocstr::RocStr;
    ///
    /// let s = RocStr::<16>::from("this is old");
    ///
    /// assert_eq!(RocStr::<16>::from("this is new"), s.replace("old", "new"));
    /// assert_eq!(RocStr::<16>::from("than an old"), s.replace("is", "an"));
    /// ```
    ///
    /// When the pattern doesn't match, it returns this [`RocStr`]:
    ///
    /// ```
    /// let s = "this is old";
    /// assert_eq!(s, s.replace("cookie monster", "little lamb"));
    /// ```
    pub fn replace(&self, from: &str, to: &str) -> Self {
        if from.is_empty() {
            *self
        } else {
            let pattern = from.as_bytes();
            let mut len = 0;
            let mut skip = 0;

            let mut inner = [b' '; SIZE];
            let frames = self.inner[..self.len].windows(from.len()).enumerate();
            for (i, frame) in frames {
                if skip == 0 {
                    // Nothing to skip
                    if frame == pattern {
                        let end = len + to.len();
                        if end <= SIZE {
                            inner[len..end].copy_from_slice(to.as_bytes());
                            len += to.len();
                            // skip the from.len() bytes minus the one we are in
                            skip = from.len() - 1;
                        } else {
                            let remaining_slots = SIZE - len;
                            inner[len..SIZE].copy_from_slice(&to.as_bytes()[0..remaining_slots]);
                            len = SIZE;
                            break;
                        }
                    } else if len < SIZE {
                        inner[len] = self.inner[i];
                        len += 1;
                    } else {
                        break;
                    }
                } else {
                    skip -= 1;
                    continue;
                }
            }

            // add the remaining bytes, the last frame, only if it remains some space
            if len < SIZE && skip == 0 {
                let remaining_slots = SIZE - len;
                let remaining_bytes = &self.inner[self.len - from.len() + 1..self.len];
                let remaining_bytes = if remaining_slots > remaining_bytes.len() {
                    remaining_bytes
                } else {
                    &remaining_bytes[..remaining_slots]
                };
                inner[len..len + remaining_bytes.len()].copy_from_slice(remaining_bytes);
                len += remaining_bytes.len();
            }

            Self { inner, len }
        }
    }

    /// Returns a copy of this [`RocStr`] with capacity set to `LEN`.
    ///
    /// It will silently trim this [`RocStr`] if its length is greater than `LEN`.
    ///
    /// # Examples
    /// ```
    /// # use rocstr::RocStr;
    /// let s = RocStr::<16>::from("foo");
    /// assert_eq!(s.reshape::<8>().capacity(), 8);
    ///
    /// let s = RocStr::<16>::from("foo bar");
    /// let t = RocStr::<4>::from("foo ");
    /// assert_eq!(s.reshape::<4>(), t);
    /// ```
    #[inline]
    #[must_use]
    pub fn reshape<const LEN: usize>(&self) -> RocStr<LEN> {
        let mut inner = [b' '; LEN];
        let slice = extract_utf8_within(&self.inner[..self.len], LEN);
        let len = slice.len();
        inner[..len].copy_from_slice(slice);

        RocStr { inner, len }
    }

    /// Returns `true` if the given `&str` matches a prefix of this RocStr.
    ///
    /// Returns `false` if it does not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rocstr::RocStr;
    /// let bananas = RocStr::<16>::from("bananas");
    ///
    /// assert!(bananas.starts_with("bana"));
    /// assert!(!bananas.starts_with("nana"));
    /// ```
    pub fn starts_with(&self, pattern: &str) -> bool {
        self.as_bytes().starts_with(pattern.as_bytes())
    }

    /// Returns a [`RocStr`] with a valid utf-8 string with at most `len` bytes.
    ///
    /// The source [`RocStr`] remains unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// # use rocstr::RocStr;
    /// let s = RocStr::<32>::from("Löwe 老虎 Léopard");
    ///
    /// /* first byte of `ö` is not utf-8 boundary */
    /// assert_eq!(s.truncate(2), "L");
    ///
    /// /* second byte of `老`is not utf-8 boundary */
    /// assert_eq!(s.truncate(8), "Löwe ");
    /// ```
    #[inline]
    #[must_use]
    pub fn truncate(&self, len: usize) -> Self {
        let slice = extract_utf8_within(self.as_bytes(), len);
        let len = slice.len();
        let mut inner = [b' '; SIZE];
        inner[..len].copy_from_slice(slice);

        Self { inner, len }
    }
}

impl<const SIZE: usize> Debug for RocStr<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let inner: &str = self.into();
        f.debug_struct("RocStr")
            .field("inner", &inner)
            .field("len", &self.len)
            .finish()
    }
}

impl<const SIZE: usize> Default for RocStr<SIZE> {
    fn default() -> Self {
        Self {
            inner: [0; SIZE],
            len: Default::default(),
        }
    }
}

impl<const SIZE: usize> Display for RocStr<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", Into::<&str>::into(self))
    }
}

impl<const SIZE: usize> Eq for RocStr<SIZE> {}

// Ideally, the signature should be
//     `fn from(value: T) -> Self where T: AsRef<str>`
// But this conflict with other `From`` implementation.
impl<const SIZE: usize> From<&str> for RocStr<SIZE> {
    #[inline]
    #[must_use]
    fn from(value: &str) -> Self {
        let bytes = value.as_bytes();
        let slice = extract_utf8_within(bytes, SIZE);
        let len = slice.len();

        let mut inner = [0; SIZE];
        inner[..len].copy_from_slice(slice);
        Self { inner, len }
    }
}

impl<'a, const SIZE: usize> From<&'a RocStr<SIZE>> for &'a str {
    #[inline]
    #[must_use]
    fn from(value: &'a RocStr<SIZE>) -> Self {
        match from_utf8(value.inner[..value.len].as_ref()) {
            Ok(string) => string,
            // Unless unsafe use, this should never happen.
            // This data is immutable and can only be initialized from a valid utf-8 string.
            Err(_) => unreachable!(),
        }
    }
}

impl<'a, const SIZE: usize> From<&'a RocStr<SIZE>> for &'a [u8] {
    #[inline]
    #[must_use]
    fn from(value: &'a RocStr<SIZE>) -> Self {
        &value.inner[..value.len]
    }
}

impl<const SIZE: usize> Hash for RocStr<SIZE> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write(&self.inner[..self.len]);
        hasher.write_u8(0xff);
    }
}

impl<const SIZE: usize> PartialEq<str> for RocStr<SIZE> {
    #[inline]
    #[must_use]
    fn eq(&self, other: &str) -> bool {
        self.len == other.len() && &self.inner[..self.len] == other.as_bytes()
    }
}

impl<const SIZE: usize, T> PartialEq<T> for RocStr<SIZE>
where
    T: AsRef<str>,
{
    #[inline]
    #[must_use]
    fn eq(&self, other: &T) -> bool {
        let other = other.as_ref();
        self.len == other.len() && &self.inner[..self.len] == other.as_bytes()
    }
}

impl<const SIZE: usize> PartialEq<RocStr<SIZE>> for &str {
    #[inline]
    #[must_use]
    fn eq(&self, other: &RocStr<SIZE>) -> bool {
        self.len() == other.len && self.as_bytes() == &other.inner[..other.len]
    }
}

impl<const SIZE: usize, const LEN: usize> PartialEq<RocStr<SIZE>> for RocStr<LEN> {
    #[inline]
    #[must_use]
    fn eq(&self, other: &RocStr<SIZE>) -> bool {
        self.len() == other.len && self.inner[..self.len] == other.inner[..other.len]
    }
}

impl<const SIZE: usize, T> Add<T> for RocStr<SIZE>
where
    T: AsRef<str>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.as_ref();
        let mut inner = self.inner;
        let (slice, len) = if self.len + rhs.len() > SIZE {
            let available_len = SIZE - self.len;
            let slice = extract_utf8_within(rhs.as_bytes(), available_len);
            (slice, SIZE)
        } else {
            let len = self.len + rhs.len();
            let slice = rhs.as_bytes();
            (slice, len)
        };

        inner[self.len..len].copy_from_slice(slice);

        Self { inner, len }
    }
}

impl<const SIZE: usize, const LEN: usize> Add<RocStr<LEN>> for RocStr<SIZE> {
    type Output = Self;

    fn add(self, rhs: RocStr<LEN>) -> Self::Output {
        let mut inner = self.inner;
        let (slice, len) = if self.len + rhs.len > SIZE {
            let available_len = SIZE - self.len;
            let slice = extract_utf8_within(&rhs.inner, available_len);
            (slice, SIZE)
        } else {
            let len = self.len + rhs.len;
            let slice = &rhs.inner[..rhs.len];
            (slice, len)
        };

        inner[self.len..len].copy_from_slice(slice);

        Self { inner, len }
    }
}

/// Extract a valid utf-8 string from this byte array with at most `len` bytes.
///
/// # Examples
/// let s = "Löwe 老虎 Léopard";
///
/// /* first byte of `ö` is not utf-8 boundary */
/// assert_eq!("L", extract_utf8_within(s.as_bytes(), 2));
///
/// /* second byte of `老`is not utf-8 boundary */
/// assert_eq!("Löwe ", extract_utf8_within(s.as_bytes(), 8));
#[inline]
#[must_use]
fn extract_utf8_within(bytes: &[u8], len: usize) -> &[u8] {
    let bytes_len = bytes.len();
    let boundary = if len < bytes_len {
        // This is bit magic equivalent to: b < 128 || b >= 192
        if (bytes[len] as i8) >= -0x40 {
            len
        } else {
            let mut boundary = len - 1;
            // This is bit magic equivalent to: b >= 128 && b <= 192
            while (bytes[boundary] as i8) < -0x40 {
                boundary -= 1;
            }
            boundary
        }
    } else {
        bytes_len
    };

    &bytes[..boundary]
}

trait Zero {
    fn zero() -> Self;

    fn zero_as_rocstr<const SIZE: usize>() -> RocStr<SIZE> {
        let len = 1;
        let mut inner = [b' '; SIZE];
        inner[0] = b'0';
        RocStr { inner, len }
    }
}

trait Ten {
    fn ten() -> Self;
}

trait AsDigit {
    fn as_digit(&self) -> u8;
}

const ROCSTR_MIN_I8: RocStr<4> = RocStr {
    inner: *b"-128",
    len: 4,
};
const ROCSTR_MIN_I16: RocStr<6> = RocStr {
    inner: *b"-32768",
    len: 6,
};
const ROCSTR_MIN_I32: RocStr<11> = RocStr {
    inner: *b"-2147483648",
    len: 11,
};
const ROCSTR_MIN_I64: RocStr<20> = RocStr {
    inner: *b"-9223372036854775808",
    len: 20,
};
const ROCSTR_MIN_ISIZE: RocStr<20> = RocStr {
    inner: *b"-9223372036854775808",
    len: 20,
};

fn next_char<T>(value: T) -> (T, u8)
where
    T: Copy + Eq + Div<Output = T> + Mul<Output = T> + Sub<Output = T> + Zero + Ten + AsDigit,
{
    let next = value / T::ten();
    let mask = next * T::ten();
    let digit = (value - mask).as_digit();
    let char = match digit {
        0 => b'0',
        1 => b'1',
        2 => b'2',
        3 => b'3',
        4 => b'4',
        5 => b'5',
        6 => b'6',
        7 => b'7',
        8 => b'8',
        9 => b'9',
        // Unreachable beaucause digit is the remainder of the division by 10
        _ => unreachable!(),
    };

    (next, char)
}

fn from_signed<const SIZE: usize, T>(value: T) -> RocStr<SIZE>
where
    T: Copy
        + Eq
        + Neg<Output = T>
        + Ord
        + Div<Output = T>
        + Mul<Output = T>
        + Sub<Output = T>
        + Zero
        + Ten
        + AsDigit,
{
    if value == T::zero() {
        T::zero_as_rocstr()
    } else {
        let mut value = value;
        let mut len = 0;
        let mut buffer = [b' '; SIZE];

        // Backup the sign
        let negative = value < T::zero();
        // Get absolute value
        if negative {
            value = -value;
        }

        while value > T::zero() {
            len += 1;
            let (next, char) = next_char(value);
            buffer[SIZE - len] = char;
            value = next;
        }

        // Add the sign at the beginning
        if negative {
            len += 1;
            buffer[SIZE - len] = b'-';
        }

        let mut inner = [b' '; SIZE];
        inner[..len].copy_from_slice(&buffer[SIZE - len..]);

        RocStr { inner, len }
    }
}

fn from_unsigned<const SIZE: usize, T>(value: T) -> RocStr<SIZE>
where
    T: Copy + Eq + Ord + Div<Output = T> + Mul<Output = T> + Sub<Output = T> + Zero + Ten + AsDigit,
{
    if value == T::zero() {
        T::zero_as_rocstr()
    } else {
        let mut value = value;
        let mut len = 0;
        let mut buffer = [b' '; SIZE];
        while value > T::zero() {
            len += 1;
            let (next, char) = next_char(value);
            buffer[SIZE - len] = char;
            value = next;
        }
        let mut inner = [b' '; SIZE];
        inner[..len].copy_from_slice(&buffer[SIZE - len..]);

        RocStr { inner, len }
    }
}

impl Zero for u8 {
    fn zero() -> Self {
        0
    }
}

impl Zero for u16 {
    fn zero() -> Self {
        0
    }
}

impl Zero for u32 {
    fn zero() -> Self {
        0
    }
}

impl Zero for u64 {
    fn zero() -> Self {
        0
    }
}

impl Zero for u128 {
    fn zero() -> Self {
        0
    }
}

impl Zero for usize {
    fn zero() -> Self {
        0
    }
}

impl Zero for i8 {
    fn zero() -> Self {
        0
    }
}

impl Zero for i16 {
    fn zero() -> Self {
        0
    }
}

impl Zero for i32 {
    fn zero() -> Self {
        0
    }
}

impl Zero for i64 {
    fn zero() -> Self {
        0
    }
}

impl Zero for isize {
    fn zero() -> Self {
        0
    }
}

impl Ten for u8 {
    fn ten() -> Self {
        10
    }
}

impl Ten for u16 {
    fn ten() -> Self {
        10
    }
}

impl Ten for u32 {
    fn ten() -> Self {
        10
    }
}

impl Ten for u64 {
    fn ten() -> Self {
        10
    }
}

impl Ten for u128 {
    fn ten() -> Self {
        10
    }
}

impl Ten for usize {
    fn ten() -> Self {
        10
    }
}

impl Ten for i8 {
    fn ten() -> Self {
        10
    }
}

impl Ten for i16 {
    fn ten() -> Self {
        10
    }
}

impl Ten for i32 {
    fn ten() -> Self {
        10
    }
}

impl Ten for i64 {
    fn ten() -> Self {
        10
    }
}

impl Ten for isize {
    fn ten() -> Self {
        10
    }
}

impl AsDigit for u8 {
    fn as_digit(&self) -> u8 {
        *self
    }
}

impl AsDigit for u16 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for u32 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for u64 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for u128 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for usize {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for i8 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for i16 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for i32 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for i64 {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl AsDigit for isize {
    fn as_digit(&self) -> u8 {
        *self as u8
    }
}

impl From<u8> for RocStr<3> {
    fn from(value: u8) -> Self {
        from_unsigned(value)
    }
}

impl From<u16> for RocStr<5> {
    fn from(value: u16) -> Self {
        from_unsigned(value)
    }
}

impl From<u32> for RocStr<10> {
    fn from(value: u32) -> Self {
        from_unsigned(value)
    }
}

impl From<u64> for RocStr<20> {
    fn from(value: u64) -> Self {
        from_unsigned(value)
    }
}

impl From<u128> for RocStr<39> {
    fn from(value: u128) -> Self {
        from_unsigned(value)
    }
}

impl From<usize> for RocStr<20> {
    fn from(value: usize) -> Self {
        from_unsigned(value)
    }
}

impl From<i8> for RocStr<4> {
    fn from(value: i8) -> Self {
        if value == i8::MIN {
            ROCSTR_MIN_I8
        } else {
            from_signed(value)
        }
    }
}

impl From<i16> for RocStr<6> {
    fn from(value: i16) -> Self {
        if value == i16::MIN {
            ROCSTR_MIN_I16
        } else {
            from_signed(value)
        }
    }
}

impl From<i32> for RocStr<11> {
    fn from(value: i32) -> Self {
        if value == i32::MIN {
            ROCSTR_MIN_I32
        } else {
            from_signed(value)
        }
    }
}

impl From<i64> for RocStr<20> {
    fn from(value: i64) -> Self {
        if value == i64::MIN {
            ROCSTR_MIN_I64
        } else {
            from_signed(value)
        }
    }
}

impl From<isize> for RocStr<20> {
    fn from(value: isize) -> Self {
        if value == isize::MIN {
            ROCSTR_MIN_ISIZE
        } else {
            from_signed(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_could_be_compared_to_rocstr() {
        let s = RocStr::<16>::from("foo");
        assert!("foo" == s);
    }

    #[test]
    fn rocstr_as_str_should_be_inner_str() {
        let s = RocStr::<16>::from("foo");
        assert_eq!(s.as_str(), "foo");
    }

    #[test]
    fn rocstr_should_equal_inner_str() {
        let s = RocStr::<16>::from("foo");
        assert_eq!(s, "foo");
    }

    #[test]
    fn rocstr_ref_should_equal_inner_str() {
        let s = RocStr::<16>::from("foo");
        assert_eq!(&s, "foo");
    }

    #[test]
    fn rocstr_capacity_should_be_its_generic_parameter_size() {
        let string = RocStr::<16>::from("");
        assert_eq!(string.capacity(), 16);
    }

    #[test]
    fn empty_rocstr_should_say_it_is_empty() {
        let s = RocStr::<16>::from("");
        assert!(s.is_empty());
    }

    #[test]
    fn not_empty_rocstr_should_say_it_is_not_empty() {
        let s = RocStr::<16>::from("foo");
        assert!(!s.is_empty());
    }

    #[test]
    fn rocstr_len_should_count_the_number_of_bytes() {
        let s = RocStr::<16>::from("foo");
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn reshaped_rocstr_should_have_the_new_capacity() {
        let s = RocStr::<16>::from("foo");
        assert_eq!(s.reshape::<8>().capacity(), 8);
    }

    #[test]
    fn rocstr_starts_with_should_return_true_if_it_starts_with() {
        let bananas = RocStr::<16>::from("bananas");
        assert!(bananas.starts_with("bana"));
    }

    #[test]
    fn rocstr_starts_with_should_return_false_if_it_does_not_start_with() {
        let bananas = RocStr::<16>::from("bananas");
        assert!(!bananas.starts_with("nana"));
    }

    #[test]
    fn bytes_from_rocstr_should_be_the_bytes_of_the_inner_str() {
        let s = RocStr::<16>::from("foo");
        let bytes: &[u8] = (&s).into();
        assert_eq!(bytes, b"foo");
    }

    #[test]
    fn rocstr_as_bytes_should_be_the_bytes_of_the_inner_str() {
        let s = RocStr::<16>::from("foo");
        let bytes = s.as_bytes();
        assert_eq!(bytes, b"foo");
    }

    #[test]
    fn concat_two_rocstrs_first_with_enough_capacity_should_be_the_concateenation() {
        let s = RocStr::<32>::from("Löwe 老虎 ");
        let t = RocStr::<16>::from("Léopard Gepardi");
        let expected = RocStr::<32>::from("Löwe 老虎 Léopard Gepardi");

        assert_eq!(s + t, expected);
    }

    #[test]
    fn concat_two_rocstrs_first_without_enough_capacity_should_be_trimmed_concateenation() {
        let s = RocStr::<16>::from("Löwe 老虎 ");
        let t = RocStr::<16>::from("Léopard Gepardi");
        let expected = RocStr::<16>::from("Löwe 老虎 Lé");

        assert_eq!(s + t, expected);
    }

    #[test]
    fn concat_a_str_to_rocstr_first_with_enough_capacity_should_be_the_concateenation() {
        let s = RocStr::<32>::from("Löwe 老虎 ");
        let t = "Léopard Gepardi";
        let expected = RocStr::<32>::from("Löwe 老虎 Léopard Gepardi");

        assert_eq!(s + t, expected);
    }

    #[test]
    fn concat_a_str_to_rocstr_first_without_enough_capacity_should_be_trimmed_concateenation() {
        let s = RocStr::<16>::from("Löwe 老虎 ");
        let t = "Léo";
        let expected = RocStr::<16>::from("Löwe 老虎 Lé");

        assert_eq!(s + t, expected);

        let s = RocStr::<16>::from("Löwe 老虎 ");
        let t = "Léopard Gepardi";
        let expected = RocStr::<16>::from("Löwe 老虎 Lé");

        assert_eq!(s + t, expected);
    }

    #[test]
    fn concat_an_empty_rocstr_to_rocstr_should_be_a_noop() {
        let s = RocStr::<32>::from("Löwe 老虎 ");
        let t = RocStr::<32>::default();
        let expected = RocStr::<32>::from("Löwe 老虎 ");

        assert_eq!(s + t, expected);
    }

    #[test]
    fn concat_an_empty_str_to_rocstr_should_be_a_noop() {
        let s = RocStr::<32>::from("Löwe 老虎 ");
        let t = "";
        let expected = RocStr::<32>::from("Löwe 老虎 ");

        assert_eq!(s + t, expected);
    }

    #[test]
    fn extract_utf8_within_should_return_the_string_if_len_is_greater_than() {
        let s = "Löwe 老虎 Léopard";
        assert_eq!(extract_utf8_within(s.as_bytes(), 32), s.as_bytes());
    }

    #[test]
    fn extract_utf8_within_should_a_valid_utf8_with_len_lower_or_eq_than_len() {
        let s = "Löwe 老虎 Léopard";
        let len = 8;
        let extracted = extract_utf8_within(s.as_bytes(), len);
        assert_eq!(extracted, "Löwe ".as_bytes());
        assert!(extracted.len() <= len);
    }

    #[test]
    fn convert_max_u8_to_rocstr_should_be_max_u8_as_str() {
        let expected = "255";
        let converted = RocStr::from(u8::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_u16_to_rocstr_should_be_max_u16_as_str() {
        let expected = "65535";
        let converted = RocStr::from(u16::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_u32_to_rocstr_should_be_max_u32_as_str() {
        let expected = "4294967295";
        let converted = RocStr::from(u32::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_u64_to_rocstr_should_be_max_u64_as_str() {
        let expected = "18446744073709551615";
        let converted = RocStr::from(u64::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_u128_to_rocstr_should_be_max_u128_as_str() {
        let expected = "340282366920938463463374607431768211455";
        let converted = RocStr::from(u128::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_usize_to_rocstr_should_be_max_usize_as_str() {
        let expected = "18446744073709551615";
        let converted = RocStr::from(usize::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_i8_to_rocstr_should_be_max_i8_as_str() {
        let expected = "127";
        let converted = RocStr::from(i8::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_i16_to_rocstr_should_be_max_i16_as_str() {
        let expected = "32767";
        let converted = RocStr::from(i16::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_i32_to_rocstr_should_be_max_i32_as_str() {
        let expected = "2147483647";
        let converted = RocStr::from(i32::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_i64_to_rocstr_should_be_max_i64_as_str() {
        let expected = "9223372036854775807";
        let converted = RocStr::from(i64::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_max_isize_to_rocstr_should_be_max_isize_as_str() {
        let expected = "9223372036854775807";
        let converted = RocStr::from(isize::MAX);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_u8_to_rocstr_should_be_min_u8_as_str() {
        let expected = "0";
        let converted = RocStr::from(u8::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_u16_to_rocstr_should_be_min_u16_as_str() {
        let expected = "0";
        let converted = RocStr::from(u16::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_u32_to_rocstr_should_be_min_u32_as_str() {
        let expected = "0";
        let converted = RocStr::from(u32::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_u64_to_rocstr_should_be_min_u64_as_str() {
        let expected = "0";
        let converted = RocStr::from(u64::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_u128_to_rocstr_should_be_min_u128_as_str() {
        let expected = "0";
        let converted = RocStr::from(u128::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_usize_to_rocstr_should_be_min_usize_as_str() {
        let expected = "0";
        let converted = RocStr::from(usize::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_i8_to_rocstr_should_be_min_i8_as_str() {
        let expected = "-128";
        let converted = RocStr::from(i8::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_i16_to_rocstr_should_be_min_i16_as_str() {
        let expected = "-32768";
        let converted = RocStr::from(i16::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_i32_to_rocstr_should_be_min_i32_as_str() {
        let expected = "-2147483648";
        let converted = RocStr::from(i32::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_i64_to_rocstr_should_be_min_i64_as_str() {
        let expected = "-9223372036854775808";
        let converted = RocStr::from(i64::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_min_isize_to_rocstr_should_be_min_isize_as_str() {
        let expected = "-9223372036854775808";
        let converted = RocStr::from(isize::MIN);

        assert_eq!(converted, expected);
    }

    #[test]
    fn convert_zero_i8_to_rocstr_should_be_str_zero() {
        let converted = RocStr::from(0i8);
        assert_eq!(converted, "0");
    }

    #[test]
    fn convert_zero_i16_to_rocstr_should_be_str_zero() {
        let converted = RocStr::from(0i16);
        assert_eq!(converted, "0");
    }

    #[test]
    fn convert_zero_i32_to_rocstr_should_be_str_zero() {
        let converted = RocStr::from(0i32);
        assert_eq!(converted, "0");
    }

    #[test]
    fn convert_zero_i64_to_rocstr_should_be_str_zero() {
        let converted = RocStr::from(0i64);
        assert_eq!(converted, "0");
    }

    #[test]
    fn convert_zero_isize_to_rocstr_should_be_str_zero() {
        let converted = RocStr::from(0isize);
        assert_eq!(converted, "0");
    }

    #[test]
    fn convert_negative_i8_to_rocstr_should_start_with_a_minus_sign() {
        let converted = RocStr::from(-42i8);

        assert_eq!(converted.inner[0], b'-');
        assert_eq!(converted, "-42");
    }

    #[test]
    fn convert_negative_i16_to_rocstr_should_start_with_a_minus_sign() {
        let converted = RocStr::from(-42i16);

        assert_eq!(converted.inner[0], b'-');
        assert_eq!(converted, "-42");
    }

    #[test]
    fn convert_negative_i32_to_rocstr_should_start_with_a_minus_sign() {
        let converted = RocStr::from(-42i32);

        assert_eq!(converted.inner[0], b'-');
        assert_eq!(converted, "-42");
    }

    #[test]
    fn convert_negative_i64_to_rocstr_should_start_with_a_minus_sign() {
        let converted = RocStr::from(-42i64);

        assert_eq!(converted.inner[0], b'-');
        assert_eq!(converted, "-42");
    }

    #[test]
    fn convert_negative_isize_to_rocstr_should_start_with_a_minus_sign() {
        let converted = RocStr::from(-42isize);

        assert_eq!(converted.inner[0], b'-');
        assert_eq!(converted, "-42");
    }

    #[test]
    fn rocerr_debug_info_should_display_inner_field_as_str() {
        extern crate std;
        use std::format;
        use std::string::ToString;

        let s = RocStr::<16>::from("foo");
        assert_eq!(
            format!("{s:?}"),
            "RocStr { inner: \"foo\", len: 3 }".to_string()
        );
    }

    #[test]
    fn hash_rocstr_should_be_the_hash_of_the_inner_str() {
        extern crate std;
        use std::hash::{DefaultHasher, Hasher};

        let mut hasher = DefaultHasher::new();
        "foo".hash(&mut hasher);
        let exptected = hasher.finish();

        let mut hasher = DefaultHasher::new();
        let s = RocStr::<16>::from("foo");
        s.hash(&mut hasher);
        let hash = hasher.finish();

        assert_eq!(hash, exptected);
    }

    #[test]
    fn replace_an_str_at_the_begining_of_a_rocstr_should_be_the_rocstr_with_str_replaced() {
        let s = RocStr::<16>::from("this is old");
        assert_eq!(RocStr::<16>::from("that is old"), s.replace("this", "that"));
    }

    #[test]
    fn replace_an_str_at_the_end_of_a_rocstr_should_be_the_rocstr_with_str_replaced() {
        let s = RocStr::<16>::from("this is old");
        assert_eq!(RocStr::<16>::from("this is new"), s.replace("old", "new"));
    }

    #[test]
    fn replace_an_str_in_a_rocstr_should_be_the_rocstr_with_str_replaced() {
        let s = RocStr::<16>::from("this is old");
        assert_eq!(RocStr::<16>::from("than an old"), s.replace("is", "an"));
    }

    #[test]
    fn replace_an_str_in_a_rocstr_that_overflow_should_be_the_truncated_str_replaced() {
        let s = RocStr::<16>::from("this is old");
        let replaced = s.replace("old", "obvously overflowing");

        assert!(
            replaced.len <= replaced.capacity(),
            "Len of replaced rocstr is greater than its capacity"
        );

        assert_eq!(RocStr::<16>::from("this is obvously"), replaced);
    }

    #[test]
    fn replace_an_str_inside_a_rocstr_that_overflow_should_be_the_truncated_str_replaced() {
        let s = RocStr::<16>::from("this is old");
        let replaced = s.replace("is", "is obvously");

        assert!(
            replaced.len <= replaced.capacity(),
            "Len of replaced rocstr is greater than its capacity"
        );

        assert_eq!(RocStr::<16>::from("this obvously is"), replaced);
    }

    #[test]
    fn replace_an_str_inside_a_rocstr_that_overflow_at_last_should_be_the_truncated_str_replaced() {
        let s = RocStr::<16>::from("this is old");
        let replaced = s.replace(" is", " is obvously");

        assert!(
            replaced.len <= replaced.capacity(),
            "Len of replaced rocstr is greater than its capacity"
        );

        assert_eq!(RocStr::<16>::from("this is obvously is"), replaced);
    }

    #[test]
    fn truncate_rocstr_should_contain_a_valid_utf8_with_at_most_len_bytes() {
        let s = RocStr::<32>::from("Löwe 老虎 Léopard");

        /* second byte of `老`is not utf-8 boundary */
        assert_eq!(s.truncate(8), "Löwe ");
    }
}
