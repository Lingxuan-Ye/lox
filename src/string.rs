use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Deref, Index};
use std::range::Range;
use std::rc::Rc;
use std::slice::SliceIndex;

#[derive(Clone)]
pub struct LoxString<'a> {
    repr: Repr<'a>,
}

#[derive(Debug, Clone)]
enum Repr<'a> {
    Borrowed(&'a str),
    /// Invariant: the bytes must be valid UTF-8.
    Shared(Rc<[u8]>),
}

impl<'a> LoxString<'a> {
    pub fn unescape(raw: &'a str) -> Result<Self, Range<usize>> {
        let Some(mut cursor) = raw.find('\\') else {
            let string = Self::from(raw);
            return Ok(string);
        };

        let raw_len = raw.len();
        let mut string = String::with_capacity(raw_len);

        let prefix = &raw[..cursor];
        string.push_str(prefix);

        loop {
            let remaining = &raw[cursor..];
            let remaining_bytes = remaining.as_bytes();
            let Some(byte_1) = remaining_bytes.get(1) else {
                // This branch is unreachable when called from `Parser::primary`. It exists only
                // for correctness should this method ever be called directly, even though it is
                // not intended to.
                let start = cursor;
                let end = raw_len;
                let range = Range { start, end };
                return Err(range);
            };
            let (unescaped, sequence_len) = match byte_1 {
                b'0' => ('\0', 2),
                b't' => ('\t', 2),
                b'n' => ('\n', 2),
                b'r' => ('\r', 2),
                b'"' => ('"', 2),
                b'\\' => ('\\', 2),
                b'x' => {
                    let Some(bytes) = remaining_bytes.get(2..4) else {
                        let start = cursor;
                        let end = raw_len;
                        let range = Range { start, end };
                        return Err(range);
                    };
                    let mut code = 0;
                    for (offset, byte) in (2..).zip(bytes) {
                        match byte {
                            b'0'..=b'9' => {
                                code = (code << 4) | (byte - b'0');
                            }
                            b'a'..=b'f' => {
                                code = (code << 4) | (byte - b'a' + 10);
                            }
                            b'A'..=b'F' => {
                                code = (code << 4) | (byte - b'A' + 10);
                            }
                            _ => {
                                let char_len = byte.leading_ones().max(1) as usize;
                                let start = cursor;
                                let end = cursor + offset + char_len;
                                let range = Range { start, end };
                                return Err(range);
                            }
                        }
                    }
                    if code > 0x7F {
                        let start = cursor;
                        let end = cursor + 4;
                        let range = Range { start, end };
                        return Err(range);
                    }
                    (code as char, 4)
                }
                b'u' => {
                    let [byte_2, byte_3, ..] = &remaining_bytes[2..] else {
                        let start = cursor;
                        let end = raw_len;
                        let range = Range { start, end };
                        return Err(range);
                    };
                    if *byte_2 != b'{' {
                        let char_len = byte_2.leading_ones().max(1) as usize;
                        let start = cursor;
                        let end = cursor + 2 + char_len;
                        let range = Range { start, end };
                        return Err(range);
                    }
                    let mut code = match byte_3 {
                        b'0'..=b'9' => (byte_3 - b'0') as u32,
                        b'a'..=b'f' => (byte_3 - b'a' + 10) as u32,
                        b'A'..=b'F' => (byte_3 - b'A' + 10) as u32,
                        _ => {
                            let char_len = byte_3.leading_ones().max(1) as usize;
                            let start = cursor;
                            let end = cursor + 3 + char_len;
                            let range = Range { start, end };
                            return Err(range);
                        }
                    };
                    let mut offset = 4;
                    loop {
                        let Some(byte) = remaining_bytes.get(offset) else {
                            let start = cursor;
                            let end = raw_len;
                            let range = Range { start, end };
                            return Err(range);
                        };
                        if offset == 10 {
                            let char_len = byte.leading_ones().max(1) as usize;
                            let start = cursor;
                            let end = cursor + offset + char_len;
                            let range = Range { start, end };
                            return Err(range);
                        }
                        match byte {
                            b'0'..=b'9' => {
                                code = (code << 4) | (byte - b'0') as u32;
                            }
                            b'a'..=b'f' => {
                                code = (code << 4) | (byte - b'a' + 10) as u32;
                            }
                            b'A'..=b'F' => {
                                code = (code << 4) | (byte - b'A' + 10) as u32;
                            }
                            b'}' => {
                                let sequence_len = offset + 1;
                                let Some(char) = char::from_u32(code) else {
                                    let start = cursor;
                                    let end = cursor + sequence_len;
                                    let range = Range { start, end };
                                    return Err(range);
                                };
                                break (char, sequence_len);
                            }
                            _ => {
                                let char_len = byte.leading_ones().max(1) as usize;
                                let start = cursor;
                                let end = cursor + offset + char_len;
                                let range = Range { start, end };
                                return Err(range);
                            }
                        }
                        offset += 1;
                    }
                }
                _ => {
                    let char_len = byte_1.leading_ones().max(1) as usize;
                    let start = cursor;
                    let end = cursor + 1 + char_len;
                    let range = Range { start, end };
                    return Err(range);
                }
            };

            let remaining = &remaining[sequence_len..];
            match remaining.find('\\') {
                None => {
                    let plain_text = remaining;
                    string.push(unescaped);
                    string.push_str(plain_text);
                    let string = Self::from(string);
                    return Ok(string);
                }
                Some(offset) => {
                    let plain_text = &remaining[..offset];
                    string.push(unescaped);
                    string.push_str(plain_text);
                    cursor += sequence_len + offset;
                }
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.repr {
            Repr::Borrowed(string) => string,
            Repr::Shared(bytes) => unsafe { str::from_utf8_unchecked(bytes) },
        }
    }

    fn is_borrowed(&self) -> bool {
        matches!(self.repr, Repr::Borrowed(_))
    }
}

impl fmt::Debug for LoxString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Display for LoxString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Deref for LoxString<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for LoxString<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for LoxString<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Borrow<str> for LoxString<'_> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for LoxString<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl PartialEq<str> for LoxString<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<&str> for LoxString<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(*other)
    }
}

