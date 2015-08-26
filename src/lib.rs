#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]

extern crate num;

#[cfg(test)] extern crate quickcheck;


use num::{Float, Num, NumCast, Zero, cast};
use std::f64::consts::PI;
use std::fmt::{Display, Formatter, Error};
use std::ops::{Add, Div, Mul, Neg, Sub};


/// An angle.
///
/// Might be a value in degrees or in radians.
#[derive(Copy, Clone, Debug)]
pub enum Angle<T=f64> {
    Radians(T),
    Degrees(T)
}

impl<T: NumCast> Angle<T> {
    /// Yield the value encoded in radians.
    pub fn in_radians(self) -> T {
        match self {
            Radians(v) => v,
            Degrees(v) => cast(cast::<T, f64>(v).unwrap() / 180.0 * PI).unwrap()
        }
    }

    /// Yield the value encoded in degrees.
    pub fn in_degrees(self) -> T {
        match self {
            Radians(v) => cast(cast::<T, f64>(v).unwrap() / PI * 180.0).unwrap(),
            Degrees(v) => v
        }
    }
}

impl<T: Copy + Num + NumCast + PartialOrd> Angle<T> {
    /// Create a new angle by normalizing the value into the range of
    /// [0, 2π) rad.
    pub fn normalized(self) -> Angle<T> {
        let (v, upper) = match self {
            Radians(v) => (v, cast(2.0 * PI).unwrap()),
            Degrees(v) => (v, cast(360.0).unwrap())
        };

        let normalized = if v < upper && v >= Zero::zero() {
            v
        } else {
            let v = v % upper;

            if v >= Zero::zero() {
                v
            } else {
                v + upper
            }
        };

        match self {
            Radians(_) => Radians(normalized),
            Degrees(_) => Degrees(normalized)
        }
    }
}

impl<T: Float + NumCast> Angle<T> {
    /// Compute the sine of the angle.
    pub fn sin(self) -> T {
        self.in_radians().sin()
    }

    /// Compute the cosine of the angle.
    pub fn cos(self) -> T {
        self.in_radians().cos()
    }

    /// Compute the tangent of the angle.
    pub fn tan(self) -> T {
        self.in_radians().tan()
    }

    /// Simultaneously compute the sine and cosine of the number, `x`.
    /// Return `(sin(x), cos(x))`.
    pub fn sin_cos(self) -> (T, T) {
        self.in_radians().sin_cos()
    }

    /*
    /// Computes the approximate mean of a set of angles by averaging the
    /// Cartesian coordinates of the angles on the unit circle.
    pub fn mean(angles: &[Angle<T>]) -> Angle<T> {
        let mut x: T = Zero::zero();
        let mut y: T = Zero::zero();

        for angle in angles.iter() {
            let radian = angle.as_radian();
            x = x + radian.cos();
            y = y + radian.sin();
        }

        let n = cast(angles.len()).unwrap();
        let a = (y/n).atan2(x/n);

        Radian(a).normalize()
    }

    /// Computes the minimal unsigned distance between two normalized angles.
    pub fn distance(&self, other: Angle<T>) -> Angle<T> {
        // FIXME: Once associated types are implemented fix this nightmares of casts #17841
        let pi: T = cast(f64::consts::PI).unwrap();
        let pi2: T = pi * cast(2.0).unwrap();
        let d = (self.as_radian() - other.as_radian()).abs();
        Radian(pi - ((d % (pi2)) - pi).abs())
    }*/
}

impl<T: Float> Angle<T> {
    /// Compute the arcsine of a number. Return value is in the range of
    /// [-π/2, π/2] rad or `None` if the number is outside the range [-1, 1].
    pub fn asin(value: T) -> Option<Angle<T>> {
        let value = value.asin();
        if value.is_nan() {
            None
        } else {
            Some(Radians(value))
        }
    }

    /// Compute the arccosine of a number. Return value is in the range of
    /// [0, π] rad or `None` if the number is outside the range [-1, 1].
    pub fn acos(value: T) -> Option<Angle<T>> {
        let value = value.acos();
        if value.is_nan() {
            None
        } else {
            Some(Radians(value))
        }
    }

    /// Compute the arctangent of a number. Return value is in the range of
    /// [-π/2, π/2] rad.
    pub fn atan(value: T) -> Angle<T> {
        Radians(value.atan())
    }

    // Computes the four quadrant arctangent of `y` and `x`.
    pub fn atan2(y: T, x: T) -> Angle<T> {
        Radians(y.atan2(x))
    }
}

impl<T: Zero + Copy + NumCast> Zero for Angle<T> {
    fn zero() -> Self {
        Radians(T::zero())
    }

    fn is_zero(&self) -> bool {
        match self {
            &Radians(ref v) => v.is_zero(),
            &Degrees(ref v) => v.is_zero()
        }
    }
}

