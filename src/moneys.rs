//! Representation of money

use anyhow::{bail, Result};
use std::convert::TryFrom;

/// `Moneys` is type that represents given floating money amount as a integer multiple of
/// tenthousandth of given currency unit.
///
/// The value stored in `Moneys` has always be less-than-or-euqal to `MAX_EXACT_UNITS`.
///
/// Moneys has finite precision so it has to be able to throw error on overflow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Moneys(u64);

impl Moneys {
    /// TODO: Get rid of all f64 arithmetic
    ///
    /// As for now in many arithmetic operation we use f64 as an intermediate
    /// representation and we want to be sure that we haven't lost any
    /// precision while doing arithmetic.
    ///
    /// f64 can represent exactly integers up to `2**53` (52-bit mantissa +
    /// leading 1 bit that's not included in the mantissa).
    ///
    /// We also want to be able to do addition on the intermediate type and
    /// not lose precision on the result.
    ///
    /// log(2**53 / 2)/log(10) = 15.65355977452702
    ///
    /// So that's about 15 decimal degits. Let's make it 14 to be on the safe
    /// side.
    /// This value is checked in test later.
    const MAX_EXACT_UNITS: u64 = 100_000_000_000_000;

    /// How much is one moneys worth in f64
    const ONE_UNIT_AS_AMOUNT: f64 = 0.0001;

    /// Maximum Moneys value
    pub const MAX: Self = Self(Self::MAX_EXACT_UNITS);

    /// Constant for convenience
    pub const ZERO: Self = Self(0);

    pub const fn new(units: u64) -> Self {
        Self(units)
    }

    pub fn add(&self, other: Self) -> Result<Self> {
        let sum = self.0 + other.0;
        if sum > Self::MAX_EXACT_UNITS {
            bail!("addition overflow");
        }
        Ok(Self(sum))
    }

    pub fn sub(&self, other: Self) -> Result<Self> {
        if self.0 < other.0 {
            bail!("subtraction underflow");
        }
        Ok(Self(self.0 - other.0))
    }

    pub fn less_than(&self, other: Self) -> bool {
        self.0 < other.0
    }
}

impl TryFrom<f64> for Moneys {
    /// TODO: add special error for this error
    type Error = anyhow::Error;

    fn try_from(num: f64) -> Result<Self, Self::Error> {
        let units = (num / Self::ONE_UNIT_AS_AMOUNT).round();

        if units < 0.0 {
            bail!("negative money value");
        }
        if units > (Self::MAX_EXACT_UNITS as f64) {
            bail!("money value too big");
        }
        Ok(Moneys::new(units as u64))
    }
}

/// TODO: Implement integer parser for `Moneys` and get rid of all f64 arithmetic.
///
/// TODO: By parsing values as f64, we implicitely interpret values past the fourth decimal place,
/// because we keep rounding values (ie. 0.00005 should be interpreted as 0.0000, but we keep
/// interpreting it as 0.0001 because of rounding; at the same time we cannot just floor the
/// values, because that would make (inexact) 0.0002 value which is represented as 0.0001999999...
/// into 0.0001).
impl TryFrom<&str> for Moneys {
    /// TODO: add special error for this error
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let num: f64 = s.parse()?;
        Moneys::try_from(num)
    }
}

impl From<Moneys> for f64 {
    fn from(m: Moneys) -> Self {
        (m.0 as f64) * Moneys::ONE_UNIT_AS_AMOUNT
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::primitive::f64;

    #[test]
    fn limits() {
        assert!(
            (Moneys::MAX_EXACT_UNITS as f64) < f64::powf(2.0, f64::from(f64::MANTISSA_DIGITS - 1)),
            "BUG: MAX_EXACT doesn't fit twice precisely into f64"
        );
    }

    #[test]
    fn conversion() {
        assert_eq!(Moneys::try_from("-0.0").unwrap(), Moneys(0));
        assert_eq!(Moneys::try_from("0.00004").unwrap(), Moneys(0));
        assert_eq!(Moneys::try_from("0.00005").unwrap(), Moneys(1));
        assert_eq!(Moneys::try_from("1").unwrap(), Moneys(10_000));
        assert_eq!(Moneys::try_from("10000000000.0000").unwrap(), Moneys::MAX);
        assert!(Moneys::try_from("10000000000.0001").is_err());
    }

    /// Test that we can do aritmetic just below the limit and that it is exact
    #[test]
    fn exact() {
        let a = Moneys::try_from("9999999999.9999").unwrap();
        let b = Moneys::try_from("0.0001").unwrap();
        let c = a.add(b).unwrap();
        assert_eq!(a, Moneys(99999999999999));
        assert_eq!(b, Moneys(1));
        assert_eq!(c, Moneys::MAX);
        assert_eq!(f64::from(c), 10000000000.0000);
    }

    #[test]
    fn add() {
        assert_eq!(Moneys(3).add(Moneys(5)).unwrap(), Moneys(8));
        assert_eq!(Moneys(10).add(Moneys(0)).unwrap(), Moneys(10));
        assert!(Moneys::MAX.add(Moneys(1)).is_err());
        assert_eq!(
            Moneys(Moneys::MAX_EXACT_UNITS - 1).add(Moneys(1)).unwrap(),
            Moneys::MAX
        );
    }

    #[test]
    fn sub() {
        assert_eq!(Moneys(8).sub(Moneys(5)).unwrap(), Moneys(3));
        assert_eq!(Moneys(10).sub(Moneys(0)).unwrap(), Moneys(10));
        assert_eq!(Moneys(0).sub(Moneys(0)).unwrap(), Moneys(0));
        assert!(Moneys(3).sub(Moneys(4)).is_err());
        assert_eq!(Moneys::MAX.sub(Moneys::MAX).unwrap(), Moneys(0));
        assert_eq!(
            Moneys::MAX
                .sub(Moneys(Moneys::MAX_EXACT_UNITS - 1))
                .unwrap(),
            Moneys(1)
        );
    }

    #[test]
    fn less_than() {
        assert!(Moneys(8).less_than(Moneys(10)));
        assert!(Moneys(8).less_than(Moneys::MAX));
        assert!(!Moneys(10).less_than(Moneys(0)));
    }
}
