use crate::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    w: f64,
    x: f64,
    y: f64,
    z: f64,
}

impl Quaternion {
    pub fn from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> Self {
        let cr = (roll / 2.0).cos();
        let sr = (roll / 2.0).sin();
        let cp = (pitch / 2.0).cos();
        let sp = (pitch / 2.0).sin();
        let cy = (yaw / 2.0).cos();
        let sy = (yaw / 2.0).sin();

        Quaternion {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    // Multiply two quaternions
    pub fn multiply(self, other: Quaternion) -> Self {
        Quaternion {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    // Conjugate of the quaternion
    pub fn conjugate(self) -> Self {
        Quaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    // Rotate a point actively (rotating the point in the coordinate system)
    pub fn rotate_point_active(self, point: Vec3) -> Vec3 {
        let point_quaternion = Quaternion {
            w: 0.0,
            x: point.0,
            y: point.1,
            z: point.2,
        };
        let rotated = self.multiply(point_quaternion).multiply(self.conjugate());
        (rotated.x, rotated.y, rotated.z)
    }

    // Rotate a point passively (rotating the coordinate system)
    pub fn rotate_point_passive(self, point: Vec3) -> Vec3 {
        let conjugate = self.conjugate();
        let point_quaternion = Quaternion {
            w: 0.0,
            x: point.0,
            y: point.1,
            z: point.2,
        };
        let rotated = conjugate.multiply(point_quaternion).multiply(self);
        (rotated.x, rotated.y, rotated.z)
    }
}
