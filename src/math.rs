use std::ops::{Mul, Add};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Point3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[repr(C, align(8))]
#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,  
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub fn from_v3_f32(v: Vector3, f: f32) -> Vector4 {
        return Vector4 { x: v.x, y: v.y, z: v.z, w: f };
    }

    pub fn from_v3_0(v: Vector3) -> Vector4 {
        return Self::from_v3_f32(v, 0.0);
    }

    pub fn from_v3_1(v: Vector3) -> Vector4 {
        return Self::from_v3_f32(v, 1.0);
    }    
}

impl Add for Vector4 {
    type Output = Vector4;
    fn add(self, rhs: Vector4) -> Vector4 {
        Vector4 {
            x: self.x + rhs.x, y: self.y + rhs.y,
            z: self.z + rhs.z, w: self.w + rhs.w,
        }
    }
}

impl Mul for Vector4 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Vector4 {
        Vector4 {
            x: self.x * rhs.x, y: self.y * rhs.y,
            z: self.z * rhs.z, w: self.w * rhs.w,
        }
    }
}

impl Mul<f32> for Vector4 {
    type Output = Vector4;
    fn mul(self, rhs: f32) -> Vector4 {
        Vector4 {
            x: self.x * rhs, y: self.y * rhs,
            z: self.z * rhs, w: self.w * rhs,
        }
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Matrix2 {
    pub v1: Vector2,
    pub v2: Vector2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Matrix3 {
    pub v1: Vector3,
    pub v2: Vector3,
    pub v3: Vector3,
}

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Matrix4 {
    pub v1: Vector4,
    pub v2: Vector4,
    pub v3: Vector4,
    pub v4: Vector4,
}

pub static M4_IDENTITY: Matrix4 = Matrix4 {
    v1: Vector4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 },
    v2: Vector4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 },
    v3: Vector4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 },
    v4: Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
};

impl Matrix4 {
    // Matrix4's can often be compressed to
    // a Matrix3 + Vector3. Decompress that.
    pub fn from_m3_v3(m: Matrix3, v: Vector3) -> Matrix4 {
        return Matrix4 {
            v1: Vector4::from_v3_0(m.v1),
            v2: Vector4::from_v3_0(m.v2),
            v3: Vector4::from_v3_0(m.v3),
            v4: Vector4::from_v3_1(v),
        }
    }
}

impl Mul<f32> for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: f32) -> Matrix4 {
        return Matrix4 {
            v1: self.v1 * rhs,
            v2: self.v2 * rhs,
            v3: self.v3 * rhs,
            v4: self.v4 * rhs,
        };
    }
}

impl Mul<Vector4> for Matrix4 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Vector4 {
        return self.v1 * rhs.x + self.v2 * rhs.y
            + self.v3 * rhs.z + self.v4 * rhs.w;
    }
}

impl Mul for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: Matrix4) -> Matrix4 {
        return Matrix4 {
            v1: self * rhs.v1,
            v2: self * rhs.v2,
            v3: self * rhs.v3,
            v4: self * rhs.v4,
        };
    }
}
