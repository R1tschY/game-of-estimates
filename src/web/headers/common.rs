use std::fmt::{Debug, Display, Formatter, Write};
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub struct Weighted<T> {
    value: T,
    weight: QValue,
}

impl<T> Weighted<T> {
    pub fn new(value: T, weight: QValue) -> Self {
        Self { value, weight }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn weight(&self) -> QValue {
        self.weight
    }
}

/// Quality Value
///
/// Standard: https://httpwg.org/specs/rfc9110.html#quality.values
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct QValue(u16);

impl QValue {
    pub fn unacceptable() -> Self {
        QValue(0)
    }

    pub fn max() -> Self {
        QValue(1000)
    }

    pub fn is_unacceptable(&self) -> bool {
        self.0 == 0
    }

    pub fn is_acceptable(&self) -> bool {
        self.0 != 0
    }

    pub(crate) fn parse_weight(s: &[u8]) -> Option<(&[u8], QValue)> {
        let (s, _) = ows(s)?;
        let (s, _) = c(s, b';')?;
        let (s, _) = ows(s)?;
        let (s, _) = c2(s, b'q', b'Q')?;
        let (s, _) = c(s, b'=')?;
        Self::parse(s)
    }

    fn parse(s: &[u8]) -> Option<(&[u8], QValue)> {
        if let Some((s, _)) = c(s, b'0') {
            let mut v = 0u16;
            let mut rs = s;
            if let Some((s, _)) = c(s, b'.') {
                rs = s;
                if let Some((s, d)) = digit(s) {
                    v += d as u16 * 100u16;
                    rs = s;
                    if let Some((s, d)) = digit(s) {
                        v += d as u16 * 10u16;
                        rs = s;
                        if let Some((s, d)) = digit(s) {
                            v += d as u16;
                            rs = s;
                        }
                    }
                }
            }
            Some((rs, QValue(v)))
        } else if let Some((s, _)) = c(s, b'1') {
            let mut rs = s;
            if let Some((s, _)) = c(s, b'.') {
                rs = s;
                if let Some((s, _)) = c(s, b'0') {
                    rs = s;
                    if let Some((s, _)) = c(s, b'0') {
                        rs = s;
                        if let Some((s, _)) = c(s, b'0') {
                            rs = s;
                        }
                    }
                }
            }
            Some((rs, QValue::default()))
        } else {
            None
        }
    }
}

pub(crate) fn ows(s: &[u8]) -> Option<(&[u8], ())> {
    let n = s
        .iter()
        .copied()
        .take_while(|&c| c == b' ' || c == b'\t')
        .count();
    Some((s.split_at(n).1, ()))
}

pub(crate) fn c(s: &[u8], lit: u8) -> Option<(&[u8], ())> {
    match s.split_first() {
        Some((c, s)) if *c == lit => Some((s, ())),
        _ => None,
    }
}

pub(crate) fn c2(s: &[u8], lit1: u8, lit2: u8) -> Option<(&[u8], ())> {
    match s.split_first() {
        Some((c, s)) if *c == lit1 || *c == lit2 => Some((s, ())),
        _ => None,
    }
}

pub(crate) fn digit(s: &[u8]) -> Option<(&[u8], u8)> {
    match s.split_first() {
        Some((c, s)) if c.is_ascii_digit() => Some((s, c - b'0')),
        _ => None,
    }
}

const O: bool = false;
const X: bool = true;
const TCHAR: &[bool; 128] = &[
    O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O, O,
    O, X, O, X, X, X, X, X, O, O, X, X, O, X, X, O, X, X, X, X, X, X, X, X, X, X, O, O, O, O, O, O,
    O, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, O, O, O, X, X,
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, O, X, O, X, O,
];

pub(crate) fn tchar_pred(c: u8) -> bool {
    TCHAR.get(c as usize).copied() == Some(true)
}

pub(crate) fn tchar(s: &[u8]) -> Option<(&[u8], ())> {
    match s.split_first() {
        Some((c, s)) if tchar_pred(*c) => Some((s, ())),
        _ => None,
    }
}

pub(crate) fn token(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let n = s.iter().copied().take_while(|c| tchar_pred(*c)).count();
    if n >= 1 {
        let (res, s) = s.split_at(n);
        Some((s, res))
    } else {
        None
    }
}

impl Default for QValue {
    fn default() -> Self {
        QValue(1000)
    }
}

impl FromStr for QValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s.as_bytes())
            .filter(|(s, _)| s.is_empty())
            .map(|(_, res)| res)
            .ok_or(())
    }
}

impl TryFrom<f32> for QValue {
    type Error = ();

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < 0.0 || value > 1.0 {
            Err(())
        } else {
            Ok(Self((value * 1000.0).round() as u16))
        }
    }
}

impl Display for QValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char((b'0' + (self.0 / 1000) as u8) as char)?;
        let _a = self.0 % 1000;
        if _a != 0 {
            f.write_char('.')?;
            f.write_char((b'0' + (_a / 100) as u8) as char)?;
            let _b = _a % 100;
            if _b != 0 {
                f.write_char((b'0' + (_b / 10) as u8) as char)?;
                let d = _b % 10;
                if d != 0 {
                    f.write_char((b'0' + d as u8) as char)?;
                }
            }
        }

        Ok(())
    }
}

impl Debug for QValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_qvalue() {
        assert_eq!(Ok(QValue(1000)), QValue::from_str("1"));
        assert_eq!(Ok(QValue(1000)), QValue::from_str("1."));
        assert_eq!(Ok(QValue(1000)), QValue::from_str("1.0"));
        assert_eq!(Ok(QValue(1000)), QValue::from_str("1.00"));
        assert_eq!(Ok(QValue(1000)), QValue::from_str("1.000"));

        assert_eq!(Ok(QValue(0000)), QValue::from_str("0"));
        assert_eq!(Ok(QValue(0000)), QValue::from_str("0."));
        assert_eq!(Ok(QValue(0000)), QValue::from_str("0.0"));
        assert_eq!(Ok(QValue(0000)), QValue::from_str("0.00"));
        assert_eq!(Ok(QValue(0000)), QValue::from_str("0.000"));

        assert_eq!(Ok(QValue(0100)), QValue::from_str("0.1"));
        assert_eq!(Ok(QValue(0900)), QValue::from_str("0.9"));
        assert_eq!(Ok(QValue(0010)), QValue::from_str("0.01"));
        assert_eq!(Ok(QValue(0001)), QValue::from_str("0.001"));
    }

    #[test]
    fn display_qvalue() {
        assert_eq!("0", QValue(0).to_string());
        assert_eq!("1", QValue(1000).to_string());
        assert_eq!("0.1", QValue(100).to_string());
        assert_eq!("0.01", QValue(10).to_string());
        assert_eq!("0.001", QValue(1).to_string());
    }
}
