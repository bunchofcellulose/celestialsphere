use crate::*;

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

#[component]
pub fn GreatCircleDrawer(
    great_circles: Signal<Vec<GreatCircle>>,
    points: Signal<Vec<Point>>,
) -> Element {
    let calculate_great_circle = |pole: Vec3| -> Vec<Vec3> {
        let mut circle_points = Vec::new();
        let steps = 200;
        let [x, y, z] = pole;
        let r2 = (x.powi(2) + y.powi(2)).sqrt();
        let u = [-y / r2, x / r2, 0.0];
        let v = [-z * u[1], z * u[0], r2];
        for i in 0..=steps {
            let theta = i as f64 * std::f64::consts::TAU / steps as f64;
            let point = [
                theta.cos() * u[0] + theta.sin() * v[0],
                theta.cos() * u[1] + theta.sin() * v[1],
                theta.cos() * u[2] + theta.sin() * v[2],
            ];
            circle_points.push(point);
        }
        circle_points
    };

    rsx! {
        for (front_path_data , back_path_data) in great_circles()
            .iter()
            .map(|gc| {
                let pole = points()[gc.pole].rotated;
                let circle_points = calculate_great_circle(pole);
                let mut front_path = Vec::new();
                let mut back_path = Vec::new();
                for [x, y, z] in circle_points {
                    let svg_x = x * 25.0 + 50.0;
                    let svg_y = y * 25.0 + 50.0;
                    if z >= 0.0 {
                        front_path.push(format!("{},{}", svg_x, svg_y));
                    } else {
                        back_path.push(format!("{},{}", svg_x, svg_y));
                    }
                }
                let front_path_data = "M ".to_string() + &front_path.join(" L ");
                let back_path_data = "M ".to_string() + &back_path.join(" L ");
                (front_path_data, back_path_data)
            })
        {
            path {
                d: front_path_data,
                stroke: "lime",
                stroke_width: "0.3",
                fill: "none",
            }
            path {
                d: back_path_data,
                stroke: "rgba(0, 255, 0, 0.4)",
                stroke_width: "0.3",
                fill: "none",
            }
        }
    }
}

#[component]
pub fn GreatCircleLabels(
    great_circles: Signal<Vec<GreatCircle>>,
    points: Signal<Vec<Point>>,
) -> Element {
    rsx! {
        for (i , name , x , y) in great_circles()
            .iter()
            .enumerate()
            .filter_map(|(idx, gc)| {
                if gc.name.is_empty() {
                    return None;
                }
                let pole = points()[gc.pole].rotated;
                let [px, py, _] = pole;
                let r2 = (px.powi(2) + py.powi(2)).sqrt();
                if r2 < 1e-5 {
                    Some((idx, gc.name.clone(), 75.0, 50.0))
                } else {
                    let x_intersect = py / r2;
                    let y_intersect = -px / r2;
                    let svg_x = x_intersect * 25.0 + 50.0;
                    let svg_y = y_intersect * 25.0 + 50.0;
                    Some((idx, gc.name.clone(), svg_x, svg_y))
                }
            })
        {
            text {
                key: "gctext-{i}",
                x: "{x - 2.0}",
                y: "{y - 2.0}",
                font_family: "Arial",
                font_size: "2",
                text_anchor: "middle",
                fill: "white",
                style: "font-weight: bold; user-select: none;",
                "{name}"
            }
        }
    }
}