impl PartialEq<&mut str> for LoxString<'_> {
    fn eq(&self, other: &&mut str) -> bool {
        self.as_str().eq(*other)
    }
}

impl PartialEq<LoxString<'_>> for str {
    fn eq(&self, other: &LoxString<'_>) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq<LoxString<'_>> for &str {
    fn eq(&self, other: &LoxString<'_>) -> bool {
        self[..].eq(other.as_str())
    }
}

impl PartialEq<LoxString<'_>> for &mut str {
    fn eq(&self, other: &LoxString<'_>) -> bool {
        self[..].eq(other.as_str())
    }
}

impl Eq for LoxString<'_> {}

impl PartialOrd for LoxString<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<str> for LoxString<'_> {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd<LoxString<'_>> for str {
    fn partial_cmp(&self, other: &LoxString<'_>) -> Option<Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl Ord for LoxString<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<I> Index<I> for LoxString<'_>
where
    I: SliceIndex<str>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_str().index(index)
    }
}

impl Add for LoxString<'_> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.is_empty(), rhs.is_empty()) {
            (true, true) => match (self.is_borrowed(), rhs.is_borrowed()) {
                (true, true) => return self,
                (true, false) => return self,
                (false, true) => return rhs,
                (false, false) => return Self::from(""),
            },
            (true, false) => return rhs,
            (false, true) => return self,
            (false, false) => (),
        }

        let lhs_len = self.len();
        let rhs_len = rhs.len();

        let lhs = self.as_ptr();
        let rhs = rhs.as_ptr();

        unsafe {
            let len = lhs_len + rhs_len;
            let mut buf = Rc::new_uninit_slice(len);
            let dst = Rc::get_mut(&mut buf)
                .unwrap_unchecked()
                .as_mut_ptr()
                .cast::<u8>();
            dst.copy_from_nonoverlapping(lhs, lhs_len);
            dst.add(lhs_len).copy_from_nonoverlapping(rhs, rhs_len);
            let buf = buf.assume_init();
            let repr = Repr::Shared(buf);
            Self { repr }
        }
    }
}

impl<'a> From<&'a str> for LoxString<'a> {
    fn from(value: &'a str) -> Self {
        let repr = Repr::Borrowed(value);
        Self { repr }
    }
}

