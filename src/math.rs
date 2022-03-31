use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

/// Linear interpolation between A and B by a "normalized percentage" T.
/// Panics if T is not in the range from 0 to 1.
pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Mul<f32, Output = T> + Add<Output = T>,
{
    assert!((0.0..=1.0).contains(&t));
    a * (1.0 - t) + b * t
}

//pub fn inv_lerp<T>(a: T, b: T, value: T) -> f32
//where
//    T: Sub<Output = T>,
pub fn inv_lerp(a: f32, b: f32, value: f32) -> f32 {
    (value - a) / (b - a)
}

#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// Mat4 stores its elements as row-major
#[repr(C)]
#[derive(PartialEq, Clone, Copy, Default)]
pub struct Mat4([f32; 16]);

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(item: (f32, f32)) -> Self {
        Self::new(item.0, item.1)
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(item: [f32; 2]) -> Self {
        Self::new(item[0], item[1])
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn normalize(&self) -> Self {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Self::new(self.x / length, self.y / length, self.z / length)
    }

    pub fn dot(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: Self) -> Self {
        Self::new(self.y * rhs.z - self.z * rhs.y, self.z * rhs.x - self.x * rhs.z, self.x * rhs.y - self.y * rhs.x)
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(item: (f32, f32, f32)) -> Self {
        Self::new(item.0, item.1, item.2)
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(item: [f32; 3]) -> Self {
        Self::new(item[0], item[1], item[2])
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            x,
            y,
            z,
            w,
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
            w: self.w.abs(),
        }
    }

    pub fn dot(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
}

impl From<(f32, f32, f32)> for Vec4 {
    fn from(item: (f32, f32, f32)) -> Self {
        Self::new(item.0, item.1, item.2, 1.0)
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(item: (f32, f32, f32, f32)) -> Self {
        Self::new(item.0, item.1, item.2, item.3)
    }
}

impl From<[f32; 4]> for Vec4 {
    fn from(item: [f32; 4]) -> Self {
        Self::new(item[0], item[1], item[2], item[3])
    }
}

impl Sub for Vec4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl Mat4 {
    pub fn new(e: [f32; 16]) -> Self {
        Self(e)
    }

    #[rustfmt::skip]
    pub fn from_rows(rows: [Vec4; 4]) -> Self {
        Self([
            rows[0].x, rows[0].y, rows[0].z, rows[0].w,
            rows[1].x, rows[1].y, rows[1].z, rows[1].w,
            rows[2].x, rows[2].y, rows[2].z, rows[2].w,
            rows[3].x, rows[3].y, rows[3].z, rows[3].w,
        ])
    }

    #[rustfmt::skip]
    pub fn from_cols(cols: [Vec4; 4]) -> Self {
        Self([
            cols[0].x, cols[1].x, cols[2].x, cols[3].x,
            cols[0].y, cols[1].y, cols[2].y, cols[3].y,
            cols[0].z, cols[1].z, cols[2].z, cols[3].z,
            cols[0].w, cols[1].w, cols[2].w, cols[3].w,
        ])
    }

    pub fn zero() -> Self {
        Self([0.0; 16])
    }

    #[rustfmt::skip]
    pub fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn row(&self, idx: usize) -> Vec4 {
        Vec4::new(self.0[idx * 4], self.0[idx * 4 + 1], self.0[idx * 4 + 2], self.0[idx * 4 + 3])
    }

    pub fn col(&self, idx: usize) -> Vec4 {
        Vec4::new(self.0[idx], self.0[4 + idx], self.0[2 * 4 + idx], self.0[3 * 4 + idx])
    }

    pub fn rows(&self) -> [Vec4; 4] {
        [self.row(0), self.row(1), self.row(2), self.row(3)]
    }

    pub fn cols(&self) -> [Vec4; 4] {
        [self.col(0), self.col(1), self.col(2), self.col(3)]
    }

    pub fn transpose(&self) -> Self {
        Self::from_rows(self.cols())
    }

    #[rustfmt::skip]
    pub fn translate<T: Into<Vec3>>(t: T) -> Self {
        let Vec3 {x, y, z} = t.into();
        Self([
            1.0, 0.0, 0.0,   x,
            0.0, 1.0, 0.0,   y,
            0.0, 0.0, 1.0,   z,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self([
            x,   0.0, 0.0, 0.0,
            0.0,   y, 0.0, 0.0,
            0.0, 0.0,   z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn rotate(angle_in_radians: f32, axis: (f32, f32, f32)) -> Self {
        // Taken from: https://www.opengl-tutorial.org/assets/faq_quaternions/index.html#Q38
        let rcos = angle_in_radians.cos();
        let rsin = angle_in_radians.sin();
        let (u, v, w) = axis;
        Self([
                 rcos + u * u * (1.0 - rcos), -w * rsin + u * v * (1.0 - rcos),  v * rsin + u * w * (1.0 - rcos), 0.0,
             w * rsin + v * u * (1.0 - rcos),      rcos + v * v * (1.0 - rcos), -u * rsin + v * w * (1.0 - rcos), 0.0,
            -v * rsin + w * u * (1.0 - rcos),  u * rsin + w * v * (1.0 - rcos),      rcos + w * w * (1.0 - rcos), 0.0,
                                         0.0,                               0.0,                             0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    #[allow(unused_variables)]
    pub fn look_at<T: Into<Vec3>>(eye: T, at: T, up: T) -> Self {
        let eye = eye.into();
        let forward = (at.into() - eye).normalize();
        let up = up.into().normalize();
        let right = forward.cross(up).normalize();
        let up = forward.cross(right).normalize();

        //let r = Mat4::rotate();
        let t = Mat4::translate(-forward);

        // TODO
        Self([
              right.x,   right.y,   right.z, 0.0,
                 up.x,      up.y,      up.z, 0.0,
            forward.x, forward.y, forward.z, 0.0,
                eye.x,     eye.y,     eye.z, 1.0,
        ])
    }

    /// Produces a Matrix to convert the given a rectangular cuboid, into a cube from -1 to 1 in
    /// all 3 axis
    /// X: Left   < Right  <=>  -1, +1
    /// Y: Bottom < Top    <=>  -1, +1
    /// Z: Near   < Far    <=>  -1, +1
    #[rustfmt::skip]
    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self([
            2.0 / (right - left),                  0.0,                0.0, -(right + left) / (right - left),
                             0.0, 2.0 / (top - bottom),                0.0, -(top + bottom) / (top - bottom),
                             0.0,                  0.0, 2.0 / (far - near), -(far + near)   / (far -   near),
                             0.0,                  0.0,                0.0,                              1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let focal_length = 1.0 / (fovy_radians / 2.0).tan();
        Self([
            focal_length / aspect,           0.0,                       0.0,                       0.0,
                              0.0, -focal_length,                       0.0,                       0.0,
                              0.0,           0.0,       near / (far - near),     near*far/(far - near),
                              0.0,           0.0,                      -1.0,                       0.0,
        ])
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[rustfmt::skip]
    fn mul(self, rhs: Vec4) -> Vec4 {
        let mut v = [0.0; 4];
        for (row, item) in v.iter_mut().enumerate() {
            *item = self.0[row * 4]     * rhs.x +
                    self.0[row * 4 + 1] * rhs.y +
                    self.0[row * 4 + 2] * rhs.z +
                    self.0[row * 4 + 3] * rhs.w;
        }
        Vec4::new(v[0], v[1], v[2], v[3])
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Mat4 {
        let mut cols = [Vec4::zero(); 4];
        for (idx, item) in cols.iter_mut().enumerate() {
            *item = self * rhs.col(idx);
        }

        Mat4::from_cols(cols)
    }
}

impl fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mat4 {{")?;
        writeln!(f, "    {:>.6}, {:>.6}, {:>.6}, {:>.6},", self.0[0], self.0[1], self.0[2], self.0[3])?;
        writeln!(f, "    {:>.6}, {:>.6}, {:>.6}, {:>.6},", self.0[4], self.0[5], self.0[6], self.0[7])?;
        writeln!(f, "    {:>.6}, {:>.6}, {:>.6}, {:>.6},", self.0[8], self.0[9], self.0[10], self.0[11])?;
        writeln!(f, "    {:>.6}, {:>.6}, {:>.6}, {:>.6},", self.0[12], self.0[13], self.0[14], self.0[15])?;
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_eq_f32 {
        ($a:expr, $b:expr) => {
            let diff = ($a - $b).abs();
            assert!(diff <= f32::EPSILON, "diff: {}, EPSILON: {}", diff, f32::EPSILON);
        };
    }

    macro_rules! assert_eq_v4 {
        ($a:expr, $b:expr) => {
            assert_eq_f32!($a.x, $b.x);
            assert_eq_f32!($a.y, $b.y);
            assert_eq_f32!($a.z, $b.z);
            assert_eq_f32!($a.w, $b.w);
        };
    }

    #[test]
    #[rustfmt::skip]
    fn multiplication() {
        let m = Mat4::identity();
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m * v, v);

        let m = Mat4::new([
             1.0,  2.0,  3.0,  4.0,
             5.0,  6.0,  7.0,  8.0,
             9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        ]);
        assert_eq!(m * v, Vec4::new(30.0, 70.0, 110.0, 150.0));
    }

    #[test]
    fn translation() {
        let v = Vec4::new(10.0, 10.0, 10.0, 1.0);
        let m = Mat4::translate((10.0, 0.0, 0.0));
        assert_eq!(m * v, Vec4::new(20.0, 10.0, 10.0, 1.0));
    }

    #[test]
    fn scale() {
        let v = Vec4::new(1.0, 2.0, 3.0, 1.0);
        let m = Mat4::scale(2.0, 2.0, 2.0);
        assert_eq!(m * v, Vec4::new(2.0, 4.0, 6.0, 1.0));
    }

    #[test]
    fn rotate() {
        let v = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let m = Mat4::rotate(std::f32::consts::FRAC_PI_2, (0.0, 0.0, 1.0));
        let actual = m * v;
        let expected = Vec4::new(0.0, 1.0, 0.0, 1.0);
        let epsilon: Vec4 = Vec4::new(f32::EPSILON, f32::EPSILON, f32::EPSILON, f32::EPSILON);
        assert!((actual - expected).abs() < epsilon);

        let v = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let m = Mat4::rotate(std::f32::consts::PI, (0.0, 0.0, 1.0));
        let actual = m * v;
        let expected = Vec4::new(-1.0, 0.0, 0.0, 1.0);
        assert!((actual - expected).abs() < epsilon);
    }

    #[test]
    fn ortho() {
        // X
        let left = 3.0;
        let right = 23.0;

        // Y
        let bottom = -2.0;
        let top = 10.0;

        // Z
        let near = 1.0;
        let far = 100.0;

        let m = Mat4::ortho(left, right, bottom, top, near, far);

        let left_bottom_near = m * Vec4::new(left, bottom, near, 1.0);
        let right_bottom_near = m * Vec4::new(right, bottom, near, 1.0);
        let left_top_near = m * Vec4::new(left, top, near, 1.0);
        let right_top_near = m * Vec4::new(right, top, near, 1.0);

        let left_bottom_far = m * Vec4::new(left, bottom, far, 1.0);
        let right_bottom_far = m * Vec4::new(right, bottom, far, 1.0);
        let left_top_far = m * Vec4::new(left, top, far, 1.0);
        let right_top_far = m * Vec4::new(right, top, far, 1.0);

        assert_eq_v4!(left_bottom_near, Vec4::new(-1.0, -1.0, -1.0, 1.0));
        assert_eq_v4!(right_bottom_near, Vec4::new(1.0, -1.0, -1.0, 1.0));
        assert_eq_v4!(left_top_near, Vec4::new(-1.0, 1.0, -1.0, 1.0));
        assert_eq_v4!(right_top_near, Vec4::new(1.0, 1.0, -1.0, 1.0));

        assert_eq_v4!(left_bottom_far, Vec4::new(-1.0, -1.0, 1.0, 1.0));
        assert_eq_v4!(right_bottom_far, Vec4::new(1.0, -1.0, 1.0, 1.0));
        assert_eq_v4!(left_top_far, Vec4::new(-1.0, 1.0, 1.0, 1.0));
        assert_eq_v4!(right_top_far, Vec4::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn perspective() {
        todo!()
    }

    #[test]
    fn ortho_translate_scale() {
        // X
        let left = 3.0;
        let right = 23.0;

        // Y
        let bottom = -2.0;
        let top = 10.0;

        // Z
        let near = 1.0;
        let far = 100.0;

        let m = Mat4::scale(1.0, 1.0, 0.5)
            * Mat4::translate((0.0, 0.0, 1.0))
            * Mat4::ortho(left, right, bottom, top, near, far);

        let left_bottom_near = m * Vec4::new(left, bottom, near, 1.0);
        let right_bottom_near = m * Vec4::new(right, bottom, near, 1.0);
        let left_top_near = m * Vec4::new(left, top, near, 1.0);
        let right_top_near = m * Vec4::new(right, top, near, 1.0);

        let left_bottom_far = m * Vec4::new(left, bottom, far, 1.0);
        let right_bottom_far = m * Vec4::new(right, bottom, far, 1.0);
        let left_top_far = m * Vec4::new(left, top, far, 1.0);
        let right_top_far = m * Vec4::new(right, top, far, 1.0);

        assert_eq_v4!(left_bottom_near, Vec4::new(-1.0, -1.0, 0.0, 1.0));
        assert_eq_v4!(right_bottom_near, Vec4::new(1.0, -1.0, 0.0, 1.0));
        assert_eq_v4!(left_top_near, Vec4::new(-1.0, 1.0, 0.0, 1.0));
        assert_eq_v4!(right_top_near, Vec4::new(1.0, 1.0, 0.0, 1.0));

        assert_eq_v4!(left_bottom_far, Vec4::new(-1.0, -1.0, 1.0, 1.0));
        assert_eq_v4!(right_bottom_far, Vec4::new(1.0, -1.0, 1.0, 1.0));
        assert_eq_v4!(left_top_far, Vec4::new(-1.0, 1.0, 1.0, 1.0));
        assert_eq_v4!(right_top_far, Vec4::new(1.0, 1.0, 1.0, 1.0));
    }
}
