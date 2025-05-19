use crate::*;

#[derive(Debug, Clone)]
pub struct SmallCircle {
    pub pole: usize,
    pub plane_distance: f64,
    pub name: String,
}

impl SmallCircle {
    pub fn new(pole: usize, plane_distance: f64) -> Self {
        Self {
            pole,
            plane_distance,
            name: String::new(),
        }
    }
}

#[component]
pub fn SmallCircleDrawer(
    small_circles: Signal<Vec<SmallCircle>>,
    points: Signal<Vec<Point>>,
) -> Element {
    let calculate_small_circle = |pole: Vec3, distance: f64| -> Vec<Vec3> {
        let mut circle_points = Vec::new();
        let steps = 200;
        let center = [pole[0] * distance, pole[1] * distance, pole[2] * distance];
        let radius = (1.0 - distance * distance).sqrt();
        let u = if pole[0].abs() < pole[1].abs() && pole[0].abs() < pole[2].abs() {
            [0.0, pole[2], -pole[1]]
        } else if pole[1].abs() < pole[2].abs() {
            [-pole[2], 0.0, pole[0]]
        } else {
            [pole[1], -pole[0], 0.0]
        };
        let u_mag = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let u = [u[0] / u_mag, u[1] / u_mag, u[2] / u_mag];
        let v = [
            pole[1] * u[2] - pole[2] * u[1],
            pole[2] * u[0] - pole[0] * u[2],
            pole[0] * u[1] - pole[1] * u[0],
        ];
        for i in 0..=steps {
            let angle = i as f64 * std::f64::consts::TAU / steps as f64;
            let point = [
                center[0] + radius * (angle.cos() * u[0] + angle.sin() * v[0]),
                center[1] + radius * (angle.cos() * u[1] + angle.sin() * v[1]),
                center[2] + radius * (angle.cos() * u[2] + angle.sin() * v[2]),
            ];
            circle_points.push(point);
        }
        circle_points
    };

    let transform_to_paths = |points: &Vec<Vec3>| {
        let mut front_segments = Vec::new();
        let mut back_segments = Vec::new();
        let mut current_front = Vec::new();
        let mut current_back = Vec::new();
        let mut last_z_positive = None;
        for i in 0..points.len() {
            let [x, y, z] = points[i];
            let z_positive = z >= 0.0;
            let svg_x = x * 25.0 + 50.0;
            let svg_y = y * 25.0 + 50.0;
            if i == 0 {
                if z_positive {
                    current_front.push(format!("{},{}", svg_x, svg_y));
                } else {
                    current_back.push(format!("{},{}", svg_x, svg_y));
                }
                last_z_positive = Some(z_positive);
                continue;
            }
            if let Some(was_positive) = last_z_positive {
                if was_positive != z_positive {
                    let prev = points[i - 1];
                    let t = prev[2] / (prev[2] - z);
                    let ix = prev[0] + t * (x - prev[0]);
                    let iy = prev[1] + t * (y - prev[1]);
                    let isvg_x = ix * 25.0 + 50.0;
                    let isvg_y = iy * 25.0 + 50.0;
                    if was_positive {
                        current_front.push(format!("{},{}", isvg_x, isvg_y));
                        front_segments.push(current_front);
                        current_front = Vec::new();
                        current_back.push(format!("{},{}", isvg_x, isvg_y));
                    } else {
                        current_back.push(format!("{},{}", isvg_x, isvg_y));
                        back_segments.push(current_back);
                        current_back = Vec::new();
                        current_front.push(format!("{},{}", isvg_x, isvg_y));
                    }
                }
            }
            if z_positive {
                current_front.push(format!("{},{}", svg_x, svg_y));
            } else {
                current_back.push(format!("{},{}", svg_x, svg_y));
            }
            last_z_positive = Some(z_positive);
        }
        if !current_front.is_empty() {
            front_segments.push(current_front);
        }
        if !current_back.is_empty() {
            back_segments.push(current_back);
        }
        let front_data = front_segments
            .iter()
            .filter(|segment| !segment.is_empty())
            .map(|segment| "M ".to_string() + &segment.join(" L "))
            .collect::<Vec<_>>()
            .join(" ");
        let back_data = back_segments
            .iter()
            .filter(|segment| !segment.is_empty())
            .map(|segment| "M ".to_string() + &segment.join(" L "))
            .collect::<Vec<_>>()
            .join(" ");
        (front_data, back_data)
    };

    rsx! {
        for (i , front_path_data , back_path_data) in small_circles
            .read()
            .iter()
            .enumerate()
            .map(|(i, sc)| {
                let pole = points()[sc.pole].rotated;
                let circle_points = calculate_small_circle(pole, sc.plane_distance);
                let (front_path_data, back_path_data) = transform_to_paths(&circle_points);
                (i, front_path_data, back_path_data)
            })
        {
            path {
                key: "sc-front-{i}",
                d: front_path_data,
                stroke: "cyan",
                stroke_width: "0.3",
                fill: "none",
            }
            path {
                key: "sc-back-{i}",
                d: back_path_data,
                stroke: "rgba(0, 255, 255, 0.4)",
                stroke_width: "0.3",
                fill: "none",
            }
        }
    }
}

#[component]
pub fn SmallCircleLabels(
    small_circles: Signal<Vec<SmallCircle>>,
    points: Signal<Vec<Point>>,
) -> Element {
    rsx! {
        for (i , svg_x , svg_y , opacity , name) in small_circles
            .read()
            .iter()
            .enumerate()
            .filter_map(|(i, sc)| {
                let [x, y, z] = points()[sc.pole].rotated;
                let svg_x = x * 25.0 + 50.0;
                let svg_y = y * 25.0 + 50.0;
                let opacity = if z > 0.0 { 1.0 } else { 0.4 };
                if !sc.name.is_empty() {
                    Some((i, svg_x, svg_y, opacity, &sc.name))
                } else {
                    None
                }
            })
        {
            text {
                key: "sc-label-{i}",
                x: "{svg_x}",
                y: "{svg_y + 3.0}",
                fill: "rgba(0, 255, 255, {opacity})",
                font_family: "Arial",
                font_size: "1.5",
                text_anchor: "middle",
                "{name}"
            }
        }
    }
}
