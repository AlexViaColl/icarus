use std::ops::{Mul, Sub};

#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// Mat4 stores its elements as row-major
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Mat4([f32; 16]);

impl Vec3 {
    #[rustfmt::skip]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Vec4 {
    #[rustfmt::skip]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
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
}

impl Sub for Vec4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
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
        Vec4::new(self.0[idx * 4 + 0], self.0[idx * 4 + 1], self.0[idx * 4 + 2], self.0[idx * 4 + 3])
    }

    pub fn col(&self, idx: usize) -> Vec4 {
        Vec4::new(self.0[0 * 4 + idx], self.0[1 * 4 + idx], self.0[2 * 4 + idx], self.0[3 * 4 + idx])
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
    pub fn translate(x: f32, y: f32, z: f32) -> Self {
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
    pub fn look_at(eye: (f32, f32, f32), at: (f32, f32, f32), up: (f32, f32, f32)) -> Self {
        //todo!();
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let focal_length = 1.0 / (fovy_radians / 2.0).tan();
        Self([
            focal_length / aspect,           0.0,                       0.0,                       0.0,
                              0.0, -focal_length,                       0.0,                       0.0,
                              0.0,           0.0,        far / (near - far),                      -1.0,
                              0.0,           0.0, near * far / (near - far),                       1.0,
        ])
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    #[rustfmt::skip]
    fn mul(self, rhs: Vec4) -> Vec4 {
        let mut v = [0.0; 4];
        for row in 0..4 {
            v[row] = self.0[row * 4 + 0] * rhs.x +
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
        for idx in 0..4 {
            cols[idx] = self * rhs.col(idx);
        }

        Mat4::from_cols(cols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let m = Mat4::translate(10.0, 0.0, 0.0);
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
    fn perspective() {}
}
