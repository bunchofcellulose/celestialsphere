use dioxus::{html::input_data::MouseButton, prelude::*};

mod panels;
mod utils;
use panels::*;
use utils::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let points = use_signal(Vec::<Point>::new); // points on the sphere
    let arcs = use_signal(Vec::<(usize, usize)>::new); // id of the two points at the end of arc
    let great_circles = use_signal(Vec::<usize>::new); // Stores indices of points that are poles of great circles
    let selected_point = use_signal(|| None::<usize>); // id of selected point
    let scale = use_signal(|| (1.0, [0.0, 0.0, 0.0], Quaternion::identity())); // zoom, rotation vec3, quaternion
    let active_mode = use_signal(|| Mode::Selection); // current mode of the application

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        SelectionBox { points, selected_point }
        SlidersPanel { points, scale }
        ModePanel { active_mode }
        SubModePanel { active_mode, points }
        FilePanel {
            points,
            arcs,
            great_circles,
            scale,
        }
        Sphere {
            points,
            arcs,
            great_circles,
            selected_point,
            scale,
            active_mode,
        }
    }
}

#[component]
pub fn Sphere(
    points: Signal<Vec<Point>>,
    arcs: Signal<Vec<(usize, usize)>>,
    great_circles: Signal<Vec<usize>>,
    selected_point: Signal<Option<usize>>,
    scale: Signal<(f64, Vec3, Quaternion)>,
    active_mode: Signal<Mode>,
) -> Element {
    let mut dragged_point = use_signal(|| None::<usize>); // id of the point being dragged

    let select_point = move |x: f64, y: f64| {
        let [px, py, pz] = transform_viewport_to_sphere(x, y);
        if pz.is_nan() {
            return Selected::None;
        }
        for p in points.read().iter() {
            let [x, y, z] = p.rotated;
            if z < 0.0 {
                continue;
            }
            let dx = px - x;
            let dy = py - y;
            if dx.powi(2) + dy.powi(2) <= 0.002 {
                return Selected::Existing(p.id);
            }
        }
        Selected::New(Point::from_vec3_rotated(
            points.len(),
            [px, py, pz],
            scale().2,
        ))
    };

    let mut primary_click_selection = move |event: Event<MouseData>| match select_point(
        event.client_coordinates().x,
        event.client_coordinates().y,
    ) {
        Selected::None => (),
        Selected::New(point) => {
            points.write().push(point);
            selected_point.set(Some(points.len() - 1));
        }
        Selected::Existing(selected) => {
            if Some(selected) == selected_point() {
                selected_point.set(None);
                dragged_point.set(None);
            } else {
                selected_point.set(Some(selected));
                if points.read()[selected].movable {
                    dragged_point.set(Some(selected));
                }
            }
        }
    };

    let mut secondary_click_selection = move |event: Event<MouseData>| {
        let Some(selected) = selected_point() else {
            return;
        };

        match select_point(event.client_coordinates().x, event.client_coordinates().y) {
            Selected::None => (),
            Selected::New(_) => {}
            Selected::Existing(p) => {
                if p == selected {
                    return;
                }
                arcs.write().push((p, selected));
            }
        }
    };

    let mut mouse_move_selection = move |event: Event<MouseData>| {
        if let Some(dragged_idx) = dragged_point() {
            let viewport_x = event.client_coordinates().x;
            let viewport_y = event.client_coordinates().y;
            let [px, py, pz] = transform_viewport_to_sphere(viewport_x, viewport_y);
            if pz.is_nan() {
                return;
            }
            points.write()[dragged_idx].move_to([px, py, pz], scale().2);
            if Some(dragged_idx) != selected_point() {
                selected_point.set(Some(dragged_idx));
            }
        }
    };

    let mut mouse_up_selection = move |_event: Event<MouseData>| {
        dragged_point.set(None);
    };

    let mut key_event_selection = move |event: Event<KeyboardData>| {
        event.prevent_default();
        if let Some(i) = selected_point() {
            match event.key() {
                Key::Delete => {
                    if !points.read()[i].removable {
                        return;
                    }
                    points.write().swap_remove(i);
                    if let Some(p) = points.write().get_mut(i) {
                        p.id = i;
                    }
                    arcs.write().retain(|&(p1, p2)| p1 != i && p2 != i);
                    great_circles.write().retain(|&x| x != i);
                    arcs.write().iter_mut().for_each(|(p1, p2)| {
                        if *p1 == points.len() {
                            *p1 = i;
                        }
                        if *p2 == points.len() {
                            *p2 = i;
                        }
                    });
                    great_circles.write().iter_mut().for_each(|x| {
                        if *x == points.len() {
                            *x = i;
                        }
                    });
                    selected_point.set(None);
                }
                Key::Escape => {
                    selected_point.set(None);
                }
                Key::Character(ref c) if c.as_str() == "." => {
                    if !great_circles.read().contains(&i) {
                        great_circles.write().push(i);
                    } else {
                        great_circles.write().retain(|&x| x != i);
                    }
                }
                Key::Character(c) => {
                    points.write()[i].name.push_str(&c);
                }
                Key::Backspace => {
                    points.write()[i].name.pop();
                }
                _ => {}
            }
        }
    };

    rsx! {
        div {
            id: "sphere",
            onkeydown: move |event| {
                match active_mode() {
                    Mode::Selection => key_event_selection(event),
                    Mode::Triangle => {}
                }
            },
            onmousemove: move |event| {
                match active_mode() {
                    Mode::Selection => mouse_move_selection(event),
                    Mode::Triangle => {}
                }
            },
            onmouseup: move |event| {
                match active_mode() {
                    Mode::Selection => mouse_up_selection(event),
                    Mode::Triangle => {}
                }
            },
            tabindex: "0",
            style: "outline: none;",
            div {
                oncontextmenu: move |event| {
                    event.prevent_default();
                },
                onmousedown: move |event| {
                    match active_mode() {
                        Mode::Selection => {
                            match event.trigger_button() {
                                Some(MouseButton::Primary) => primary_click_selection(event),
                                Some(MouseButton::Secondary) => secondary_click_selection(event),
                                _ => {}
                            }
                        }
                        Mode::Triangle => {}
                    }
                },
                svg {
                    width: "95vw",
                    height: "95vh",
                    view_box: "{50.0 - 50.0 / scale().0} {50.0 - 50.0 / scale().0} {100.0 / scale().0} {100.0 / scale().0}",
                    circle {
                        cx: "50",
                        cy: "50",
                        r: "25",
                        stroke: "white",
                        stroke_width: "0.5",
                        fill: "rgba(0, 0, 0, 0.4)",
                    }

                    GreatCircleDrawer { great_circles, points }
                    ArcDrawer { arcs, points }

                    for (i , x , y , _ , r , opacity , name) in points
                        .read()
                        .iter()
                        .map(|point| {
                            let [x, y, z] = point.rotated;
                            let opacity = if z > 0.0 { 1.0 } else { 0.4 };
                            let r = if Some(point.id) == selected_point() { 1.0 } else { 0.6 };
                            (point.id, x * 25.0 + 50.0, y * 25.0 + 50.0, z, r, opacity, &point.name)
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
fn ArcDrawer(arcs: Signal<Vec<(usize, usize)>>, points: Signal<Vec<Point>>) -> Element {
    // Function to calculate the great circle arc points
    let calculate_great_circle_arc = |p1: Vec3, p2: Vec3| -> Vec<Vec3> {
        let mut arc_points = Vec::new();
        let steps = 200; // Number of points along the arc
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = (1.0 - t) * p1[0] + t * p2[0];
            let y = (1.0 - t) * p1[1] + t * p2[1];
            let z = (1.0 - t) * p1[2] + t * p2[2];
            let length = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
            arc_points.push([x / length, y / length, z / length]);
        }
        arc_points
    };

    rsx! {
        for (front_path_data , back_path_data) in arcs.read()
            .iter()
            .map(|&(p1_idx, p2_idx)| {
                let p1 = points.read()[p1_idx].rotated;
                let p2 = points.read()[p2_idx].rotated;
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
fn GreatCircleDrawer(great_circles: Signal<Vec<usize>>, points: Signal<Vec<Point>>) -> Element {
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
        for (front_path_data , back_path_data) in great_circles
            .read()
            .iter()
            .map(|&pole_idx| {
                let pole = points.read()[pole_idx].rotated;
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
