use std::ops::{Add, Div, Mul, Sub};

// vector oh yeah!!! this is basically the type used for everything from 3d rotation to
// position. addition and subtraction between Vec3fs is implemented and multiplication and division
// are only implemented with an f32.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn z(&self) -> f32 {
        self.z
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl Add<Vec3f> for Vec3f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<Vec3f> for Vec3f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vec3f {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

// rotation matrix yummy
pub struct RotationMatrix {
    aa: f32,
    ab: f32,
    ac: f32,
    ba: f32,
    bb: f32,
    bc: f32,
    ca: f32,
    cb: f32,
    cc: f32,
}

impl RotationMatrix {
    pub fn from_euler(angle: Vec3f) -> Self {
        // pitch roll yaw
        let a = Vec3f {
            x: -angle.x.to_radians(),
            y: -angle.y.to_radians(),
            z: -angle.z.to_radians(),
        };

        let xsin = a.x.sin();
        let xcos = a.x.cos();

        let ysin = a.y.sin();
        let ycos = a.y.cos();

        let zsin = a.z.sin();
        let zcos = a.z.cos();

        Self {
            aa: ycos * zcos,
            ab: xsin * ysin * zcos - zsin * xcos,
            ac: ysin * xcos * zcos + xsin * zsin,

            ba: zsin * ycos,
            bb: xsin * ysin * zsin + xcos * zcos,
            bc: ysin * zsin * xcos - xsin * zcos,

            ca: -ysin,
            cb: xsin * ycos,
            cc: xcos * ycos,
        }
    }

    pub fn rotate_vector(&self, v: Vec3f) -> Vec3f {
        Vec3f {
            x: self.aa * v.x + self.ab * v.y + self.ac * v.z,
            y: self.ba * v.x + self.bb * v.y + self.bc * v.z,
            z: self.ca * v.x + self.cb * v.y + self.cc * v.z,
        }
    }
}

// wow triangles so cool!! this is a simple struct for storing the data of a triangle, i may add
// color soon:tm:
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    pub v1: Vec3f,
    pub v2: Vec3f,
    pub v3: Vec3f,
}

impl Triangle {
    pub fn from_points(v1: Vec3f, v2: Vec3f, v3: Vec3f) -> Self {
        Self { v1, v2, v3 }
    }
}

// this only exists to store the camera's info.
pub struct Camera {
    pub pos: Vec3f, // x y z
    pub rot: Vec3f, // pitch roll yaw
    pub fov: f32,
}
