use crate::*;

#[component]
pub fn SelectionBox(points: Signal<Vec<Point>>, selected_point: Signal<Option<usize>>) -> Element {
    let Some(i) = selected_point() else {
        return rsx! {};
    };
    let [x, y, z] = points()[i].absolute;
    let [rx, ry, rz] = points()[i].rotated;
    let [theta, phi] = points()[i].abs_polar;
    let [rtheta, rphi] = points()[i].rot_polar;
    let name = &points()[i].name;
    let movable = points()[i].movable;
    let removable = points()[i].removable;

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
    if points().len() < 3 {
        return rsx! {}; // Do not display if there are fewer than 3 points
    }

    // Get the last three points
    let len = points().len();
    let p1 = points()[len - 3].absolute;
    let p2 = points()[len - 2].absolute;
    let p3 = points()[len - 1].absolute;

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
                span { "{scale().1[0]}°" }
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
                span { "{scale().1[1]}°" }
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

use dioxus::web::WebEventExt;
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, FileReader, HtmlInputElement, Url};

#[derive(Serialize, Deserialize)]
struct SaveData {
    points: Vec<(Vec3, String, bool, bool)>,
    arcs: Vec<(usize, usize)>,
    great_circles: Vec<usize>,
}

#[component]
pub fn FilePanel(
    points: Signal<Vec<Point>>,
    arcs: Signal<Vec<(usize, usize)>>,
    great_circles: Signal<Vec<usize>>,
    scale: Signal<(f64, Vec3, Quaternion)>,
) -> Element {
    let save_to_file = move || {
        let save_data = SaveData {
            points: points
                ()
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
            great_circles: great_circles().clone(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&save_data) {
            let blob =
                web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&json.into())).unwrap();
            let url = Url::create_object_url_with_blob(&blob).unwrap();

            let document = web_sys::window().unwrap().document().unwrap();
            let a = document.create_element("a").unwrap();
            a.set_attribute("href", &url).unwrap();
            a.set_attribute("download", "celestial_data.json").unwrap();
            a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
            Url::revoke_object_url(&url).unwrap();
        }
    };

    let load_from_file = {
        move |event: web_sys::Event| {
            let input = event
                .target()
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap();
    
            // if let Some(file) = input.files().and_then(|f| f.get(0)) {
            //     let reader = FileReader::new().unwrap();
    
            //     let onload = Closure::wrap(Box::new(move |event: web_sys::Event| {
            //         let reader: FileReader = event.target().unwrap().dyn_into().unwrap();
    
            //         let Ok(result) = reader.result() else {return;};
            //         let Some(text) = result.as_string() else {return;};
            //         if let Ok(SaveData {
            //             points: saved_points,
            //             arcs: saved_arcs,
            //             great_circles: saved_great_circles,
            //         }) = serde_json::from_str::<SaveData>(&text)
            //         {
            //             let new_points: Vec<Point> = saved_points
            //                 .into_iter()
            //                 .enumerate()
            //                 .map(|(id, (absolute, name, movable, removable))| {
            //                     let mut point = Point::from_vec3(id, absolute);
            //                     point.name = name;
            //                     point.movable = movable;
            //                     point.removable = removable;
            //                     point
            //                 })
            //                 .collect();

            //             // points.set(new_points);
            //             // arcs.set(saved_arcs);
            //             // great_circles.set(saved_great_circles);

            //             web_sys::console::log_1(&format!("File loaded successfully : {new_points:?} :: {saved_arcs:?} :: {saved_great_circles:?}").into());
            //         } else {
            //             web_sys::console::log_1(&"Failed to parse JSON".into());
            //         }
            //     }) as Box<dyn FnMut(_)>);
    
            //     reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            //     reader_as_text(&file).unwrap();
            //     onload.forget();
            // }
        }
    };

    let mut new_file = move || {
        points.set(Vec::new());
        arcs.set(Vec::new());
        great_circles.set(Vec::new());
        scale.set((1.0, [0.0, 0.0, 0.0], Quaternion::identity()));
        web_sys::console::log_1(&"New file created".into());
    };

    rsx! {
        div { class: "file-panel",
            button {
                onclick: move |_| save_to_file(),
                style: "background-image: url({SAVE});",
            }
            button {
                class: "file-load-label",
                style: "background-image: url({LOAD});",
                onclick: move |_| {
                    let input = window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("file-upload")
                        .unwrap()
                        .dyn_into::<HtmlInputElement>()
                        .unwrap();
                    input.click();
                },
                input {
                    id: "file-upload",
                    r#type: "file",
                    accept: ".json",
                    onchange: move |event| {
                        load_from_file(event.data().as_web_event());
                        scale.set((1.0, [0.0, 0.0, 0.0], Quaternion::identity()));
                    },
                }
            }
            button {
                onclick: move |_| new_file(),
                style: "background-image: url({NEW_FILE});",
            }
        }
    }
}
