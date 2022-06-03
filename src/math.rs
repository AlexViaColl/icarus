use std::fmt;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

macro_rules! eq_f32 {
    ($a:expr, $b:expr) => {
        ($a - $b).abs() <= f32::EPSILON
    };
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Rect {
    pub offset: Vec2,
    pub extent: Vec2,
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

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Frame {
    pub o: Vec3, // Origin

    // Orthonormal base
    pub u: Vec3, // Pointing rightward
    pub v: Vec3, // Pointing upward
    pub w: Vec3, // Pointing backward (opposite to the view direction)
}
impl Frame {
    /// Produces a matrix that transforms points specified relative to the frame into points
    /// relative to the world, or canonical frame.
    #[rustfmt::skip]
    pub fn to_canonical(&self) -> Mat4 {
        Mat4::new([
            self.u.x, self.v.x, self.w.x, self.o.x,
            self.u.y, self.v.y, self.w.y, self.o.y,
            self.u.z, self.v.z, self.w.z, self.o.z,
                 0.0,      0.0,      0.0,      1.0,
        ])
    }

    /// Returns a matrix that transforms points specified in canonical coordinates into points
    /// relative to the frame.
    #[rustfmt::skip]
    pub fn from_canonical(&self) -> Mat4 {
        Mat4::new([
            self.u.x, self.u.y, self.u.z, 0.0,
            self.v.x, self.v.y, self.v.z, 0.0,
            self.w.x, self.w.y, self.w.z, 0.0,
                 0.0,      0.0,      0.0, 1.0,
        ]) *
        Mat4::translate((-self.o.x, -self.o.y, -self.o.z))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Sphere {
    pub c: Vec3, // Center of the sphere
    pub r: f32,  // Radius
}

// Mat4 stores its elements as row-major
#[repr(C)]
#[derive(PartialEq, Clone, Copy, Default)]
pub struct Mat4(pub [f32; 16]);

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

impl Rect {
    pub fn offset_extent<T1: Into<Vec2>, T2: Into<Vec2>>(offset: T1, extent: T2) -> Self {
        Self {
            offset: offset.into(),
            extent: extent.into(),
        }
    }

    pub fn center_extent<T1: Into<Vec2>, T2: Into<Vec2>>(center: T1, extent: T2) -> Self {
        let extent = extent.into();
        Self {
            offset: center.into() - extent * 0.5,
            extent,
        }
    }

    pub fn is_inside<T: Into<Vec2>>(&self, p: T) -> bool {
        let p = p.into();
        p.x >= self.offset.x
            && p.x <= self.offset.x + self.extent.x
            && p.y >= self.offset.y
            && p.y <= self.offset.y + self.extent.y
    }

    pub fn center(&self) -> Vec2 {
        self.offset + self.extent * 0.5
    }

    pub fn collides(&self, other: &Rect) -> bool {
        if self.is_inside(other.offset)
            || self.is_inside(other.offset + Vec2::new(0.0, other.extent.y))
            || self.is_inside(other.offset + Vec2::new(other.extent.x, 0.0))
            || self.is_inside(other.offset + other.extent)
        {
            return true;
        }

        false
    }
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn normalize(&self) -> Self {
        let length = (self.x * self.x + self.y * self.y).sqrt();
        Self::new(self.x / length, self.y / length)
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    pub fn len_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
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

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn len_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }

    pub fn normalize(&self) -> Self {
        let length = self.len();
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

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
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

    pub fn len_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }

    pub fn dot(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }

    pub fn as_f32(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
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

impl Add for Vec4 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
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

impl Mul<f32> for Vec4 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}
impl Mul<Vec4> for Vec4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Vec4::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z, self.w * rhs.z)
    }
}

impl Quaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            x,
            y,
            z,
            w,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Quaternion {
    fn from(item: (f32, f32, f32, f32)) -> Self {
        Self::new(item.0, item.1, item.2, item.3)
    }
}

impl From<[f32; 4]> for Quaternion {
    fn from(item: [f32; 4]) -> Self {
        Self::new(item[0], item[1], item[2], item[3])
    }
}

impl Mat4 {
    pub const fn new(e: [f32; 16]) -> Self {
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

    pub fn diag(&self) -> Vec4 {
        Vec4::new(self.0[0], self.0[5], self.0[10], self.0[15])
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

    pub fn is_linear(&self) -> bool {
        self.row(3) == Vec4::new(0.0, 0.0, 0.0, 1.0) && self.col(3) == Vec4::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn is_orthogonal(&self) -> bool {
        self.is_linear()
            && eq_f32!(self.row(0).dot(self.row(1)), 0.0)
            && eq_f32!(self.row(1).dot(self.row(2)), 0.0)
            && eq_f32!(self.row(0).len_sq(), 1.0)
            && eq_f32!(self.row(1).len_sq(), 1.0)
            && eq_f32!(self.row(2).len_sq(), 1.0)
    }
    pub fn is_rotation(&self) -> bool {
        self.is_orthogonal()
    }
    pub fn is_scale(&self) -> bool {
        let d = self.diag();
        self.is_linear()
            && d.x != 0.0
            && d.y != 0.0
            && d.z != 0.0
            && d.w == 1.0
            && self.0[1] == 0.0
            && self.0[2] == 0.0
            && self.0[4] == 0.0
            && self.0[6] == 0.0
            && self.0[8] == 0.0
            && self.0[9] == 0.0
    }
    pub fn is_translation(&self) -> bool {
        self.col(0) == Vec4::new(1.0, 0.0, 0.0, 0.0)
            && self.col(1) == Vec4::new(0.0, 1.0, 0.0, 0.0)
            && self.col(2) == Vec4::new(0.0, 0.0, 1.0, 0.0)
            && self.0[15] == 1.0
    }

    #[rustfmt::skip]
    pub fn inverse(&self) -> Self {
        match self {
            r if r.is_rotation() => r.transpose(),
            s if s.is_scale() => Mat4::scale(1.0 / s.0[0], 1.0 / s.0[5], 1.0 / s.0[10]),
            t if t.is_translation() => Mat4::translate((-t.0[3], -t.0[7], -t.0[11])),
            m => {
                let m = m.0;
                let adj00 = m[5] * m[10] * m[15] + m[6] * m[11] * m[13] + m[7] * m[9] * m[14]
                    - m[7] * m[10] * m[13]
                    - m[5] * m[11] * m[14]
                    - m[6] * m[9] * m[15];
                let adj01 = m[4] * m[10] * m[15] + m[6] * m[11] * m[12] + m[7] * m[8] * m[14]
                    - m[7] * m[10] * m[12]
                    - m[4] * m[11] * m[14]
                    - m[6] * m[8] * m[15];
                let adj02 = m[4] * m[9] * m[15] + m[5] * m[11] * m[12] + m[7] * m[8] * m[13]
                    - m[7] * m[9] * m[12]
                    - m[4] * m[11] * m[13]
                    - m[5] * m[8] * m[15];
                let adj03 = m[4] * m[9] * m[14] + m[5] * m[10] * m[12] + m[6] * m[8] * m[13]
                    - m[6] * m[9] * m[12]
                    - m[4] * m[10] * m[13]
                    - m[5] * m[8] * m[14];

                let adj10 = m[1] * m[10] * m[15] + m[2] * m[11] * m[13] + m[3] * m[9] * m[14]
                    - m[3] * m[10] * m[13]
                    - m[1] * m[11] * m[14]
                    - m[2] * m[9] * m[15];
                let adj11 = m[0] * m[10] * m[15] + m[2] * m[11] * m[12] + m[3] * m[8] * m[14]
                    - m[3] * m[10] * m[12]
                    - m[0] * m[11] * m[14]
                    - m[2] * m[8] * m[15];
                let adj12 = m[0] * m[9] * m[15] + m[1] * m[11] * m[12] + m[3] * m[8] * m[13]
                    - m[3] * m[9] * m[12]
                    - m[0] * m[11] * m[13]
                    - m[1] * m[8] * m[15];
                let adj13 = m[0] * m[9] * m[14] + m[1] * m[10] * m[12] + m[2] * m[8] * m[13]
                    - m[2] * m[9] * m[12]
                    - m[0] * m[10] * m[13]
                    - m[1] * m[8] * m[14];

                let adj20 = m[1] * m[6] * m[15] + m[2] * m[7] * m[13] + m[3] * m[5] * m[14]
                    - m[3] * m[6] * m[13]
                    - m[1] * m[7] * m[14]
                    - m[2] * m[5] * m[15];
                let adj21 = m[0] * m[6] * m[15] + m[2] * m[7] * m[12] + m[3] * m[4] * m[14]
                    - m[3] * m[6] * m[12]
                    - m[0] * m[7] * m[14]
                    - m[2] * m[4] * m[15];
                let adj22 = m[0] * m[5] * m[15] + m[1] * m[7] * m[12] + m[3] * m[4] * m[13]
                    - m[3] * m[5] * m[12]
                    - m[0] * m[7] * m[13]
                    - m[1] * m[4] * m[15];
                let adj23 = m[0] * m[5] * m[14] + m[1] * m[6] * m[12] + m[2] * m[4] * m[13]
                    - m[2] * m[5] * m[12]
                    - m[0] * m[6] * m[13]
                    - m[1] * m[4] * m[14];

                let adj30 = m[1] * m[6] * m[11] + m[2] * m[7] * m[9] + m[3] * m[5] * m[10]
                    - m[3] * m[6] * m[9]
                    - m[1] * m[7] * m[10]
                    - m[2] * m[5] * m[11];
                let adj31 = m[0] * m[6] * m[11] + m[2] * m[7] * m[8] + m[3] * m[4] * m[10]
                    - m[3] * m[6] * m[8]
                    - m[0] * m[7] * m[10]
                    - m[2] * m[4] * m[11];
                let adj32 = m[0] * m[5] * m[11] + m[1] * m[7] * m[8] + m[3] * m[4] * m[9]
                    - m[3] * m[5] * m[8]
                    - m[0] * m[7] * m[9]
                    - m[1] * m[4] * m[11];
                let adj33 = m[0] * m[5] * m[10] + m[1] * m[6] * m[8] + m[2] * m[4] * m[9]
                    - m[2] * m[5] * m[8]
                    - m[0] * m[6] * m[9]
                    - m[1] * m[4] * m[10];

                let det = m[0] * adj00 + m[1] * -adj01 + m[2] * adj02 + m[3] * -adj03;
                let inv_det = 1.0 / det;

                Mat4::new([
                     adj00, -adj10,  adj20, -adj30,
                    -adj01,  adj11, -adj21,  adj31,
                     adj02, -adj12,  adj22, -adj32,
                    -adj03,  adj13, -adj23,  adj33,
                ]) * inv_det
            }
        }
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
    pub fn rotate_x(angle_in_radians: f32) -> Self {
        let c = angle_in_radians.cos();
        let s = angle_in_radians.sin();
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0,   c,  -s, 0.0,
            0.0,   s,   c, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn rotate_y(angle_in_radians: f32) -> Self {
        let c = angle_in_radians.cos();
        let s = angle_in_radians.sin();
        Self([
              c, 0.0,   s, 0.0,
            0.0, 1.0, 0.0, 0.0,
             -s, 0.0,   c, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn rotate_z(angle_in_radians: f32) -> Self {
        let c = angle_in_radians.cos();
        let s = angle_in_radians.sin();
        Self([
              c,  -s, 0.0, 0.0,
              s,   c, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn rotate(angle_in_radians: f32, axis: (f32, f32, f32)) -> Self {
        // Taken from: https://www.opengl-tutorial.org/assets/faq_quaternions/index.html#Q38
        let c = angle_in_radians.cos();
        let s = angle_in_radians.sin();
        let axis = Vec3::new(axis.0, axis.1, axis.2);
        let Vec3{x: u, y: v, z: w} = axis.normalize();
        Self([
                 c + u * u * (1.0 - c), -w * s + u * v * (1.0 - c),  v * s + u * w * (1.0 - c), 0.0,
             w * s + v * u * (1.0 - c),      c + v * v * (1.0 - c), -u * s + v * w * (1.0 - c), 0.0,
            -v * s + w * u * (1.0 - c),  u * s + w * v * (1.0 - c),      c + w * w * (1.0 - c), 0.0,
                                   0.0,                        0.0,                        0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn look_at<T: Into<Vec3>>(eye: T, at: T, up: T) -> Self {
        let eye = eye.into(); // Position of the camera
        let w = -(at.into() - eye).normalize(); // Remember that w points in the opposite direction that the camera is looking into.
        let u = up.into().cross(w).normalize(); // Camera right vector.
        let v = w.cross(u).normalize(); // NOTE: In theory we shouldn't need to normalize here.

        let frame_cam = Frame {o: eye, u, v, w};
        let m_cam = frame_cam.from_canonical();
        m_cam
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
    // perspective RH
    pub fn perspective(fovy_radians: f32, aspect: f32, n: f32, f: f32) -> Self {
        assert!(n < f);

        let tan_half_fovy = (fovy_radians * 0.5).tan();

        // https://vincent-p.github.io/posts/vulkan_perspective_matrix/
        let result = Mat4::new([
            1.0 / (aspect * tan_half_fovy), 0.0,                  0.0,         0.0,
            0.0,                            -1.0 / tan_half_fovy, 0.0,         0.0,
            0.0,                            0.0,                  n / (f - n), n*f / (f - n),
            0.0,                            0.0,                  -1.0,        0.0,
        ]);

        // Vulkan Examples (GLM)
        let _result = Mat4::new([
            1.0 / (aspect * tan_half_fovy), 0.0,                 0.0,         0.0,
            0.0,                            1.0 / tan_half_fovy, 0.0,         0.0,
            0.0,                            0.0,                 f / (n - f), -(f * n) / (f - n),
            0.0,                            0.0,                 -1.0,        0.0,
        ]);

        result
    }

    pub fn rotate_acum<T: Into<Vec3>>(&self, angle_rad: f32, axis: T) -> Mat4 {
        let a = angle_rad;
        let c = a.cos();
        let s = a.sin();
        let axis = axis.into().normalize();
        let temp = axis * (1.0 - c);

        let mut rotate = Mat4::default();
        rotate.0[0] = c + temp.x * axis.x;
        rotate.0[4] = temp.x * axis.y + s * axis.z;
        rotate.0[8] = temp.x * axis.z - s * axis.y;

        rotate.0[1] = temp.y * axis.x - s * axis.z;
        rotate.0[5] = c + temp.y * axis.y;
        rotate.0[9] = temp.y * axis.z + s * axis.x;

        rotate.0[2] = temp.z * axis.x + s * axis.y;
        rotate.0[6] = temp.z * axis.y - s * axis.x;
        rotate.0[10] = c + temp.z * axis.z;

        let mut result = Mat4::default();
        // result[0] = self[0] * rotate[0][0] + self[1] * rotate[0][1] + self[2] * rotate[0][2];
        // result[1] = self[0] * rotate[1][0] + self[1] * rotate[1][1] + self[2] * rotate[1][2];
        // result[2] = self[0] * rotate[2][0] + self[1] * rotate[2][1] + self[2] * rotate[2][2];

        // First row
        let row = self.row(0) * rotate.0[0] + self.row(1) * rotate.0[4] + self.row(2) * rotate.0[8];
        result.0[0] = row.x;
        result.0[1] = row.y;
        result.0[2] = row.z;
        result.0[3] = row.w;

        // Second row
        let row = self.row(0) * rotate.0[1] + self.row(1) * rotate.0[5] + self.row(2) * rotate.0[9];
        result.0[4] = row.x;
        result.0[5] = row.y;
        result.0[6] = row.z;
        result.0[7] = row.w;

        // Third row
        let row = self.row(0) * rotate.0[2] + self.row(1) * rotate.0[6] + self.row(2) * rotate.0[10];
        result.0[8] = row.x;
        result.0[9] = row.y;
        result.0[10] = row.z;
        result.0[11] = row.w;

        // Last row
        result.0[12] = self.0[12];
        result.0[13] = self.0[13];
        result.0[14] = self.0[14];
        result.0[15] = self.0[15];
        result
    }
}

impl Add for Mat4 {
    type Output = Mat4;
    fn add(self, m: Mat4) -> Mat4 {
        Mat4::from_rows([
            self.row(0) + m.row(0),
            self.row(1) + m.row(1),
            self.row(2) + m.row(2),
            self.row(3) + m.row(3),
        ])
    }
}

impl Mul<f32> for Mat4 {
    type Output = Mat4;

    #[rustfmt::skip]
    fn mul(self, s: f32) -> Mat4 {
        Mat4::from_rows([self.row(0) * s, self.row(1) * s, self.row(2) * s, self.row(3) * s])
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
            assert_eq_f32!($a, $b, f32::EPSILON);
        };
        ($a:expr, $b:expr, $eps:expr) => {
            let diff = ($a - $b).abs();
            assert!(diff <= $eps, "diff: {}, EPSILON: {}", diff, $eps);
        };
    }

    macro_rules! assert_eq_v4 {
        ($a:expr, $b:expr) => {
            assert_eq_v4!($a, $b, f32::EPSILON);
        };
        ($a:expr, $b:expr, $eps:expr) => {
            assert_eq_f32!($a.x, $b.x, $eps);
            assert_eq_f32!($a.y, $b.y, $eps);
            assert_eq_f32!($a.z, $b.z, $eps);
            assert_eq_f32!($a.w, $b.w, $eps);
        };
    }

    macro_rules! assert_eq_mat4 {
        ($a:expr, $b:expr) => {
            assert_eq_mat4!($a, $b, f32::EPSILON);
        };
        ($a:expr, $b:expr, $eps:expr) => {
            for i in 0..4 {
                assert_eq_v4!($a.row(i), $b.row(i), $eps);
            }
        };
    }

    #[test]
    fn vec2_neg() {
        let v = Vec2::new(-1.0, 2.0);
        assert_eq!(-v, Vec2::new(1.0, -2.0));
    }

    #[test]
    #[rustfmt::skip]
    fn mat4_addition() {
        let m1 = Mat4::new([
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 1.0, 2.0, 3.0,
            4.0, 5.0, 6.0, 7.0,
        ]);
        let m2 = Mat4::new([
            0.0, 1.0, 1.0, 2.0,
            3.0, 5.0, 8.0, 9.0,
            9.0, 8.0, 5.0, 3.0,
            2.0, 1.0, 1.0, 0.0,
        ]);
        assert_eq!(m1 + m2, Mat4::new([
             1.0,  3.0,  4.0,  6.0,
             8.0, 11.0, 15.0, 17.0,
            18.0,  9.0,  7.0,  6.0,
             6.0,  6.0,  7.0,  7.0,
        ]));
    }

    #[test]
    fn mat4_times_scalar() {
        let m = Mat4::identity();
        let res = m * 4.0;
        assert_eq!(Mat4::new([4.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 4.0]), res);
    }

    #[test]
    #[rustfmt::skip]
    fn mat4_times_mat4() {
        let s = Mat4::scale(2.0, 3.0, 4.0);
        let t = Mat4::translate([1.0, 2.0, 3.0]);
        assert_eq!(t * s, Mat4::new([
            2.0, 0.0, 0.0, 1.0,
            0.0, 3.0, 0.0, 2.0,
            0.0, 0.0, 4.0, 3.0,
            0.0, 0.0, 0.0, 1.0,
        ]));
        assert_eq!(s * t, Mat4::new([
            2.0, 0.0, 0.0, 2.0,
            0.0, 3.0, 0.0, 6.0,
            0.0, 0.0, 4.0, 12.0,
            0.0, 0.0, 0.0, 1.0,
        ]));
    }

    #[test]
    #[rustfmt::skip]
    fn mat4_times_vec4() {
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
    #[rustfmt::skip]
    fn mat4_transpose() {
        let m = Mat4::new([
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 1.0, 2.0, 3.0,
            4.0, 5.0, 6.0, 7.0,
        ]);
        assert_eq!(m.transpose(), Mat4::new([
            1.0, 5.0, 9.0, 4.0,
            2.0, 6.0, 1.0, 5.0,
            3.0, 7.0, 2.0, 6.0,
            4.0, 8.0, 3.0, 7.0,
        ]));
    }

    #[test]
    fn mat4_translate() {
        let m = Mat4::translate((10.0, 0.0, 0.0));
        assert_eq!(m * Vec4::new(10.0, 10.0, 10.0, 1.0), Vec4::new(20.0, 10.0, 10.0, 1.0));
        assert_eq!(m * Vec4::new(10.0, 10.0, 10.0, 0.0), Vec4::new(10.0, 10.0, 10.0, 0.0));
    }

    #[test]
    fn mat4_scale() {
        let v = Vec4::new(1.0, 2.0, 3.0, 1.0);
        let m = Mat4::scale(2.0, 2.0, 2.0);
        assert_eq!(m * v, Vec4::new(2.0, 4.0, 6.0, 1.0));
    }

    #[test]
    fn mat4_rotate_z() {
        let v = Vec4::new(1.0, 0.0, 0.0, 0.0);
        let m = Mat4::rotate_z(45.0_f32.to_radians());
        assert_eq_v4!(m * v, Vec4::new(std::f32::consts::FRAC_1_SQRT_2, std::f32::consts::FRAC_1_SQRT_2, 0.0, 0.0));
    }

    #[test]
    fn mat4_rotate_x() {
        let v = Vec4::new(0.0, 0.0, 1.0, 0.0);
        let m = Mat4::rotate_x(90.0_f32.to_radians());
        assert_eq_v4!(m * v, Vec4::new(0.0, -1.0, 0.0, 0.0));
    }

    #[test]
    fn mat4_rotate_y() {
        let v = Vec4::new(1.0, 0.0, 0.0, 0.0);
        let m = Mat4::rotate_y(90.0_f32.to_radians());
        assert_eq_v4!(m * v, Vec4::new(0.0, 0.0, -1.0, 0.0));
    }

    #[test]
    fn mat4_rotate() {
        let angle = 23.2_f32.to_radians();
        assert_eq_mat4!(Mat4::rotate_x(angle), Mat4::rotate(angle, (1.0, 0.0, 0.0)));
        assert_eq_mat4!(Mat4::rotate_y(angle), Mat4::rotate(angle, (0.0, 1.0, 0.0)));
        assert_eq_mat4!(Mat4::rotate_z(angle), Mat4::rotate(angle, (0.0, 0.0, 1.0)));

        assert_eq_v4!(
            Mat4::rotate(90_f32.to_radians(), (1.0, 1.0, 0.0)) * Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.5, 0.5, -std::f32::consts::FRAC_1_SQRT_2, 0.0)
        );
    }

    #[test]
    fn mat4_windowing_transform() {
        // (-1, -1)  ---  (+1, -1)            (0, 0)    --- (800, 0)
        //          |   |              =>              |   |
        // (-1, +1)  ---  (+1, +1)            (0, 600)  --- (800, 600)

        let t1 = Mat4::translate((1.0, 1.0, 0.0)); // Translate point (-1,-1) to (0,0)
        let s = Mat4::scale(800.0 / 2.0, 600.0 / 2.0, 1.0);
        let t2 = Mat4::translate((0.0, 0.0, 0.0));

        let m = t2 * s * t1;
        assert_eq_v4!(m * Vec4::new(-1.0, -1.0, 0.0, 1.0), Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert_eq_v4!(m * Vec4::new(1.0, -1.0, 0.0, 1.0), Vec4::new(800.0, 0.0, 0.0, 1.0));
        assert_eq_v4!(m * Vec4::new(-1.0, 1.0, 0.0, 1.0), Vec4::new(0.0, 600.0, 0.0, 1.0));
        assert_eq_v4!(m * Vec4::new(1.0, 1.0, 0.0, 1.0), Vec4::new(800.0, 600.0, 0.0, 1.0));
        assert_eq_v4!(m * Vec4::new(0.0, 0.0, 0.0, 1.0), Vec4::new(400.0, 300.0, 0.0, 1.0));
    }

    #[test]
    fn mat4_is_linear() {
        assert!(Mat4::scale(1.0, 2.0, 3.0).is_linear());
        assert!(Mat4::rotate_x(30.0_f32.to_radians()).is_linear());
        assert!(!Mat4::translate((1.0, 2.0, 3.0)).is_linear());
        assert!(!Mat4::perspective(45.0_f32.to_radians(), 16.0 / 9.0, 0.1, 256.0).is_linear());
    }

    #[test]
    fn mat4_is_orthogonal() {
        assert!(!Mat4::scale(1.0, 2.0, 3.0).is_orthogonal());
        assert!(Mat4::rotate(31.223_f32.to_radians(), (1.12, 0.73, 0.29)).is_orthogonal());
        assert!(!Mat4::translate((1.0, 2.0, 3.0)).is_orthogonal());
        assert!(!Mat4::perspective(45.0_f32.to_radians(), 16.0 / 9.0, 0.1, 256.0).is_orthogonal());
    }

    #[test]
    fn mat4_is_scale() {
        assert!(Mat4::scale(1.0, 2.0, 3.0).is_scale());
        assert!(!Mat4::rotate(31.223_f32.to_radians(), (1.12, 0.73, 0.29)).is_scale());
        assert!(!Mat4::translate((1.0, 2.0, 3.0)).is_scale());
        assert!(!Mat4::perspective(45.0_f32.to_radians(), 16.0 / 9.0, 0.1, 256.0).is_scale());
    }

    #[test]
    fn mat4_is_translation() {
        assert!(!Mat4::scale(1.0, 2.0, 3.0).is_translation());
        assert!(!Mat4::rotate(31.223_f32.to_radians(), (1.12, 0.73, 0.29)).is_translation());
        assert!(Mat4::translate((1.0, 2.0, 3.0)).is_translation());
        assert!(Mat4::identity().is_translation());
        assert!(!Mat4::perspective(45.0_f32.to_radians(), 16.0 / 9.0, 0.1, 256.0).is_translation());
    }

    #[test]
    fn mat4_inverse() {
        let s = Mat4::scale(2.0, 1.0, 1.0);
        assert_eq_mat4!(s.inverse(), Mat4::scale(1.0 / 2.0, 1.0, 1.0));

        let angle = 33.33_f32.to_radians();
        let axis = (0.76, 0.13, 0.54);
        let r = Mat4::rotate(angle, axis);
        assert_eq_mat4!(r.inverse(), Mat4::rotate(-angle, axis));

        let t = Mat4::translate((1.0, 2.0, 3.0));
        assert_eq_mat4!(t.inverse(), Mat4::translate((-1.0, -2.0, -3.0)));

        assert_eq_mat4!((t * s * r).inverse(), r.inverse() * s.inverse() * t.inverse(), f32::EPSILON * 2.0);
    }

    #[test]
    fn frame_convert_frame_point_to_canonical() {
        let frame = Frame {
            o: Vec3::new(0.0, 0.0, 10.0),
            u: Vec3::new(1.0, 0.0, 0.0),
            v: Vec3::new(0.0, 1.0, 0.0),
            w: Vec3::new(0.0, 0.0, 1.0),
        };
        let p_frame = Vec4::new(1.0, 0.0, 0.0, 1.0);
        let p_canonical = frame.to_canonical() * p_frame;
        assert_eq_v4!(p_canonical, Vec4::new(1.0, 0.0, 10.0, 1.0));
    }

    #[test]
    fn frame_convert_canonical_point_to_frame() {
        let frame = Frame {
            o: Vec3::new(0.0, 0.0, 10.0),
            u: Vec3::new(1.0, 0.0, 0.0),
            v: Vec3::new(0.0, 1.0, 0.0),
            w: Vec3::new(0.0, 0.0, 1.0),
        };
        let p_canonical = Vec4::new(1.0, 0.0, 10.0, 1.0);
        let p_frame = frame.from_canonical() * p_canonical;
        assert_eq_v4!(p_frame, Vec4::new(1.0, 0.0, 0.0, 1.0));

        assert_eq_mat4!(frame.to_canonical().inverse(), frame.from_canonical());
        assert_eq_mat4!(frame.from_canonical().inverse(), frame.to_canonical());
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
        //todo!()
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
