use crate::*;

#[component]
pub fn ArcDrawer(arcs: Signal<Vec<(usize, usize)>>, points: Signal<Vec<Point>>) -> Element {
    let calculate_great_circle_arc = |p1: Vec3, p2: Vec3| -> Vec<Vec3> {
        let mut arc_points = Vec::new();
        let pq = (p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2]).clamp(-1.0, 1.0);
        let z = if pq.abs() > 1.0 - 1e-5 {
            let r = (p1[0].powi(2) + p1[1].powi(2)).sqrt();
            [-p1[1] / r, p1[0] / r, 0.0]
        } else {
            let r = (1.0 - pq.powi(2)).sqrt();
            [
                (p2[0] - p1[0] * pq) / r,
                (p2[1] - p1[1] * pq) / r,
                (p2[2] - p1[2] * pq) / r,
            ]
        };
        let t = pq.acos();
        let steps = 200;
        for i in 0..=steps {
            let theta = i as f64 * t / steps as f64;
            let point = [
                theta.cos() * p1[0] + theta.sin() * z[0],
                theta.cos() * p1[1] + theta.sin() * z[1],
                theta.cos() * p1[2] + theta.sin() * z[2],
            ];
            arc_points.push(point);
        }
        arc_points
    };

    rsx! {
        for (front_path_data , back_path_data) in arcs()
            .iter()
            .map(|&(p1_idx, p2_idx)| {
                let p1 = points()[p1_idx].rotated;
                let p2 = points()[p2_idx].rotated;
                let arc_points = calculate_great_circle_arc(p1, p2);
                let mut front_path = Vec::new();
                let mut back_path = Vec::new();
                for [x, y, z] in arc_points {
                    let svg_x = x * 25.0 + 50.0;
                    let svg_y = y * 25.0 + 50.0;
                    if z >= 0.0 {
                        front_path.push(format!("{},{}", svg_x, svg_y));
                    } else {
                        back_path.push(format!("{},{}", svg_x, svg_y));
                    }
                }
                let front_path_data = if !front_path.is_empty() {
                    "M ".to_string() + &front_path.join(" L ")
                } else {
                    String::new()
                };
                let back_path_data = if !back_path.is_empty() {
                    "M ".to_string() + &back_path.join(" L ")
                } else {
                    String::new()
                };
                (front_path_data, back_path_data)
            })
        {
            path {
                d: front_path_data,
                stroke: "#FFA500",
                stroke_width: "0.3",
                fill: "none",
            }
            path {
                d: back_path_data,
                stroke: "rgba(255, 165, 0, 0.4)",
                stroke_width: "0.3",
                fill: "none",
            }
        }
    }
}
