//! Small 3D math: vectors and column-major 4×4 matrices laid out like
//! raylib's (`m[col * 4 + row]`), so they pass straight to rlgl.

use raylib_sys::Matrix;
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const fn v3(x: f32, y: f32, z: f32) -> V3 {
    V3 { x, y, z }
}

impl V3 {
    pub fn dot(self, o: V3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn cross(self, o: V3) -> V3 {
        v3(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn norm(self) -> V3 {
        let l = self.len();
        if l < 1e-6 {
            v3(0.0, 1.0, 0.0)
        } else {
            self * (1.0 / l)
        }
    }
    pub fn lerp(self, o: V3, t: f32) -> V3 {
        self + (o - self) * t
    }
    pub fn arr(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl Add for V3 {
    type Output = V3;
    fn add(self, o: V3) -> V3 {
        v3(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for V3 {
    type Output = V3;
    fn sub(self, o: V3) -> V3 {
        v3(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f32> for V3 {
    type Output = V3;
    fn mul(self, k: f32) -> V3 {
        v3(self.x * k, self.y * k, self.z * k)
    }
}

impl Neg for V3 {
    type Output = V3;
    fn neg(self) -> V3 {
        v3(-self.x, -self.y, -self.z)
    }
}

/// A column-major 4×4 matrix: `m[col * 4 + row]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct M4(pub [f32; 16]);

impl M4 {
    pub const ID: M4 = M4([1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

    pub fn translate(p: V3) -> M4 {
        let mut m = M4::ID;
        m.0[12] = p.x;
        m.0[13] = p.y;
        m.0[14] = p.z;
        m
    }

    pub fn scale(x: f32, y: f32, z: f32) -> M4 {
        let mut m = M4::ID;
        m.0[0] = x;
        m.0[5] = y;
        m.0[10] = z;
        m
    }

    /// Turns by `a` radians about the up axis; 0 keeps +Z facing +Z.
    pub fn yaw(a: f32) -> M4 {
        let (s, c) = a.sin_cos();
        let mut m = M4::ID;
        m.0[0] = c;
        m.0[2] = -s;
        m.0[8] = s;
        m.0[10] = c;
        m
    }

    /// Turns by `a` radians about the Z axis (a roll, seen from the camera).
    pub fn roll(a: f32) -> M4 {
        let (s, c) = a.sin_cos();
        let mut m = M4::ID;
        m.0[0] = c;
        m.0[1] = s;
        m.0[4] = -s;
        m.0[5] = c;
        m
    }

    /// Turns by `a` radians about the X axis.
    pub fn pitch(a: f32) -> M4 {
        let (s, c) = a.sin_cos();
        let mut m = M4::ID;
        m.0[5] = c;
        m.0[6] = s;
        m.0[9] = -s;
        m.0[10] = c;
        m
    }

    /// `self` applied first, then `o`.
    pub fn then(&self, o: &M4) -> M4 {
        o.mul(self)
    }

    /// The product `self · o`: `o` applied first.
    pub fn mul(&self, o: &M4) -> M4 {
        let mut r = [0.0; 16];
        for c in 0..4 {
            for row in 0..4 {
                r[c * 4 + row] = (0..4).map(|k| self.0[k * 4 + row] * o.0[c * 4 + k]).sum();
            }
        }
        M4(r)
    }

    pub fn point(&self, p: V3) -> V3 {
        let m = &self.0;
        v3(
            m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12],
            m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13],
            m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14],
        )
    }

    pub fn dir(&self, p: V3) -> V3 {
        let m = &self.0;
        v3(m[0] * p.x + m[4] * p.y + m[8] * p.z, m[1] * p.x + m[5] * p.y + m[9] * p.z, m[2] * p.x + m[6] * p.y + m[10] * p.z)
    }

    /// The homogeneous image of `p`.
    pub fn project(&self, p: V3) -> [f32; 4] {
        let m = &self.0;
        [
            m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12],
            m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13],
            m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14],
            m[3] * p.x + m[7] * p.y + m[11] * p.z + m[15],
        ]
    }

    /// raylib's MatrixLookAt.
    pub fn look_at(eye: V3, target: V3, up: V3) -> M4 {
        let z = (eye - target).norm();
        let x = up.cross(z).norm();
        let y = z.cross(x);
        M4([x.x, y.x, z.x, 0.0, x.y, y.y, z.y, 0.0, x.z, y.z, z.z, 0.0, -x.dot(eye), -y.dot(eye), -z.dot(eye), 1.0])
    }

    /// A perspective projection like rlFrustum's, `fovy` in radians.
    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> M4 {
        let top = near * (fovy * 0.5).tan();
        let right = top * aspect;
        let (l, r, b, t, n, f) = (-right, right, -top, top, near, far);
        let mut m = [0.0; 16];
        m[0] = 2.0 * n / (r - l);
        m[5] = 2.0 * n / (t - b);
        m[8] = (r + l) / (r - l);
        m[9] = (t + b) / (t - b);
        m[10] = -(f + n) / (f - n);
        m[11] = -1.0;
        m[14] = -2.0 * f * n / (f - n);
        M4(m)
    }

    pub fn rl(&self) -> Matrix {
        let m = &self.0;
        Matrix {
            m0: m[0],
            m1: m[1],
            m2: m[2],
            m3: m[3],
            m4: m[4],
            m5: m[5],
            m6: m[6],
            m7: m[7],
            m8: m[8],
            m9: m[9],
            m10: m[10],
            m11: m[11],
            m12: m[12],
            m13: m[13],
            m14: m[14],
            m15: m[15],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrices_compose_in_order() {
        let m = M4::scale(2.0, 2.0, 2.0).then(&M4::translate(v3(1.0, 0.0, 0.0)));
        assert_eq!(m.point(v3(1.0, 1.0, 1.0)), v3(3.0, 2.0, 2.0));
        let y = M4::yaw(std::f32::consts::FRAC_PI_2).point(v3(0.0, 0.0, 1.0));
        assert!((y.x - 1.0).abs() < 1e-5 && y.z.abs() < 1e-5, "{y:?}");
    }

    #[test]
    fn look_at_puts_the_target_ahead() {
        let v = M4::look_at(v3(0.0, 5.0, 5.0), v3(0.0, 0.0, 0.0), v3(0.0, 1.0, 0.0));
        let p = v.point(v3(0.0, 0.0, 0.0));
        assert!(p.z < 0.0 && p.x.abs() < 1e-5 && p.y.abs() < 1e-4, "{p:?}");
    }
}
