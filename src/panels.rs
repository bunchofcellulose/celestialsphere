use crate::*;

#[component]
pub fn SelectionBox(points: Signal<Vec<Point>>, selected_point: Signal<Option<usize>>) -> Element {
    let Some(i) = selected_point() else {
        return rsx! {};
    };
    let [x, y, z] = points.read()[i].absolute;
    let [rx, ry, rz] = points.read()[i].rotated;
    let [theta, phi] = points.read()[i].abs_polar;
    let [rtheta, rphi] = points.read()[i].rot_polar;
    let name = &points.read()[i].name;
    let movable = points.read()[i].movable;
    let removable = points.read()[i].removable;

    rsx! {
        div { class: "selection-box",
            "Absolute Coordinates:"
            br {}
            "x: {x:.2}, y: {-y:.2}, z: {z:.2}"
            br {}
            "θ: {-theta:.2}, φ: {phi:.2}°"
            br {}
            br {}
            "Rotated Frame Coordinates:"
            br {}
            "x: {rx:.2}, y: {-ry:.2}, z: {rz:.2}"
            br {}
            "θ: {-rtheta:.2}, φ: {rphi:.2}"
            br {}
            br {}
            "ID: {i}"
            br {}
            "Name: {name}"
            br {}
            div {
                input {
                    r#type: "checkbox",
                    checked: "{movable}",
                    onchange: move |event| {
                        match event.value().as_str() {
                            "true" => points.write()[i].movable = true,
                            "false" => points.write()[i].movable = false,
                            a => panic!("{a}"),
                        }
                    },
                }
                span { "Movable" }
            }
            div {
                input {
                    r#type: "checkbox",
                    checked: "{removable}",
                    onchange: move |event| {
                        match event.value().as_str() {
                            "true" => points.write()[i].removable = true,
                            "false" => points.write()[i].removable = false,
                            a => panic!("{a}"),
                        }
                    },
                }
                span { "Removable" }
            }
        }
    }
}

#[component]
pub fn TriangleInfoBox(points: Signal<Vec<Point>>) -> Element {
    if points.read().len() < 3 {
        return rsx! {}; // Do not display if there are fewer than 3 points
    }

    // Get the last three points
    let len = points.read().len();
    let p1 = points.read()[len - 3].absolute;
    let p2 = points.read()[len - 2].absolute;
    let p3 = points.read()[len - 1].absolute;

    // Function to calculate the angular distance between two points on a sphere
    let angular_distance = |a: Vec3, b: Vec3| -> f64 {
        let dot_product = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        dot_product.acos()
    };

    // Calculate the lengths of the sides
    let a = angular_distance(p2, p3);
    let b = angular_distance(p1, p3);
    let c = angular_distance(p1, p2);

    // Calculate the angles of the triangle using the spherical law of cosines
    let angle_a = ((-b.cos() * c.cos() + a.cos()) / (b.sin() * c.sin())).acos();
    let angle_b = ((-a.cos() * c.cos() + b.cos()) / (a.sin() * c.sin())).acos();
    let angle_c = ((-a.cos() * b.cos() + c.cos()) / (a.sin() * b.sin())).acos();

    rsx! {
        div { class: "triangle-info-box",
            "Spherical Triangle Information:"
            br {}
            "Sides:"
            br {}
            "a: {a.to_degrees():.2}°, b: {b.to_degrees():.2}°, c: {c.to_degrees():.2}°"
            br {}
            "Angles:"
            br {}
            "A: {angle_a.to_degrees():.2}°, B: {angle_b.to_degrees():.2}°, C: {angle_c.to_degrees():.2}°"
        }
    }
}

#[component]
pub fn SlidersPanel(points: Signal<Vec<Point>>, scale: Signal<(f64, Vec3, Quaternion)>) -> Element {
    let mut change = move || {
        let q = Quaternion::from_euler_angles(
            scale().1[0].to_radians(),
            scale().1[1].to_radians(),
            scale().1[2].to_radians(),
        );
        scale.write().2 = q;
        for p in points.write().iter_mut() {
            p.rotate(q);
        }
    };
    rsx! {
        div { class: "sliders-panel",
            div {
                span { "X scale: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    value: "{scale().1[0]}",
                    oninput: move |evt| {
                        scale.write().1[0] = evt.value().parse::<f64>().unwrap_or(0.0);
                        change();
                    },
                }
                span { "{scale().1[0]}°" }
            }
            div {
                span { "Y scale: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    step: "1",
                    value: "{scale().1[1]}",
                    oninput: move |evt| {
                        scale.write().1[1] = evt.value().parse::<f64>().unwrap_or(0.0);
                        change();
                    },
                }
                span { "{scale().1[1]}°" }
            }
            div {
                span { "Z scale: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    step: "1",
                    value: "{scale().1[2]}",
                    oninput: move |evt| {
                        scale.write().1[2] = evt.value().parse::<f64>().unwrap_or(0.0);
                        change();
                    },
                }
                span { "{scale().1[2]}°" }
            }
            div {
                span { "Zoom: " }
                input {
                    r#type: "range",
                    min: "0.5",
                    max: "2.0",
                    step: "0.01",
                    value: "{scale().0}",
                    oninput: move |evt| {
                        scale.write().0 = evt.value().parse::<f64>().unwrap_or(1.0);
                    },
                }
                span { "{(scale().0 * 100.0).round()}%" }
            }
        }
    }
}

#[component]
pub fn ModePanel(active_mode: Signal<Mode>) -> Element {
    rsx! {
        div { class: "mode-panel",
            for (mode , icon , color) in Mode::MODES
                .iter()
                .map(|&mode| {
                    (
                        mode,
                        mode.icon(),
                        if mode == active_mode() { "active" } else { "inactive" },
                    )
                })
            {
                button {
                    class: "{color}",
                    style: "background-image: url({icon});",
                    onclick: move |_| {
                        active_mode.set(mode);
                    },
                }
            }
        }
    }
}

#[component]
pub fn SubModePanel(active_mode: Signal<Mode>, points: Signal<Vec<Point>>) -> Element {
    match active_mode() {
        Mode::Selection => rsx! {},
        Mode::Triangle => rsx! {
            div { class: "left-panel", TrianglePanel {} }
            TriangleInfoBox { points }
        },
    }
}

#[component]
pub fn TrianglePanel() -> Element {
    rsx! {}
}
