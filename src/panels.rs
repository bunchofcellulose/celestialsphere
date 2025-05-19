use crate::*;
use dioxus::web::WebEventExt;
use serde::{Deserialize, Serialize};
use web_sys::{wasm_bindgen::{JsCast, closure}, window, HtmlInputElement, Url};

#[component]
pub fn SelectionBox(points: Signal<Vec<Point>>, state: Signal<State>) -> Element {
    rsx! {
        div { class: "right-info-boxes-container",
            for Point { id , absolute : [x , y , z] , name , movable , removable , abs_polar : [theta , phi] , rotated : [rx , ry , rz] , rot_polar : [rtheta , rphi] } in state.read().selected().iter().map(|&id| points.read()[id].clone()) {
                div { class: "info-box",
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
                    "ID: {id}"
                    br {}
                    "Name: {name}"
                    br {}
                    div {
                        input {
                            r#type: "checkbox",
                            checked: "{movable}",
                            onchange: move |event| {
                                points.write()[id].movable = event.value() == "true";
                            },
                        }
                        span { "Movable" }
                    }
                    div {
                        input {
                            r#type: "checkbox",
                            checked: "{removable}",
                            onchange: move |event| {
                                points.write()[id].removable = event.value() == "true";
                            },
                        }
                        span { "Removable" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn GitHubIcon() -> Element {
    rsx! {
        a {
            href: "https://github.com/Bunch-of-cells/celestialsphere",
            target: "_blank",
            rel: "noopener noreferrer",
            class: "github-icon",
            title: "View on GitHub",
            svg {
                width: "40",
                height: "40",
                view_box: "0 0 16 16",
                fill: "white",
                path { d: "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" }
            }
        }
    }
}

#[component]
pub fn SlidersPanel(
    points: Signal<Vec<Point>>,
    scale: Signal<(f64, Vec3, Quaternion)>,
    show_grid: Signal<bool>,
) -> Element {
    let mut change = move || {
        let q = Quaternion::from_euler_deg(scale().1);
        scale.write().2 = q;
        for p in points.write().iter_mut() {
            p.rotate(q);
        }
    };
    rsx! {
        div { class: "sliders-panel",
            div {
                span { "X rotation: " }
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
                span { "{scale().1[0]:.1}°" }
            }
            div {
                span { "Y rotation: " }
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
                span { "{scale().1[1]:.1}°" }
            }
            div {
                span { "Z rotation: " }
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
                span { "{scale().1[2]:.1}°" }
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
            div { class: "checkbox-control",
                input {
                    r#type: "checkbox",
                    id: "grid-toggle",
                    checked: "{show_grid()}",
                    onchange: move |evt| {
                        show_grid.set(evt.value() == "true");
                    },
                }
                label { r#for: "grid-toggle", "Show coordinate grid" }
            }
        }
    }
}

#[component]
pub fn LeftPanel(
    state: Signal<State>,
    points: Signal<Vec<Point>>,
    great_circles: Signal<Vec<GreatCircle>>,
    small_circles: Signal<Vec<SmallCircle>>,
) -> Element {
    rsx! {
        div { class: "left-info-boxes-container",
            if let &[pole] = state.read().selected() {
                if let Some(gc) = great_circles().iter().find(|gc| gc.pole == pole) {
                    div { class: "info-box",
                        h3 { "Great Circle" }
                        "Pole ID: {pole}"
                        br {}
                        "Name: {gc.name}"
                    }
                }
                if let Some(sc) = small_circles.read().iter().find(|sc| sc.pole == pole) {
                    div { class: "info-box",
                        h3 { "Small Circle" }
                        "Pole ID: {pole}"
                        br {}
                        "Name: {sc.name}"
                        br {}
                        "Plane Distance: {sc.plane_distance:.4}"
                        br {}
                        "Radius: {(1.0 - sc.plane_distance.powi(2)).sqrt():.2}"
                    }
                }
            }
            if let Some([a, b, c, aa, ab, ac, e]) = 'block: {
                let &[a, b, c] = state.read().selected() else { break 'block None };
                let a_pos = points()[a].absolute;
                let b_pos = points()[b].absolute;
                let c_pos = points()[c].absolute;
                let side_a = arc_distance(b_pos, c_pos);
                let side_b = arc_distance(a_pos, c_pos);
                let side_c = arc_distance(a_pos, b_pos);
                let [angle_a, angle_b, angle_c] = calculate_angle(side_a, side_b, side_c);
                let a = side_a.to_degrees();
                let b = side_b.to_degrees();
                let c = side_c.to_degrees();
                let aa = angle_a.to_degrees();
                let ab = angle_b.to_degrees();
                let ac = angle_c.to_degrees();
                let e = aa + ab + ac - 180.0;
                Some([a, b, c, aa, ab, ac, e])
            }
            {
                div { class: "info-box",
                    h3 { "Spherical Triangle" }
                    "Side a: {a:.4}°"
                    br {}
                    "Side b: {b:.4}°"
                    br {}
                    "Side c: {c:.4}°"
                    br {}
                    br {}
                    "Angle A: {aa:.4}°"
                    br {}
                    "Angle B: {ab:.4}°"
                    br {}
                    "Angle C: {ac:.4}°"
                    br {}
                    br {}
                    "Spherical Excess: {e:.4}°"
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    points: Vec<(Vec3, String, bool, bool)>,
    arcs: Vec<(usize, usize)>,
    great_circles: Vec<(usize, String)>,
    small_circles: Vec<(usize, f64, String)>,
}

pub fn save_to_file(
    points: Signal<Vec<Point>>,
    arcs: Signal<Vec<(usize, usize)>>,
    great_circles: Signal<Vec<GreatCircle>>,
    small_circles: Signal<Vec<SmallCircle>>,
) {
    let save_data = SaveData {
        points: points()
            .iter()
            .map(|point| {
                (
                    point.absolute,
                    point.name.clone(),
                    point.movable,
                    point.removable,
                )
            })
            .collect(),
        arcs: arcs().clone(),
        great_circles: great_circles()
            .iter()
            .map(|gc| (gc.pole, gc.name.clone()))
            .collect(),
        small_circles: small_circles
            .read()
            .iter()
            .map(|sc| (sc.pole, sc.plane_distance, sc.name.clone()))
            .collect(),
    };

    if let Ok(json) = serde_json::to_string_pretty(&save_data) {
        let blob = web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&json.into())).unwrap();
        let url = Url::create_object_url_with_blob(&blob).unwrap();

        let document = window().unwrap().document().unwrap();
        let a = document.create_element("a").unwrap();
        a.set_attribute("href", &url).unwrap();
        a.set_attribute("download", "celestial_data.json").unwrap();
        a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
        Url::revoke_object_url(&url).unwrap();
    }
}

pub fn new_file(
    mut points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    mut great_circles: Signal<Vec<GreatCircle>>,
    mut small_circles: Signal<Vec<SmallCircle>>,
    mut scale: Signal<(f64, Vec3, Quaternion)>,
    mut state: Signal<State>,
) {
    points.write().clear();
    arcs.write().clear();
    great_circles.write().clear();
    small_circles.write().clear();

    scale.write().0 = 1.0;
    scale.write().1 = [0.0, 0.0, 0.0];
    scale.write().2 = Quaternion::identity();

    state.write().clear_selection();
}

#[component]
pub fn FilePanel(
    mut points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    mut great_circles: Signal<Vec<GreatCircle>>,
    mut small_circles: Signal<Vec<SmallCircle>>,
    mut scale: Signal<(f64, Vec3, Quaternion)>,
    mut state: Signal<State>,
) -> Element {
    let load_from_file = {
        move |event: web_sys::Event| {
            let input = event
                .target()
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap();

            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let file_reader = web_sys::FileReader::new().unwrap();
                    let fr_c = file_reader.clone();

                    let onloadend = closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                        let result = fr_c.result().unwrap();
                        let text = result.as_string().unwrap();
                        match serde_json::from_str::<SaveData>(&text) {
                            Ok(data) => {
                                // Restore points
                                let mut pts = Vec::new();
                                for (i, (vec, name, movable, removable)) in data.points.into_iter().enumerate() {
                                    let mut p = Point::from_vec3(i, vec);
                                    p.name = name;
                                    p.movable = movable;
                                    p.removable = removable;
                                    pts.push(p);
                                }
                                points.set(pts);
                                arcs.set(data.arcs);
                                let gcs = data.great_circles
                                    .into_iter()
                                    .map(|(pole, name)| {
                                        let mut gc = GreatCircle::new(pole);
                                        gc.name = name;
                                        gc
                                    })
                                    .collect();
                                great_circles.set(gcs);
                                let scs = data.small_circles
                                    .into_iter()
                                    .map(|(pole, plane_distance, name)| {
                                        let mut sc = SmallCircle::new(pole, plane_distance);
                                        sc.name = name;
                                        sc
                                    })
                                    .collect();
                                small_circles.set(scs);
                                scale.set((1.0, [0.0, 0.0, 0.0], Quaternion::identity()));
                                state.write().clear_selection();
                            }
                            Err(e) => {
                                web_sys::console::error_1(&format!("Failed to parse file: {e}").into());
                            }
                        }
                    }) as Box<dyn FnMut(_)>);

                    file_reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
                    file_reader.read_as_text(&file).unwrap();
                    onloadend.forget();
                }
            }
        }
    };

    rsx! {
        div { class: "file-panel",
            button {
                onclick: move |_| save_to_file(points, arcs, great_circles, small_circles),
                style: "background-image: url({SAVE});",
            }
            label {
                class: "file-load-label",
                style: "background-image: url({LOAD}); cursor: pointer; display: inline-block;",
                input {
                    id: "file-upload",
                    r#type: "file",
                    accept: ".json",
                    onchange: move |event| {
                        load_from_file(event.data().as_web_event());
                    },
                }
            }
            button {
                onclick: move |_| new_file(points, arcs, great_circles, small_circles, scale, state),
                style: "background-image: url({NEW_FILE});",
            }
        }
    }
}
