use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Represent a 3D point.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point3D {
    /// The x coordinate
    pub x: i32,
    /// The y coordinate
    pub y: i32,
    /// The z coordinate
    pub z: i32,
}

impl Point3D {
    /// Create a new [3D point](Point3D).
    ///
    /// # Arguments
    ///
    /// * `x` - The coordinate of the point in the x axis.
    /// * `y` - The coordinate of the point in the y axis.
    /// * `z` - The coordinate of the point in the z axis.
    ///
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl Add for Point3D {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Point3D {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul for Point3D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Div for Point3D {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Add<i32> for Point3D {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl Sub<i32> for Point3D {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl Mul<i32> for Point3D {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<i32> for Point3D {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl AddAssign for Point3D {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Point3D {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign for Point3D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl DivAssign for Point3D {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl AddAssign<i32> for Point3D {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs;
    }
}

impl SubAssign<i32> for Point3D {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs;
    }
}

impl MulAssign<i32> for Point3D {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs;
    }
}

impl DivAssign<i32> for Point3D {
    fn div_assign(&mut self, rhs: i32) {
        *self = *self / rhs;
    }
}

impl From<(i32, i32, i32)> for Point3D {
    fn from(value: (i32, i32, i32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

impl From<i32> for Point3D {
    fn from(value: i32) -> Self {
        Self::new(value, value, value)
    }
}

impl PartialEq<i32> for Point3D {
    fn eq(&self, other: &i32) -> bool {
        self.x == *other && self.y == *other && self.z == *other
    }
}
