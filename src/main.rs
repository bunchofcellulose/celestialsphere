use dioxus::{html::input_data::MouseButton, prelude::*};

mod circles;
mod panels;
mod utils;
use circles::*;
use panels::*;
use utils::*;

const FAVICON: Asset = asset!("/assets/triangle.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let points = use_signal(Vec::<Point>::new); // points on the sphere
    let arcs = use_signal(Vec::<(usize, usize)>::new); // id of the two points at the end of arc
    let great_circles = use_signal(Vec::<GreatCircle>::new); // Stores indices of points that are poles of great circles
    let scale = use_signal(|| (1.0, [0.0, 0.0, 0.0], Quaternion::identity())); // zoom, rotation vec3, quaternion
    let state = use_signal(State::initialize); // state of the application
    let show_grid = use_signal(|| false); // show/hide grid

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        SelectionBox { points, state }
        SlidersPanel { points, scale, show_grid }
        LeftPanel { state, points, great_circles }
        GitHubIcon {}
        FilePanel {
            points,
            arcs,
            great_circles,
            scale,
            state,
        }
        Sphere {
            points,
            arcs,
            great_circles,
            state,
            scale,
            show_grid,
        }
    }
}

#[component]
pub fn Sphere(
    points: Signal<Vec<Point>>,
    arcs: Signal<Vec<(usize, usize)>>,
    great_circles: Signal<Vec<GreatCircle>>,
    state: Signal<State>,
    scale: Signal<(f64, Vec3, Quaternion)>,
    show_grid: Signal<bool>,
) -> Element {
    let mut dragged_point = use_signal(|| None::<usize>); // id of the point being dragged
    let mut is_rotating = use_signal(|| false);
    let mut last_rotation_pos = use_signal(|| (0.0, 0.0));

    let select_point = move |x: f64, y: f64| {
        let [px, py, pz] = transform_viewport_to_sphere(x, y);
        if pz.is_nan() {
            return Selected::None;
        }
        for p in points().iter() {
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

    let mut primary_click = move |event: Event<MouseData>| {
        let multi = event.modifiers().shift();
        match select_point(event.client_coordinates().x, event.client_coordinates().y) {
            Selected::None => (),
            Selected::New(mut point) => {
                if event.modifiers().shift() {
                    let threshold = 0.05; // About 3 degrees in radians
                    let snapped =
                        snap_to_great_circle(point.rotated, &great_circles(), &points(), threshold);

                    // Update with snapped coordinates
                    point = Point::from_vec3_rotated(points.len(), snapped, scale().2);
                }
                points.write().push(point);
                state.write().toggle_select(multi, points.len() - 1);
            }
            Selected::Existing(selected) => {
                if state.write().toggle_select(multi, selected) && points()[selected].movable {
                    dragged_point.set(Some(selected));
                }
            }
        }
    };

    let mut secondary_click = move |event: Event<MouseData>| {
        if state.read().selected().is_empty() {
            return;
        }

        match select_point(event.client_coordinates().x, event.client_coordinates().y) {
            Selected::None => (),
            Selected::New(_) => {}
            Selected::Existing(p) => {
                for &selected in state.read().selected() {
                    if p == selected {
                        continue;
                    }
                    if arcs().contains(&(selected, p)) {
                        arcs.write().retain(|&(p1, p2)| p1 != selected || p2 != p);
                    } else if arcs().contains(&(p, selected)) {
                        arcs.write().retain(|&(p1, p2)| p1 != p || p2 != selected);
                    } else {
                        arcs.write().push((selected, p));
                    }
                }
            }
        }
    };

    let mut middle_click = move |event: Event<MouseData>| {
        // Start rotation when middle button is pressed
        is_rotating.set(true);

        // Store initial position for rotation calculation
        last_rotation_pos.set((event.client_coordinates().x, event.client_coordinates().y));

        // Prevent default browser middle-click behavior
        event.prevent_default();
    };

    let scroll = move |event: Event<WheelData>| {
        // Get scroll delta (negative for scroll up/zoom in, positive for scroll down/zoom out)
        let delta = event.delta().strip_units().y;

        // Calculate new scale with smooth zooming behavior
        // The factor 0.001 controls zoom sensitivity
        let zoom_factor = 1.0 - delta * 0.001;
        let mut new_scale = scale().0 * zoom_factor;

        // Constrain scale to reasonable limits
        new_scale = new_scale.clamp(0.5, 2.0);

        // Update scale
        scale.write().0 = new_scale;
    };

    let mouse_move = move |event: Event<MouseData>| {
        // Handle point dragging (your existing code)
        if let Some(dragged_idx) = dragged_point() {
            let viewport_x = event.client_coordinates().x;
            let viewport_y = event.client_coordinates().y;
            let [px, py, pz] = transform_viewport_to_sphere(viewport_x, viewport_y);
            if pz.is_nan() {
                return;
            }

            // Apply snapping during drag if Shift is held
            if event.modifiers().shift() {
                let threshold = 0.05; // About 7 degrees
                let snapped =
                    snap_to_great_circle([px, py, pz], &great_circles(), &points(), threshold);
                points.write()[dragged_idx].move_to(snapped, scale().2);
            } else {
                points.write()[dragged_idx].move_to([px, py, pz], scale().2);
            }

            state.write().select(dragged_idx);
        }

        // Handle sphere rotation
        if is_rotating() {
            let current_x = event.client_coordinates().x;
            let current_y = event.client_coordinates().y;
            let (last_x, last_y) = last_rotation_pos();

            // Calculate rotation angles based on mouse movement
            // Adjust sensitivity as needed
            let sensitivity = 0.005;
            let delta_x = (current_x - last_x) * sensitivity;
            let delta_y = -(current_y - last_y) * sensitivity;

            // Create rotation quaternions for both axes
            let rotation_y = Quaternion::from_axis_angle([1.0, 0.0, 0.0], delta_y);
            let rotation_x = Quaternion::from_axis_angle([0.0, 1.0, 0.0], delta_x);

            // Combine rotations with existing rotation
            let new_rotation = rotation_y.multiply(rotation_x).multiply(scale().2);

            // Update scale with new rotation quaternion
            scale.write().2 = new_rotation;
            scale.write().1 = new_rotation.to_euler_deg();

            // Update last position
            last_rotation_pos.set((current_x, current_y));

            // Apply rotation to all points
            for point in points.write().iter_mut() {
                point.rotate(new_rotation);
            }
        }
    };

    let mouse_up = move |_: Event<MouseData>| {
        // Stop dragging point
        dragged_point.set(None);

        // Stop rotation
        is_rotating.set(false);
    };

    let key_event = move |event: Event<KeyboardData>| {
        event.prevent_default();

        let mut s = state.write();
        for i in s.selected().iter().rev().copied().collect::<Vec<_>>() {
            match event.key() {
                Key::Delete => {
                    if !points()[i].removable {
                        return;
                    }
                    points.write().swap_remove(i);
                    if let Some(p) = points.write().get_mut(i) {
                        p.id = i;
                    }
                    arcs.write().retain(|&(p1, p2)| p1 != i && p2 != i);
                    great_circles.write().retain(|x| x.pole != i);
                    arcs.write().iter_mut().for_each(|(p1, p2)| {
                        if *p1 == points.len() {
                            *p1 = i;
                        }
                        if *p2 == points.len() {
                            *p2 = i;
                        }
                    });
                    great_circles.write().iter_mut().for_each(|x| {
                        if x.pole == points.len() {
                            x.pole = i;
                        }
                    });
                    s.pop_selected();
                }
                Key::Escape => {
                    s.clear_selection();
                }
                Key::Character(ref c) if c.as_str() == "." => {
                    if great_circles().iter().all(|x| x.pole != i) {
                        great_circles.write().push(GreatCircle::new(i));
                    } else {
                        great_circles.write().retain(|x| x.pole != i);
                    }
                }
                Key::Character(ref c) if c.as_str() == "/" => {
                    let new = points()[i].new_inverted(points().len());
                    points.write().push(new);
                }
                Key::Character(c) => {
                    if let Some(gc) = great_circles.write().iter_mut().find(|x| x.pole == i) {
                        if event.modifiers().shift() {
                            gc.name.push_str(&toggle_case(&c));
                            break;
                        }
                    }
                    points.write()[i].name.push_str(&c);
                }
                Key::Backspace => {
                    if let Some(gc) = great_circles.write().iter_mut().find(|x| x.pole == i) {
                        if event.modifiers().shift() {
                            gc.name.pop();
                            break;
                        }
                    }
                    points.write()[i].name.pop();
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
                        Some(MouseButton::Auxiliary) => middle_click(event),
                        _ => {}
                    }
                },
                onwheel: scroll,
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

                    if show_grid() {
                        CoordinateGrid { scale }
                    }
                    GreatCircleDrawer { great_circles, points }
                    GreatCircleLabels { great_circles, points }
                    ArcDrawer { arcs, points }

                    for (i , x , y , _ , r , opacity , name) in points()
                        .iter()
                        .map(|point| {
                            let [x, y, z] = point.rotated;
                            let opacity = if z > 0.0 { 1.0 } else { 0.4 };
                            let r = if state.read().selected().contains(&point.id) { 1.0 } else { 0.6 };
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
