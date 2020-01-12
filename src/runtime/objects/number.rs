use std::cmp::Ordering;
use std::{fmt, ops, u64};

#[derive(Debug, Clone)]
pub struct KodyNumber {
    numerator: u128,
    denominator: u128,
    is_negative: bool,
}

impl KodyNumber {
    pub fn from_int(x: i64) -> KodyNumber {
        KodyNumber {
            numerator: x.abs() as u128,
            denominator: 1,
            is_negative: x < 0,
        }
    }

    pub fn from_float(x: f64) -> KodyNumber {
        if x == 0.0 {
            return KodyNumber {
                numerator: 0,
                denominator: 1,
                is_negative: false,
            };
        }
        let is_negative = x < 0.0;
        let mut num = x.abs();
        let mut denominator = 1_u128;
        while 2.0*num < u64::MAX as f64 && 2*denominator < u64::MAX as u128 {
            num *= 2.0;
            denominator *= 2;
        }
        KodyNumber {
            numerator: num as u128,
            denominator,
            is_negative,
        }
    }

    fn simplify(&mut self) {
        let gcd = gcd(self.numerator, self.denominator);
        self.numerator /= gcd;
        self.denominator /= gcd;

        while self.numerator > u64::MAX as u128 || self.denominator > u64::MAX as u128 {
            self.numerator /= 2;
            self.denominator /= 2;
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Add for &KodyNumber {
    type Output = KodyNumber;
    fn add(self, other: &KodyNumber) -> KodyNumber {
        let lcm = lcm(self.denominator, other.denominator);
        let lhs = self.numerator * (lcm / self.denominator);
        let rhs = other.numerator * (lcm / other.denominator);

        let (numerator, is_negative) = match (self.is_negative, other.is_negative) {
            (false, false) => (lhs + rhs, false),
            (true, false) => (u128::max(lhs, rhs) - u128::min(lhs, rhs), lhs > rhs),
            (false, true) => (u128::max(lhs, rhs) - u128::min(lhs, rhs), rhs > lhs),
            (true, true) => (lhs + rhs, true),
        };

        let mut result = KodyNumber {
            numerator,
            denominator: lcm,
            is_negative,
        };
        result.simplify();

        result
    }
}

impl ops::Sub for &KodyNumber {
    type Output = KodyNumber;
    fn sub(self, other: &KodyNumber) -> KodyNumber {
        self + &(-other)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Mul for &KodyNumber {
    type Output = KodyNumber;
    fn mul(self, other: &KodyNumber) -> KodyNumber {
        let mut result = KodyNumber {
            numerator: self.numerator * other.numerator,
            denominator: self.denominator * other.denominator,
            is_negative: self.is_negative ^ other.is_negative,
        };
        result.simplify();

        result
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Div for &KodyNumber {
    type Output = KodyNumber;
    fn div(self, other: &KodyNumber) -> KodyNumber {
        let mut result = KodyNumber {
            numerator: self.numerator * other.denominator,
            denominator: self.denominator * other.numerator,
            is_negative: self.is_negative ^ other.is_negative,
        };
        result.simplify();

        result
    }
}

impl ops::Neg for &KodyNumber {
    type Output = KodyNumber;
    fn neg(self) -> KodyNumber {
        KodyNumber {
            denominator: self.denominator,
            numerator: self.numerator,
            is_negative: !self.is_negative,
        }
    }
}

impl fmt::Display for KodyNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}", self.numerator as f64 / self.denominator as f64)
        }
    }
}

impl Ord for KodyNumber {
    fn cmp(&self, other: &KodyNumber) -> Ordering {
        let lcm = lcm(self.denominator, other.denominator);
        let lhs = self.numerator * (lcm / self.denominator);
        let rhs = other.numerator * (lcm / other.denominator);

        match (self.is_negative, other.is_negative) {
            (false, false) => lhs.cmp(&rhs),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (true, true) => rhs.cmp(&lhs),
        }
    }
}

impl PartialOrd for KodyNumber {
    fn partial_cmp(&self, other: &KodyNumber) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for KodyNumber {}

impl PartialEq for KodyNumber {
    fn eq(&self, other: &KodyNumber) -> bool {
        self.denominator == other.denominator && self.numerator == other.numerator
    }
}

fn gcd(x: u128, y: u128) -> u128 {
    let mut dividend = u128::max(x, y);
    let mut divisor = u128::min(x, y);
    while divisor > 0 {
        let remainder = dividend % divisor;
        dividend = divisor;
        divisor = remainder;
    }
    dividend
}

fn lcm(x: u128, y: u128) -> u128 {
    (x * y) / gcd(x, y)
}