impl From<String> for LoxString<'_> {
    fn from(value: String) -> Self {
        let bytes = value.as_bytes();
        let bytes = Rc::from(bytes);
        let repr = Repr::Shared(bytes);
        Self { repr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape() {
        let raw = "\tHello,\n\tworld!\n";
        let unescaped = LoxString::unescape(raw).unwrap();
        let expected = raw;
        assert_eq!(unescaped, expected);
        assert!(unescaped.is_borrowed());

        let raw = r"\tHe\x6c\x6Co,\n\twor\u{006c}d!\n";
        let unescaped = LoxString::unescape(raw).unwrap();
        let expected = "\tHello,\n\tworld!\n";
        assert_eq!(unescaped, expected);
        assert!(!unescaped.is_borrowed());

        let raw = r"\\";
        let unescaped = LoxString::unescape(raw).unwrap();
        let expected = "\\";
        assert_eq!(unescaped, expected);
        assert!(!unescaped.is_borrowed());

        let raw = r"\";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 1 };
        assert_eq!(range, expected);

        let raw = r"\x";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 2 };
        assert_eq!(range, expected);

        let raw = r"\x0";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 3 };
        assert_eq!(range, expected);

        let raw = r"\x0文";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 6 };
        assert_eq!(range, expected);

        let raw = r"\xFF";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 4 };
        assert_eq!(range, expected);

        let raw = r"\x000";
        let unescaped = LoxString::unescape(raw).unwrap();
        let expected = "\x000";
        assert_eq!(unescaped, expected);
        assert!(!unescaped.is_borrowed());

        let raw = r"\u";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 2 };
        assert_eq!(range, expected);

        let raw = r"\u文";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 5 };
        assert_eq!(range, expected);

        let raw = r"\u{";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 3 };
        assert_eq!(range, expected);

        let raw = r"\u{0";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 4 };
        assert_eq!(range, expected);

        let raw = r"\u{0文";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 7 };
        assert_eq!(range, expected);

        let raw = r"\u{6587}";
        let unescaped = LoxString::unescape(raw).unwrap();
        let expected = "文";
        assert_eq!(unescaped, expected);
        assert!(!unescaped.is_borrowed());

        let raw = r"\u{006587}";
        let unescaped = LoxString::unescape(raw).unwrap();
        let expected = "文";
        assert_eq!(unescaped, expected);
        assert!(!unescaped.is_borrowed());

        let raw = r"\u{0006587}";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 11 };
        assert_eq!(range, expected);

        let raw = r"\u{110000}";
        let range = LoxString::unescape(raw).unwrap_err();
        let expected = Range { start: 0, end: 10 };
        assert_eq!(range, expected);
    }

    #[test]
    fn test_add() {
        const EMPTY: &str = "";

        let lhs = LoxString::from(EMPTY);
        let rhs = LoxString::from(EMPTY);
        let output = lhs + rhs;
        let expected = EMPTY;
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = LoxString::from(EMPTY);
        let rhs = String::from(EMPTY);
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = EMPTY;
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = String::from(EMPTY);
        let lhs = LoxString::from(lhs);
        let rhs = LoxString::from(EMPTY);
        let output = lhs + rhs;
        let expected = EMPTY;
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = String::from(EMPTY);
        let lhs = LoxString::from(lhs);
        let rhs = String::from(EMPTY);
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = EMPTY;
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = LoxString::from(EMPTY);
        let rhs = LoxString::from("rhs");
        let output = lhs + rhs;
        let expected = "rhs";
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = LoxString::from(EMPTY);
        let rhs = String::from("rhs");
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = "rhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());

        let lhs = String::from(EMPTY);
        let lhs = LoxString::from(lhs);
        let rhs = LoxString::from("rhs");
        let output = lhs + rhs;
        let expected = "rhs";
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = String::from(EMPTY);
        let lhs = LoxString::from(lhs);
        let rhs = String::from("rhs");
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = "rhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());

        let lhs = LoxString::from("lhs");
        let rhs = LoxString::from(EMPTY);
        let output = lhs + rhs;
        let expected = "lhs";
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = LoxString::from("lhs");
        let rhs = String::from(EMPTY);
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = "lhs";
        assert_eq!(output, expected);
        assert!(output.is_borrowed());

        let lhs = String::from("lhs");
        let lhs = LoxString::from(lhs);
        let rhs = LoxString::from(EMPTY);
        let output = lhs + rhs;
        let expected = "lhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());

        let lhs = String::from("lhs");
        let lhs = LoxString::from(lhs);
        let rhs = String::from(EMPTY);
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = "lhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());

        let lhs = LoxString::from("lhs");
        let rhs = LoxString::from("rhs");
        let output = lhs + rhs;
        let expected = "lhsrhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());

        let lhs = LoxString::from("lhs");
        let rhs = String::from("rhs");
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = "lhsrhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());

        let lhs = String::from("lhs");
        let lhs = LoxString::from(lhs);
        let rhs = LoxString::from("rhs");
        let output = lhs + rhs;
        let expected = "lhsrhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());

        let lhs = String::from("lhs");
        let lhs = LoxString::from(lhs);
        let rhs = String::from("rhs");
        let rhs = LoxString::from(rhs);
        let output = lhs + rhs;
        let expected = "lhsrhs";
        assert_eq!(output, expected);
        assert!(!output.is_borrowed());
    }
}
