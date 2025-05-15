use dioxus::prelude::*;
use web_sys::window;

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

pub fn toggle_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().collect::<String>()
            } else if c.is_lowercase() {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct GreatCircle {
    pub pole: usize,
    pub name: String,
}

impl GreatCircle {
    pub fn new(pole: usize) -> Self {
        GreatCircle {
            pole,
            name: String::new(),
        }
    }
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

    pub fn new_inverted(&self, id: usize) -> Self {
        Point {
            id,
            absolute: [-self.absolute[0], -self.absolute[1], -self.absolute[2]],
            rotated: [-self.rotated[0], -self.rotated[1], -self.rotated[2]],
            abs_polar: vec3_to_polar([-self.absolute[0], -self.absolute[1], -self.absolute[2]]),
            rot_polar: vec3_to_polar([-self.rotated[0], -self.rotated[1], -self.rotated[2]]),
            name: String::new(),
            movable: true,
            removable: true,
        }
    }
}

pub fn snap_to_great_circle(
    point: Vec3,
    great_circles: &[GreatCircle],
    points: &[Point],
    threshold: f64,
) -> Vec3 {
    let mut closest_distance = threshold;
    let mut snapped_point = point;

    // Check each great circle
    for gc in great_circles {
        let pole = points[gc.pole].rotated;

        // Distance from point to great circle plane is |dot(point, pole)|
        let distance = (point[0] * pole[0] + point[1] * pole[1] + point[2] * pole[2]).abs();

        if distance < closest_distance {
            // Project point onto the plane of the circle (subtract the component along the pole)
            let dot_product = point[0] * pole[0] + point[1] * pole[1] + point[2] * pole[2];
            let projected = [
                point[0] - dot_product * pole[0],
                point[1] - dot_product * pole[1],
                point[2] - dot_product * pole[2],
            ];

            // Normalize to get a point on the sphere
            let mag = (projected[0].powi(2) + projected[1].powi(2) + projected[2].powi(2)).sqrt();
            if mag > 1e-10 {
                snapped_point = [projected[0] / mag, projected[1] / mag, projected[2] / mag];
                closest_distance = distance;
            }
        }
    }

    snapped_point
}

pub fn arc_distance(a: Vec3, b: Vec3) -> f64 {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let dot = dot.clamp(-1.0, 1.0);
    dot.acos()
}

pub fn calculate_angle(a: f64, b: f64, c: f64) -> [f64; 3] {
    let cos_a = a.cos();
    let cos_b = b.cos();
    let cos_c = c.cos();
    let sin_a = a.sin();
    let sin_b = b.sin();
    let sin_c = c.sin();

    // Use the spherical Law of Cosines for angles
    // cos(A) = (cos(a) - cos(b)cos(c)) / (sin(b)sin(c))

    // Handle potential divisions by zero
    let epsilon = 1e-10;

    // Calculate angle A
    let cos_angle_a = if sin_b.abs() < epsilon || sin_c.abs() < epsilon {
        1.0 // If sides are very small, angle approaches 0
    } else {
        (cos_a - cos_b * cos_c) / (sin_b * sin_c)
    };

    // Calculate angle B
    let cos_angle_b = if sin_a.abs() < epsilon || sin_c.abs() < epsilon {
        1.0
    } else {
        (cos_b - cos_a * cos_c) / (sin_a * sin_c)
    };

    // Calculate angle C
    let cos_angle_c = if sin_a.abs() < epsilon || sin_b.abs() < epsilon {
        1.0
    } else {
        (cos_c - cos_a * cos_b) / (sin_a * sin_b)
    };

    // Clamp values to handle floating point errors and take inverse cosine
    [
        cos_angle_a.clamp(-1.0, 1.0).acos(),
        cos_angle_b.clamp(-1.0, 1.0).acos(),
        cos_angle_c.clamp(-1.0, 1.0).acos(),
    ]
}

pub struct State {
    selected: Vec<usize>,
}

impl State {
    pub fn initialize() -> Self {
        Self { selected: vec![] }
    }

    pub fn selected(&self) -> &[usize] {
        self.selected.as_slice()
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn pop_selected(&mut self) -> Option<usize> {
        self.selected.pop()
    }

    pub fn toggle_select(&mut self, multi: bool, id: usize) -> bool {
        if multi {
            if self.selected.contains(&id) {
                self.selected.retain(|&x| x != id);
                false
            } else {
                self.selected.push(id);
                true
            }
        } else if self.selected.len() == 1 && self.selected[0] == id {
            self.selected.clear();
            false
        } else {
            self.selected.clear();
            self.selected.push(id);
            true
        }
    }

    pub fn select(&mut self, id: usize) -> bool {
        if self.selected.contains(&id) {
            false
        } else {
            self.selected.push(id);
            true
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
    pub fn from_euler_deg(euler: Vec3) -> Self {
        let [yaw, pitch, roll] = euler.map(|x| x.to_radians());
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

    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Self {
        // Normalize the axis vector
        let norm = (axis[0].powi(2) + axis[1].powi(2) + axis[2].powi(2)).sqrt();

        // Handle zero vector case
        if norm < 1e-10 {
            return Self::identity();
        }

        let normalized_axis = [axis[0] / norm, axis[1] / norm, axis[2] / norm];

        // Calculate quaternion components
        let half_angle = angle * 0.5;
        let sin_half_angle = half_angle.sin();
        let cos_half_angle = half_angle.cos();

        Quaternion {
            w: cos_half_angle,
            x: normalized_axis[0] * sin_half_angle,
            y: normalized_axis[1] * sin_half_angle,
            z: normalized_axis[2] * sin_half_angle,
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

    pub fn to_euler_deg(self) -> Vec3 {
        let Quaternion { w, x, y, z } = self;

        // Calculate Euler angles (yaw, pitch, roll) from quaternion
        let mut yaw = f64::atan2(2.0 * (w * z + x * y), 1.0 - 2.0 * (y * y + z * z)).to_degrees();
        let mut pitch = f64::asin(2.0 * (w * y - z * x)).to_degrees();
        let mut roll = f64::atan2(2.0 * (w * x + y * z), 1.0 - 2.0 * (x * x + y * y)).to_degrees();

        // Normalize angles to 0-360 range
        yaw = (yaw + 360.0) % 360.0;
        pitch = (pitch + 360.0) % 360.0;
        roll = (roll + 360.0) % 360.0;

        [yaw, pitch, roll]
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
