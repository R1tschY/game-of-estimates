use crate::web::headers::common::{c, ows, token, QValue, Weighted};
use crate::web::headers::{Header, InvalidHeaderValue};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Coding {
    #[cfg(feature = "compress-gzip")]
    Gzip,
    #[cfg(feature = "compress-deflate")]
    Deflate,
    #[cfg(feature = "compress-brotli")]
    Brotli,
    #[cfg(feature = "compress-zstd")]
    Zstandard,
    Identity,
}

impl Display for Coding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Coding {
    pub fn as_str(&self) -> &'static str {
        match self {
            Coding::Identity => "identity",
            #[cfg(feature = "compress-gzip")]
            Coding::Gzip => "gzip",
            #[cfg(feature = "compress-deflate")]
            Coding::Deflate => "deflate",
            #[cfg(feature = "compress-brotli")]
            Coding::Brotli => "br",
            #[cfg(feature = "compress-zstd")]
            Coding::Zstandard => "zstd",
        }
    }

    fn slice_starts_with<'a>(input: &'a [u8], test: &[u8]) -> Option<&'a [u8]> {
        if input.len() >= test.len() {
            let (start, rest) = input.split_at(test.len());
            if start.eq_ignore_ascii_case(test) {
                return Some(rest);
            }
        }

        None
    }

    fn parse(input: &[u8]) -> Option<(&[u8], Self)> {
        #[cfg(feature = "compress-gzip")]
        if let Some(rest) = Self::slice_starts_with(input, b"gzip") {
            return Some((rest, Self::Gzip));
        }

        #[cfg(feature = "compress-deflate")]
        if let Some(rest) = Self::slice_starts_with(input, b"deflate") {
            return Some((rest, Self::Deflate));
        }

        #[cfg(feature = "compress-zstd")]
        if let Some(rest) = Self::slice_starts_with(input, b"zstd") {
            return Some((rest, Self::Zstandard));
        }

        #[cfg(feature = "compress-brotli")]
        if let Some(rest) = Self::slice_starts_with(input, b"br") {
            return Some((rest, Self::Brotli));
        }

        #[cfg(feature = "compress-gzip")]
        if let Some(rest) = Self::slice_starts_with(input, b"x-gzip") {
            return Some((rest, Self::Gzip));
        }

        if let Some(rest) = Self::slice_starts_with(input, b"identity") {
            return Some((rest, Self::Identity));
        }

        None
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum CodingDirective {
    Coding(Coding),
    Asterisk,
    Unknown,
}

impl CodingDirective {
    pub fn as_str(&self) -> &str {
        match self {
            CodingDirective::Coding(coding) => coding.as_str(),
            CodingDirective::Asterisk => "*",
            CodingDirective::Unknown => "<unknown>",
        }
    }

    fn parse(input: &[u8]) -> Option<(&[u8], Self)> {
        if let Some((s, _)) = c(input, b'*') {
            Some((s, CodingDirective::Asterisk))
        } else if let Some((s, r)) = Coding::parse(input) {
            Some((s, CodingDirective::Coding(r)))
        } else {
            let (s, _) = token(input)?;
            Some((s, CodingDirective::Unknown))
        }
    }
}

impl Display for CodingDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CodingDirective {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Self::parse(s.as_bytes()) {
            Some(([], res)) => Ok(res),
            _ => Err(()),
        }
    }
}

impl From<Coding> for CodingDirective {
    fn from(value: Coding) -> Self {
        CodingDirective::Coding(value)
    }
}

/// Accept-Encoding Header
///
/// Standard: https://httpwg.org/specs/rfc9110.html#field.accept-encoding
#[derive(PartialEq, Debug)]
pub struct AcceptEncoding {
    identity: Option<QValue>,
    #[cfg(feature = "compress-gzip")]
    gzip: Option<QValue>,
    #[cfg(feature = "compress-deflate")]
    deflate: Option<QValue>,
    #[cfg(feature = "compress-zstd")]
    zstd: Option<QValue>,
    #[cfg(feature = "compress-brotli")]
    brotli: Option<QValue>,
    asterisk: Option<QValue>,
}

