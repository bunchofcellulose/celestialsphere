use dioxus::{html::input_data::MouseButton, prelude::*};
use web_sys::window;

mod quaternion;
use quaternion::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

type Vec3 = (f64, f64, f64);

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let selected_point = use_signal(|| None::<(usize, Vec3, Vec3, String)>);
    let rotation = use_signal(|| ((0.0, 0.0, 0.0), Quaternion::identity()));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        CoordinatesBox { selected_point }
        SlidersPanel { rotation, selected_point }
        Sphere { selected_point, rotation }
    }
}

#[component]
pub fn Sphere(
    selected_point: Signal<Option<(usize, Vec3, Vec3, String)>>,
    rotation: Signal<(Vec3, Quaternion)>,
) -> Element {
    let mut points = use_signal(Vec::<(Vec3, String)>::new);
    let mut arcs = use_signal(Vec::<(usize, usize)>::new);
    let mut dragged_point = use_signal(|| None::<usize>);

    let mut primary_click = move |event: Event<MouseData>| {
        let viewport_x = event.client_coordinates().x;
        let viewport_y = event.client_coordinates().y;
        let (px, py, pz) = transform_viewport_to_sphere(viewport_x, viewport_y);
        if pz.is_nan() {
            return;
        }
        let q = rotation().1;

        let mut dot_clicked = false;
        for (i, &(orig_point, ref name)) in points.read().iter().enumerate() {
            let point = q.rotate_point_active(orig_point);
            if point.2 < 0.0 {
                continue;
            }
            let dx = px - point.0;
            let dy = py - point.1;
            if dx.powi(2) + dy.powi(2) <= 0.002 {
                selected_point.set(Some((i, orig_point, point, name.clone())));
                dragged_point.set(Some(i));
                dot_clicked = true;
                break;
            }
        }
        if !dot_clicked {
            let new = q.rotate_point_passive((px, py, pz));
            points.write().push((new, String::new()));
            selected_point.set(Some((points.len() - 1, new, (px, py, pz), String::new())));
        }
    };

    let mut secondary_click = move |event: Event<MouseData>| {
        let Some((pt, _, _, _)) = selected_point() else {
            return;
        };
        let viewport_x = event.client_coordinates().x;
        let viewport_y = event.client_coordinates().y;
        let (px, py, pz) = transform_viewport_to_sphere(viewport_x, viewport_y);
        if pz.is_nan() {
            return;
        }
        let q = rotation().1;

        for (i, &(orig_point, _)) in points.read().iter().enumerate() {
            let point = q.rotate_point_active(orig_point);
            if point.2 < 0.0 {
                continue;
            }
            let dx = px - point.0;
            let dy = py - point.1;
            if dx.powi(2) + dy.powi(2) <= 0.002 {
                if pt == i {
                    return;
                }
                arcs.write().push((pt, i));
                return;
            }
        }
    };

    let mouse_move = move |event: Event<MouseData>| {
        if let Some(dragged_idx) = dragged_point() {
            let viewport_x = event.client_coordinates().x;
            let viewport_y = event.client_coordinates().y;
            let (px, py, pz) = transform_viewport_to_sphere(viewport_x, viewport_y);
            if pz.is_nan() {
                return;
            }
            let q = rotation().1;
            let new = q.rotate_point_passive((px, py, pz));
            points.write()[dragged_idx].0 = new; // Update the position of the dragged dot
            if let Some((i, _, _, name)) = selected_point() {
                if i == dragged_idx {
                    selected_point.set(Some((i, new, (px, py, pz), name)));
                }
            } else {
                selected_point.set(Some((dragged_idx, new, (px, py, pz), points.read()[dragged_idx].1.clone())));
            }
        }
    };

    let mouse_up = move |_event: Event<MouseData>| {
        dragged_point.set(None);
    };

    let key_event = move |event: Event<KeyboardData>| {
        event.prevent_default();
        if let Some((i, p, p_rot, mut name)) = selected_point() {
            match event.key() {
                Key::Delete => {
                    points.write().swap_remove(i);
                    arcs.write().retain(|&(p1, p2)| p1 != i && p2 != i);
                    arcs.write().iter_mut().for_each(|(p1, p2)| {
                        if *p1 == points.len() {
                            *p1 = i;
                        }
                        if *p2 == points.len() {
                            *p2 = i;
                        }
                    });
                    selected_point.set(None);
                }
                Key::Escape => {
                    selected_point.set(None);
                }
                Key::Character(c) => {
                    name.push_str(&c);
                    points.write()[i].1 = name.clone();
                    selected_point.set(Some((i, p, p_rot, name)));
                }
                Key::Backspace => {
                    name.pop();
                    points.write()[i].1 = name.clone();
                    selected_point.set(Some((i, p, p_rot, name)));
                }
                _ => {}
            }
        }
    };

    rsx! {
        div {
            id: "sphere",
            onkeydown: key_event,
            onmousemove: mouse_move,
            onmouseup: mouse_up,
            tabindex: "0",
            style: "outline: none;",
            div {
                oncontextmenu: move |event| {
                    event.prevent_default();
                },
                onmousedown: move |event| {
                    match event.trigger_button() {
                        Some(MouseButton::Primary) => primary_click(event),
                        Some(MouseButton::Secondary) => secondary_click(event),
                        _ => {}
                    }
                },
                svg { width: "95vw", height: "95vh", view_box: "0 0 100 100",

                    circle {
                        cx: "50",
                        cy: "50",
                        r: "25",
                        stroke: "white",
                        stroke_width: "0.5",
                        fill: "rgba(0, 0, 0, 0.4)",
                    }

                    ArcDrawer { arcs, rotation, points }

                    for (i , x , y , _ , r , opacity , name) in points
                        .read()
                        .iter()
                        .enumerate()
                        .map(|(i, &(point, ref name))| {
                            let q = rotation().1;
                            let (x, y, z) = q.rotate_point_active(point);
                            let opacity = if z > 0.0 { 1.0 } else { 0.4 };
                            let mut r = 0.6;
                            if let Some((j, _, _, _)) = selected_point() {
                                if i == j {
                                    r = 1.0;
                                }
                            }
                            (i, x * 25.0 + 50.0, y * 25.0 + 50.0, z, r, opacity, name)
                        })
                    {
                        circle {
                            key: "{i}",
                            cx: "{x}",
                            cy: "{y}",
                            r: "{r}",
                            fill: "rgba(255, 0, 0, {opacity})",
                        }
                        text {
                            key: "text-{i}",
                            x: "{x}",
                            y: "{y - 2.0}",
                            fill: "rgba(255, 255, 255, {opacity})",
                            font_family: "Arial",
                            font_size: "2",
                            text_anchor: "middle",
                            style: "font-weight: bold; user-select: none;",
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ArcDrawer(
    arcs: Signal<Vec<(usize, usize)>>,
    rotation: Signal<(Vec3, Quaternion)>,
    points: Signal<Vec<(Vec3, String)>>,
) -> Element {
    // Function to calculate the great circle arc points
    let calculate_great_circle_arc = |p1: Vec3, p2: Vec3| -> Vec<Vec3> {
        let mut arc_points = Vec::new();
        let steps = 100; // Number of points along the arc
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = (1.0 - t) * p1.0 + t * p2.0;
            let y = (1.0 - t) * p1.1 + t * p2.1;
            let z = (1.0 - t) * p1.2 + t * p2.2;
            let length = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
            arc_points.push((x / length, y / length, z / length));
        }
        arc_points
    };

    rsx! {
        for (front_path_data , back_path_data) in arcs.read()
            .iter()
            .map(|&(p1_idx, p2_idx)| {
                let p1 = points.read()[p1_idx].0;
                let p2 = points.read()[p2_idx].0;
                let arc_points = calculate_great_circle_arc(p1, p2);
                let mut front_path = Vec::new();
                let mut back_path = Vec::new();
                for (x, y, z) in arc_points {
                    let rotated = rotation().1.rotate_point_active((x, y, z));
                    let svg_x = rotated.0 * 25.0 + 50.0;
                    let svg_y = rotated.1 * 25.0 + 50.0;
                    if rotated.2 >= 0.0 {
                        front_path.push(format!("{},{}", svg_x, svg_y));
                    } else {
                        back_path.push(format!("{},{}", svg_x, svg_y));
                    }
                }
                let mut front_path_data = String::new();
                let mut back_path_data = String::new();
                if !front_path.is_empty() {
                    front_path_data = "M ".to_string() + &front_path.join("L ");
                }
                if !back_path.is_empty() {
                    back_path_data = "M ".to_string() + &back_path.join("L ");
                }
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

fn transform_viewport_to_sphere(viewport_x: f64, viewport_y: f64) -> Vec3 {
    let nan = (f64::NAN, f64::NAN, f64::NAN);
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
        return (circle_x, circle_y, (1.0 - r2).sqrt());
    }

    nan
}

#[component]
fn CoordinatesBox(selected_point: Signal<Option<(usize, Vec3, Vec3, String)>>) -> Element {
    let Some((_, (x, y, z), (rx, ry, rz), name)) = selected_point() else {
        return rsx! {};
    };
    let x = format!("{:.2}", x);
    let y = format!("{:.2}", y);
    let z = format!("{:.2}", z);
    let rx = format!("{:.2}", rx);
    let ry = format!("{:.2}", ry);
    let rz = format!("{:.2}", rz);
    rsx! {
        div { style: "
                position: absolute;
                top: 20px;
                right: 20px;
                padding: 8px;
                background-color: rgba(0, 0, 0, 0.6);
                color: white;
                border-radius: 4px;
                font-family: monospace;
                z-index: 100;
            ",
            "Absolute Coordinates: x={x}, y={y}, z={z}"
            br {}
            "Rotated Frame Coordinates: x={rx}, y={ry}, z={rz}"
            br {}
            "Name: {name}"
        }
    }
}

#[component]
fn SlidersPanel(
    rotation: Signal<(Vec3, Quaternion)>,
    selected_point: Signal<Option<(usize, Vec3, Vec3, String)>>,
) -> Element {
    rsx! {
        div { style: "
                position: absolute;
                top: 20px;
                left: 20px;
                display: flex;
                flex-direction: column;
                gap: 10px;
                background-color: rgba(0, 0, 0, 0.6);
                padding: 10px;
                border-radius: 4px;
                color: white;
                font-family: monospace;
            ",
            div { style: "display: flex; align-items: center; gap: 10px;",
                span { "X rotation: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    value: "{rotation().0.0}",
                    oninput: move |evt| {
                        rotation.write().0.0 = evt.value().parse::<f64>().unwrap_or(0.0);
                        rotation.write().1 = Quaternion::from_euler_angles(
                            rotation().0.0.to_radians(),
                            rotation().0.1.to_radians(),
                            rotation().0.2.to_radians(),
                        );
                        if let Some((i, point, _, name)) = selected_point() {
                            let rotated = rotation().1.rotate_point_active(point);
                            selected_point.set(Some((i, point, rotated, name)));
                        }
                    },
                }
                span { "{rotation().0.0}°" }
            }
            div { style: "display: flex; align-items: center; gap: 10px;",
                span { "Y rotation: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    value: "{rotation().0.1}",
                    oninput: move |evt| {
                        rotation.write().0.1 = evt.value().parse::<f64>().unwrap_or(0.0);
                        rotation.write().1 = Quaternion::from_euler_angles(
                            rotation().0.0.to_radians(),
                            rotation().0.1.to_radians(),
                            rotation().0.2.to_radians(),
                        );
                        if let Some((i, point, _, name)) = selected_point() {
                            let rotated = rotation().1.rotate_point_active(point);
                            selected_point.set(Some((i, point, rotated, name)));
                        }
                    },
                }
                span { "{rotation().0.1}°" }
            }
            div { style: "display: flex; align-items: center; gap: 10px;",
                span { "Z rotation: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    value: "{rotation().0.2}",
                    oninput: move |evt| {
                        rotation.write().0.2 = evt.value().parse::<f64>().unwrap_or(0.0);
                        rotation.write().1 = Quaternion::from_euler_angles(
                            rotation().0.0.to_radians(),
                            rotation().0.1.to_radians(),
                            rotation().0.2.to_radians(),
                        );
                        if let Some((i, point, _, name)) = selected_point() {
                            let rotated = rotation().1.rotate_point_active(point);
                            selected_point.set(Some((i, point, rotated, name)));
                        }
                    },
                }
                span { "{rotation().0.2}°" }
            }
        }
    }
}
