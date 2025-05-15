use crate::*;

#[component]
pub fn ArcDrawer(arcs: Signal<Vec<(usize, usize)>>, points: Signal<Vec<Point>>) -> Element {
    // Function to calculate the great circle arc points
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
        let steps = 200; // Number of points along the arc
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

#[component]
pub fn GreatCircleDrawer(
    great_circles: Signal<Vec<GreatCircle>>,
    points: Signal<Vec<Point>>,
) -> Element {
    let calculate_great_circle = |pole: Vec3| -> Vec<Vec3> {
        let mut circle_points = Vec::new();
        let steps = 200; // Number of points along the great circle
        let [x, y, z] = pole;

        let r2 = (x.powi(2) + y.powi(2)).sqrt();
        let u = [-y / r2, x / r2, 0.0]; // u = p × (0, 0, 1)
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

#[component]
pub fn CoordinateGrid(scale: Signal<(f64, Vec3, Quaternion)>) -> Element {
    let mut lat_lines = Vec::new();
    let mut lon_lines = Vec::new();

    // Get current rotation quaternion
    let rotation = scale().2;

    // Create longitude lines (meridians)
    for i in 0..12 {
        let phi = i as f64 * std::f64::consts::TAU / 12.0; // Every 30 degrees of longitude
        let mut points = Vec::new();
        for j in 0..=60 {
            // More points for smoother curves
            // Calculate initial unrotated point
            let theta = j as f64 * std::f64::consts::PI / 60.0 - std::f64::consts::FRAC_PI_2;
            let x = phi.cos() * theta.cos();
            let y = phi.sin() * theta.cos();
            let z = theta.sin();

            // Apply rotation quaternion to the point
            let rotated = rotation.rotate_point_active([x, y, z]);
            points.push(rotated);
        }
        lon_lines.push(points);
    }

    // Create latitude lines (parallels)
    for i in -5..=5 {
        if i == 0 {
            continue;
        } // Skip equator for now (we'll add it separately)

        let theta = i as f64 * std::f64::consts::FRAC_PI_6; // Every 30 degrees of latitude
        let mut points = Vec::new();
        for j in 0..=60 {
            // More points for smoother circles
            // Calculate initial unrotated point
            let phi = j as f64 * std::f64::consts::TAU / 60.0;
            let x = phi.cos() * theta.cos();
            let y = phi.sin() * theta.cos();
            let z = theta.sin();

            // Apply rotation quaternion to the point
            let rotated = rotation.rotate_point_active([x, y, z]);
            points.push(rotated);
        }
        lat_lines.push(points);
    }

    // Equator (special case at theta = 0)
    let mut equator = Vec::new();
    for j in 0..=60 {
        // Calculate initial unrotated point
        let phi = j as f64 * std::f64::consts::TAU / 60.0;
        let point = [phi.cos(), phi.sin(), 0.0];

        // Apply rotation quaternion to the point
        let rotated = rotation.rotate_point_active(point);
        equator.push(rotated);
    }
    lat_lines.push(equator);

    // Replace the transform_to_paths function in your CoordinateGrid component
    let transform_to_paths = |points: &Vec<Vec3>| {
        // We'll create multiple front and back segments to prevent straight lines through the sphere
        let mut front_segments = Vec::new();
        let mut back_segments = Vec::new();
        let mut current_front = Vec::new();
        let mut current_back = Vec::new();

        // Track whether we're currently on the front or back of the sphere
        let mut last_z_positive = None;

        // For each pair of consecutive points
        for i in 0..points.len() {
            let [x, y, z] = points[i];
            let z_positive = z >= 0.0;
            let svg_x = x * 25.0 + 50.0;
            let svg_y = y * 25.0 + 50.0;

            // If this is the first point, just store its position
            if i == 0 {
                if z_positive {
                    current_front.push(format!("{},{}", svg_x, svg_y));
                } else {
                    current_back.push(format!("{},{}", svg_x, svg_y));
                }
                last_z_positive = Some(z_positive);
                continue;
            }

            // We're crossing the z=0 boundary if the sign of z changed
            if let Some(was_positive) = last_z_positive {
                if was_positive != z_positive {
                    // Find intersection with z=0 plane
                    let prev = points[i - 1];

                    // Calculate intersection point (linear interpolation)
                    let t = prev[2] / (prev[2] - z); // 0 <= t <= 1
                    let ix = prev[0] + t * (x - prev[0]);
                    let iy = prev[1] + t * (y - prev[1]);

                    // Convert to SVG coordinates
                    let isvg_x = ix * 25.0 + 50.0;
                    let isvg_y = iy * 25.0 + 50.0;

                    // Add intersection point to both current segments
                    if was_positive {
                        current_front.push(format!("{},{}", isvg_x, isvg_y));
                        // Finish front segment and start a new back segment
                        front_segments.push(current_front);
                        current_front = Vec::new();
                        current_back.push(format!("{},{}", isvg_x, isvg_y));
                    } else {
                        current_back.push(format!("{},{}", isvg_x, isvg_y));
                        // Finish back segment and start a new front segment
                        back_segments.push(current_back);
                        current_back = Vec::new();
                        current_front.push(format!("{},{}", isvg_x, isvg_y));
                    }
                }
            }

            // Add the current point to the appropriate segment
            if z_positive {
                current_front.push(format!("{},{}", svg_x, svg_y));
            } else {
                current_back.push(format!("{},{}", svg_x, svg_y));
            }

            last_z_positive = Some(z_positive);
        }

        // Don't forget to add the final segments if they contain any points
        if !current_front.is_empty() {
            front_segments.push(current_front);
        }
        if !current_back.is_empty() {
            back_segments.push(current_back);
        }

        // Create SVG path data from segments
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
        // Draw latitude lines
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
                stroke: "#6B8E23", // Olive green
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
                opacity: "0.1", // Lower opacity for back side
            }
        }
        // Draw longitude lines
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
                opacity: "0.1", // Lower opacity for back side
            }
        }
    }
}
