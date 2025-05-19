use crate::*;

#[component]
pub fn CoordinateGrid(scale: Signal<(f64, Vec3, Quaternion)>) -> Element {
    let mut lat_lines = Vec::new();
    let mut lon_lines = Vec::new();
    let rotation = scale().2;

    for i in 0..12 {
        let phi = i as f64 * std::f64::consts::TAU / 12.0;
        let mut points = Vec::new();
        for j in 0..=60 {
            let theta = j as f64 * std::f64::consts::PI / 60.0 - std::f64::consts::FRAC_PI_2;
            let x = phi.cos() * theta.cos();
            let y = phi.sin() * theta.cos();
            let z = theta.sin();
            let rotated = rotation.rotate_point_active([x, y, z]);
            points.push(rotated);
        }
        lon_lines.push(points);
    }

    for i in -5..=5 {
        if i == 0 {
            continue;
        }
        let theta = i as f64 * std::f64::consts::FRAC_PI_6;
        let mut points = Vec::new();
        for j in 0..=60 {
            let phi = j as f64 * std::f64::consts::TAU / 60.0;
            let x = phi.cos() * theta.cos();
            let y = phi.sin() * theta.cos();
            let z = theta.sin();
            let rotated = rotation.rotate_point_active([x, y, z]);
            points.push(rotated);
        }
        lat_lines.push(points);
    }

    let mut equator = Vec::new();
    for j in 0..=60 {
        let phi = j as f64 * std::f64::consts::TAU / 60.0;
        let point = [phi.cos(), phi.sin(), 0.0];
        let rotated = rotation.rotate_point_active(point);
        equator.push(rotated);
    }
    lat_lines.push(equator);

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
        for (i , front_path , back_path) in lat_lines
            .iter()
            .enumerate()
            .map(|(i, points)| {
                let (front_path, back_path) = transform_to_paths(points);
                (i, front_path, back_path)
            })
        {
            path {
                key: "lat-front-{i}",
                d: front_path,
                stroke: "#6B8E23",
                stroke_width: "0.15",
                stroke_dasharray: "0.5 0.5",
                fill: "none",
                opacity: "0.3",
            }
            path {
                key: "lat-back-{i}",
                d: back_path,
                stroke: "#6B8E23",
                stroke_width: "0.15",
                stroke_dasharray: "0.5 0.5",
                fill: "none",
                opacity: "0.1",
            }
        }
        for (i , front_path , back_path) in lon_lines
            .iter()
            .enumerate()
            .map(|(i, points)| {
                let (front_path, back_path) = transform_to_paths(points);
                (i, front_path, back_path)
            })
        {
            path {
                key: "lon-front-{i}",
                d: front_path,
                stroke: "#6B8E23",
                stroke_width: "0.15",
                stroke_dasharray: "0.5 0.5",
                fill: "none",
                opacity: "0.3",
            }
            path {
                key: "lon-back-{i}",
                d: back_path,
                stroke: "#6B8E23",
                stroke_width: "0.15",
                stroke_dasharray: "0.5 0.5",
                fill: "none",
                opacity: "0.1",
            }
        }
    }
}
