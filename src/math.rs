use std::ops::{Mul, Add};

#[repr(C)]
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
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

#[repr(C)]
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Matrix4 {
    pub v1: Vector4,
    pub v2: Vector4,
    pub v3: Vector4,
    pub v4: Vector4,   
}

impl Mul for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: Matrix4) -> Matrix4 {
        return Matrix4 {
            v1: self.v1 * rhs.v1.x + self.v2 * rhs.v1.y
            + self.v3 * rhs.v1.z + self.v4 * rhs.v1.w,
            v2: self.v1 * rhs.v2.x + self.v2 * rhs.v2.y
            + self.v3 * rhs.v2.z + self.v4 * rhs.v2.w,
            v3: self.v1 * rhs.v1.x + self.v2 * rhs.v1.y
            + self.v3 * rhs.v3.z + self.v4 * rhs.v3.w,
            v4: self.v1 * rhs.v4.x + self.v2 * rhs.v4.y
            + self.v3 * rhs.v4.z + self.v4 * rhs.v4.w,
        };
    }
}