impl Default for AcceptEncoding {
    fn default() -> Self {
        AcceptEncoding {
            identity: Some(QValue::default()),
            #[cfg(feature = "compress-gzip")]
            gzip: None,
            #[cfg(feature = "compress-deflate")]
            deflate: None,
            #[cfg(feature = "compress-zstd")]
            zstd: None,
            #[cfg(feature = "compress-brotli")]
            brotli: None,
            asterisk: None,
        }
    }
}

fn qagg(a: Option<QValue>, b: QValue) -> QValue {
    if let Some(a) = a {
        if a.is_unacceptable() || b.is_unacceptable() {
            QValue::unacceptable()
        } else if a >= b {
            a
        } else {
            b
        }
    } else {
        b
    }
}

impl AcceptEncoding {
    pub(crate) fn empty() -> Self {
        Self {
            identity: None,
            #[cfg(feature = "compress-gzip")]
            gzip: None,
            #[cfg(feature = "compress-deflate")]
            deflate: None,
            #[cfg(feature = "compress-zstd")]
            zstd: None,
            #[cfg(feature = "compress-brotli")]
            brotli: None,
            asterisk: None,
        }
    }

    pub fn match_one_of(&self, providing: &[Coding]) -> Option<Coding> {
        let mut res: Option<Coding> = None;
        let mut weight = QValue::unacceptable();

        for provided in providing {
            let q = self.find_qvalue(*provided);
            if q.is_unacceptable() {
                continue;
            } else if q == QValue::max() {
                return Some(*provided);
            } else if q > weight {
                weight = q;
                res = Some(*provided);
            }
        }

        // identity fallback
        if res.is_none() && self.is_identity_acceptable() {
            return Some(Coding::Identity);
        }

        res
    }

    fn find_qvalue(&self, provided: Coding) -> QValue {
        match provided {
            #[cfg(feature = "compress-gzip")]
            Coding::Gzip => {
                if let Some(gzip) = self.gzip {
                    return gzip;
                }
            }
            #[cfg(feature = "compress-deflate")]
            Coding::Deflate => {
                if let Some(deflate) = self.deflate {
                    return deflate;
                }
            }
            #[cfg(feature = "compress-zstd")]
            Coding::Zstandard => {
                if let Some(zstd) = self.zstd {
                    return zstd;
                }
            }
            #[cfg(feature = "compress-brotli")]
            Coding::Brotli => {
                if let Some(brotli) = self.brotli {
                    return brotli;
                }
            }
            Coding::Identity => {
                if let Some(identity) = self.identity {
                    return identity;
                }
            }
        }

        self.asterisk.unwrap_or(QValue::unacceptable())
    }

    fn is_identity_acceptable(&self) -> bool {
        if let Some(q) = self.identity {
            q.is_acceptable()
        } else if let Some(q) = self.asterisk {
            q.is_acceptable()
        } else {
            true
        }
    }

    fn parse(mut s: &[u8]) -> Option<(&[u8], Self)> {
        if s.is_empty() {
            return Some((s, AcceptEncoding::empty()));
        }

        let mut res = Self::empty();
        if let Some((is, r)) = Self::parse_weighted_codings(s) {
            res.agg(r);
            s = is;
        }

        while let Some((is, r)) = Self::parse_weighted_codings_(s) {
            res.agg(r);
            s = is;
        }

        Some((s, res))
    }

    fn agg(&mut self, value: Weighted<CodingDirective>) {
        match value.value() {
            CodingDirective::Coding(coding) => match coding {
                #[cfg(feature = "compress-gzip")]
                Coding::Gzip => {
                    self.gzip = Some(qagg(self.gzip, value.weight()));
                }
                #[cfg(feature = "compress-deflate")]
                Coding::Deflate => {
                    self.deflate = Some(qagg(self.deflate, value.weight()));
                }
                #[cfg(feature = "compress-zstd")]
                Coding::Zstandard => {
                    self.zstd = Some(qagg(self.zstd, value.weight()));
                }
                #[cfg(feature = "compress-brotli")]
                Coding::Brotli => {
                    self.brotli = Some(qagg(self.brotli, value.weight()));
                }
                Coding::Identity => {
                    self.identity = Some(qagg(self.identity, value.weight()));
                }
            },
            CodingDirective::Asterisk => {
                self.asterisk = Some(qagg(self.asterisk, value.weight()));
            }
            CodingDirective::Unknown => {}
        }
    }