impl<T: PartialEq + Copy + NumCast> PartialEq for Angle<T> {
    fn eq(&self, other: &Angle<T>) -> bool {
        if let (Degrees(a), Degrees(b)) = (*self, *other) {
            a == b
        } else {
            self.in_radians() == other.in_radians()
        }
    }
}


macro_rules! math_additive(
    ($bound:ident, $func:ident) => (
        impl<T: $bound + Copy + NumCast> $bound for Angle<T> {
            type Output = Angle<T::Output>;
            fn $func(self, rhs: Angle<T>) -> Self::Output {
                if let (Degrees(a), Degrees(b)) = (self, rhs) {
                    Degrees(a.$func(b))
                } else {
                    Radians(self.in_radians().$func(rhs.in_radians()))
                }
            }
        }
    );
);

math_additive!(Add, add);
math_additive!(Sub, sub);

macro_rules! math_multiplicative(
    ($bound:ident, $func:ident, $($t:ident),*) => (
        impl<T: $bound> $bound<T> for Angle<T> {
            type Output = Angle<T::Output>;
            fn $func(self, rhs: T) -> Self::Output {
                match self {
                    Radians(v) => Radians(v.$func(rhs)),
                    Degrees(v) => Degrees(v.$func(rhs))
                }
            }
        }

        $(
            impl $bound<Angle<$t>> for $t {
                type Output = Angle<$t>;
                fn $func(self, rhs: Angle<$t>) -> Self::Output {
                    match rhs {
                        Radians(v) => Radians(self.$func(v)),
                        Degrees(v) => Degrees(self.$func(v))
                    }
                }
            }
        )*
    );
);

math_multiplicative!(Mul, mul, u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64);
math_multiplicative!(Div, div, u8, u16, u32, u64, i8, i16, i32, i64, usize, isize, f32, f64);

impl<T: Neg> Neg for Angle<T> {
    type Output = Angle<T::Output>;
    fn neg(self) -> Self::Output {
        match self {
            Radians(v) => Radians(-v),
            Degrees(v) => Degrees(-v)
        }
    }
}

impl<T: Display> Display for Angle<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Radians(ref v) => write!(f, "{}rad", v),
            Degrees(ref v) => write!(f, "{}°", v)
        }
    }
}

unsafe impl<T: Send> Send for Angle<T> {  }

/*
impl<T: Encodable + Float + NumCast> Encodable for Angle<T> {
    fn encode<S: Encoder>(&self,  s: &mut S) -> Result<(), S::Error> {
        self.as_radian().encode(s)
    }
}

impl<T: Decodable + Float + NumCast> Decodable for Angle<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let t = try!(T::decode(d));
        Ok(Angle::Radian(t))
    }
}
*/

// re-exports
pub use Angle::{Radians, Degrees};


#[cfg(test)]
mod tests {
    use num::{Float, cast};
    use quickcheck::{Arbitrary, Gen};
    use std::f64::consts::PI;
    use super::{Angle, Radians, Degrees};

    #[quickcheck]
    fn test_angle_conversions(angle: Angle<f64>) -> bool {
        are_close(angle.in_radians(), Degrees(angle.in_degrees()).in_radians())
    }

    #[quickcheck]
    fn test_angle_math_multiplicative(a: Angle<f64>, x: f64) -> bool {
        match a {
            Radians(v) => (a * x).in_radians() == v * x &&
                          (a / x).in_radians() == v / x,
            Degrees(v) => (a * x).in_degrees() == v * x &&
                          (a / x).in_degrees() == v / x
        }
    }

    #[quickcheck]
    fn test_angle_math_additive(a: Angle, b: Angle) -> bool {
        if let (Radians(x), Radians(y)) = (a, b) {
            (a + b).in_radians() == x + y &&
            (a - b).in_radians() == x - y
        } else if let (Degrees(x), Degrees(y)) = (a, b) {
            (a + b).in_degrees() == x + y &&
            (a - b).in_degrees() == x - y
        } else {
            (a + b).in_radians() == a.in_radians() + b.in_radians()
        }
    }

    #[quickcheck]
    fn test_angle_normalization(angle: Angle) -> bool {
        let v = angle.normalized();
        let rad = v.in_radians();
        let deg = v.in_degrees();

        0.0 <= rad && rad < 2.0 * PI &&
        0.0 <= deg && deg < 360.0 &&
        are_close(rad.cos(), angle.cos())
    }

    fn are_close<T: Float>(a: T, b: T) -> bool {
        (a - b).abs() < cast(1.0e-10).unwrap()
    }

    impl<T: Arbitrary> Arbitrary for Angle<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let v = Arbitrary::arbitrary(g);
            if bool::arbitrary(g) {
                Radians(v)
            } else {
                Degrees(v)
            }
        }
    }
}
