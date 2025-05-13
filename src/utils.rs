use serde::{Deserialize, Serialize};
use web_sys::window;
use dioxus::prelude::*;

const SELECT: Asset = asset!("/assets/select.ico");
const TRIANGLE: Asset = asset!("/assets/triangle.ico");
pub const SAVE: Asset = asset!("/assets/save.ico");
pub const LOAD: Asset = asset!("/assets/load.ico");
pub const NEW_FILE: Asset = asset!("/assets/new.ico");

pub type Vec3 = [f64; 3];

#[derive(Debug, Clone)]
pub enum Selected {
    Existing(usize),
    New(Point),
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Point {
    pub id: usize,
    pub absolute: Vec3,
    pub rotated: Vec3,
    pub abs_polar: [f64; 2],
    pub rot_polar: [f64; 2],
    pub name: String,
    pub movable: bool,
    pub removable: bool,
}

impl Point {
    pub fn from_vec3(id: usize, vec: Vec3) -> Self {
        Point {
            id,
            absolute: vec,
            rotated: vec,
            name: String::new(),
            movable: true,
            removable: true,
            abs_polar: vec3_to_polar(vec),
            rot_polar: vec3_to_polar(vec),
        }
    }

    pub fn from_vec3_rotated(id: usize, vec: Vec3, q: Quaternion) -> Self {
        let absolute = q.rotate_point_passive(vec);
        Point {
            id,
            absolute,
            rotated: vec,
            name: String::new(),
            movable: true,
            removable: true,
            abs_polar: vec3_to_polar(absolute),
            rot_polar: vec3_to_polar(vec),
        }
    }

    pub fn move_to(&mut self, vec: Vec3, q: Quaternion) {
        if !self.movable {
            return;
        }
        self.absolute = q.rotate_point_passive(vec);
        self.abs_polar = vec3_to_polar(self.absolute);
        self.rotated = vec;
        self.rot_polar = vec3_to_polar(vec);
    }

    pub fn rotate(&mut self, q: Quaternion) {
        self.rotated = q.rotate_point_active(self.absolute);
        self.rot_polar = vec3_to_polar(self.rotated);
    }

    pub fn name(&mut self, name: String) {
        self.name = name;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Selection,
    Triangle,
}

impl Mode {
    pub const MODES: [Mode; 2] = [Mode::Selection, Mode::Triangle];

    pub fn icon(&self) -> Asset {
        match self {
            Mode::Selection => SELECT,
            Mode::Triangle => TRIANGLE,
        }
    }
}

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

    pub fn identity() -> Self {
        Quaternion {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
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
            x: point[0],
            y: point[1],
            z: point[2],
        };
        let rotated = self.multiply(point_quaternion).multiply(self.conjugate());
        [rotated.x, rotated.y, rotated.z]
    }

    // Rotate a point passively (rotating the coordinate system)
    pub fn rotate_point_passive(self, point: Vec3) -> Vec3 {
        let conjugate = self.conjugate();
        let point_quaternion = Quaternion {
            w: 0.0,
            x: point[0],
            y: point[1],
            z: point[2],
        };
        let rotated = conjugate.multiply(point_quaternion).multiply(self);
        [rotated.x, rotated.y, rotated.z]
    }
}

pub fn transform_viewport_to_sphere(viewport_x: f64, viewport_y: f64) -> Vec3 {
    let nan = [f64::NAN; 3];
    let Some(window) = window() else {
        return nan;
    };
    let Some(document) = window.document() else {
        return nan;
    };
    let Some(Some(circle_element)) = document.query_selector("circle").ok() else {
        return nan;
    };

    let rect = circle_element.get_bounding_client_rect();

    let (circle_left, circle_top, circle_width, circle_height) =
        (rect.left(), rect.top(), rect.width(), rect.height());

    let circle_x = (viewport_x - circle_left - circle_width / 2.0) / circle_width * 2.0;
    let circle_y = (viewport_y - circle_top - circle_height / 2.0) / circle_height * 2.0;

    let r2 = circle_x.powi(2) + circle_y.powi(2);
    if r2 <= 1.0 {
        return [circle_x, circle_y, (1.0 - r2).sqrt()];
    }

    nan
}

fn vec3_to_polar(vec: Vec3) -> [f64; 2] {
    // y = 0 is the equator and x = 0 is the meridian. r = 1. theta goes from - 90 to 90, phi from 0 to 360
    let [x, y, z] = vec;
    let theta = y.asin().to_degrees();
    let phi = x.atan2(z).to_degrees();
    let phi = if phi < 0.0 { phi + 360.0 } else { phi };
    [theta, phi]
}