    fn parse_weighted_codings(s: &[u8]) -> Option<(&[u8], Weighted<CodingDirective>)> {
        let (s, coding) = CodingDirective::parse(s)?;
        if let Some((s, weight)) = QValue::parse_weight(s) {
            Some((s, Weighted::new(coding, weight)))
        } else {
            Some((s, Weighted::new(coding, QValue::default())))
        }
    }

    fn parse_weighted_codings_(s: &[u8]) -> Option<(&[u8], Weighted<CodingDirective>)> {
        let (s, _) = ows(s)?;
        let (s, _) = c(s, b',')?;
        let (s, _) = ows(s)?;
        Self::parse_weighted_codings(s)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AcceptEncoding {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Accept-Encoding") {
            None => Outcome::Error((Status::BadRequest, ())),
            Some(value) => match AcceptEncoding::from_str(value) {
                Ok(res) => Outcome::Success(res),
                Err(_) => Outcome::Error((Status::BadRequest, ())),
            },
        }
    }
}

impl FromStr for AcceptEncoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s.as_bytes())
            .filter(|(s, _)| s.is_empty())
            .map(|(_, res)| res)
            .ok_or(())
    }
}

impl<'h> Header<'h> for AcceptEncoding {
    fn name() -> &'static str {
        "Accept-Encoding"
    }

    fn decode<I>(values: &mut I) -> Result<Self, InvalidHeaderValue>
    where
        Self: Sized,
        I: Iterator<Item = &'h str>,
    {
        match values.next() {
            Some(x) => x.parse().map_err(|_| InvalidHeaderValue),
            None => Ok(AcceptEncoding::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parsing {
        use super::*;

        #[test]
        fn test_two() {
            let acceptable = AcceptEncoding::from_str("deflate, gzip");
            let expect = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::default()),
                deflate: Some(QValue::default()),
                zstd: None,
                brotli: None,
                asterisk: None,
            };
            assert_eq!(Ok(expect), acceptable);
        }

        #[test]
        fn test_empty() {
            let acceptable = AcceptEncoding::from_str("");

            assert_eq!(Ok(AcceptEncoding::empty()), acceptable);
        }

        #[test]
        fn test_asterisk() {
            let acceptable = AcceptEncoding::from_str("*");

            let expect = AcceptEncoding {
                identity: None,
                gzip: None,
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::default()),
            };

            assert_eq!(Ok(expect), acceptable);
        }

        #[test]
        fn test_with_weight() {
            let acceptable = AcceptEncoding::from_str("deflate;q=0.5, gzip;q=1.0");

            let expect = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::max()),
                deflate: Some(QValue::try_from(0.5).unwrap()),
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            assert_eq!(Ok(expect), acceptable);
        }

        #[test]
        fn test_with_weight_2() {
            let acceptable = AcceptEncoding::from_str("gzip;q=1.0, identity; q=0.5, *;q=0");

            let expect = AcceptEncoding {
                identity: Some(QValue::try_from(0.5).unwrap()),
                gzip: Some(QValue::max()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::unacceptable()),
            };

            assert_eq!(Ok(expect), acceptable);
        }

        #[test]
        fn test_aggregate_unacceptable() {
            let acceptable = AcceptEncoding::from_str("gzip;q=0.1, gzip;q=0, gzip");

            let expect = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::unacceptable()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            assert_eq!(Ok(expect), acceptable);
        }

        #[test]
        fn test_aggregate() {
            let acceptable =
                AcceptEncoding::from_str("gzip;q=0.1, gzip;q=0.5, gzip;q=0.9, gzip;q=0.01");

            let expect = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::try_from(0.9).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            assert_eq!(Ok(expect), acceptable);
        }

        #[test]
        fn test_unknown() {
            let acceptable = AcceptEncoding::from_str("snappy");

            let expect = AcceptEncoding {
                identity: None,
                gzip: None,
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            assert_eq!(Ok(expect), acceptable);
        }
    }

    #[cfg(all(
        feature = "compress-gzip",
        feature = "compress-deflate",
        feature = "compress-zstd"
    ))]
    mod matching {
        use super::*;
        use crate::web::headers::Coding::{Deflate, Gzip, Identity, Zstandard};

        #[test]
        fn test_non_provided() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::try_from(0.1).unwrap()),
                deflate: Some(QValue::try_from(0.1).unwrap()),
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            let actual = accept.match_one_of(&[]);

            assert_eq!(Some(Identity.into()), actual);
        }

        #[test]
        fn test_non_accepted() {
            let accept = AcceptEncoding::empty();

            let actual = accept.match_one_of(&[Gzip, Deflate]);

            assert_eq!(Some(Identity.into()), actual);
        }

        #[test]
        fn test_only_one() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::try_from(0.1).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            let actual = accept.match_one_of(&[Gzip]);

            assert_eq!(Some(Gzip.into()), actual);
        }

        #[test]
        fn test_only_one_forbidden() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::unacceptable()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            let actual = accept.match_one_of(&[Gzip]);

            assert_eq!(Some(Identity.into()), actual);
        }

        #[test]
        fn test_all_forbidden() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: None,
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::unacceptable()),
            };

            let actual = accept.match_one_of(&[Gzip]);

            assert_eq!(None, actual);
        }

        #[test]
        fn test_unknown_forbidden() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::try_from(0.1).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::unacceptable()),
            };

            let actual = accept.match_one_of(&[Deflate]);

            assert_eq!(None, actual);
        }

        #[test]
        fn test_accept_only_known() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::try_from(0.1).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::unacceptable()),
            };

            let actual = accept.match_one_of(&[Gzip]);

            assert_eq!(Some(Gzip.into()), actual);
        }

        #[test]
        fn test_prefer_provided() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::default()),
                deflate: Some(QValue::default()),
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            let actual = accept.match_one_of(&[Gzip, Deflate]);

            assert_eq!(Some(Gzip.into()), actual);
        }

        #[test]
        fn test_consider_weight() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::default()),
                deflate: Some(QValue::try_from(0.5).unwrap()),
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::unacceptable()),
            };

            let actual = accept.match_one_of(&[Zstandard, Deflate, Gzip]);

            assert_eq!(Some(Gzip.into()), actual);
        }

        #[test]
        fn test_use_any() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: None,
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::default()),
            };

            let actual = accept.match_one_of(&[Gzip, Deflate]);

            assert_eq!(Some(Gzip.into()), actual);
        }

        #[test]
        fn test_use_any_but_not_this() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::unacceptable()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::default()),
            };

            let actual = accept.match_one_of(&[Gzip]);

            assert_eq!(Some(Identity.into()), actual);
        }

        #[test]
        fn test_identity_fallback_1() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::try_from(0.2).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            let actual = accept.match_one_of(&[Zstandard]);

            assert_eq!(Some(Identity.into()), actual);
        }

        #[test]
        fn test_identity_fallback_2() {
            let accept = AcceptEncoding {
                identity: Some(QValue::try_from(0.1).unwrap()),
                gzip: Some(QValue::try_from(0.2).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::unacceptable()),
            };

            let actual = accept.match_one_of(&[Zstandard]);

            assert_eq!(Some(Identity.into()), actual);
        }

        #[test]
        fn test_no_identity_fallback_1() {
            let accept = AcceptEncoding {
                identity: Some(QValue::unacceptable()),
                gzip: Some(QValue::try_from(0.2).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: None,
            };

            let actual = accept.match_one_of(&[Zstandard]);

            assert_eq!(None, actual);
        }

        #[test]
        fn test_no_identity_fallback_2() {
            let accept = AcceptEncoding {
                identity: None,
                gzip: Some(QValue::try_from(0.2).unwrap()),
                deflate: None,
                zstd: None,
                brotli: None,
                asterisk: Some(QValue::unacceptable()),
            };

            let actual = accept.match_one_of(&[Zstandard]);

            assert_eq!(None, actual);
        }
    }
}
