use crate::*;

#[component]
pub fn SelectionBox(points: Signal<Vec<Point>>, state: Signal<State>) -> Element {
    rsx! {
        div { class: "right-info-boxes-container",
            for Point { id , absolute : [x , y , z] , name , movable , removable , abs_polar : [theta , phi] , rotated : [rx , ry , rz] , rot_polar : [rtheta , rphi] } in state.read().selected().iter().map(|&i| points()[i].clone()) {
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
                                match event.value().as_str() {
                                    "true" => points.write()[id].movable = true,
                                    "false" => points.write()[id].movable = false,
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
                                    "true" => points.write()[id].removable = true,
                                    "false" => points.write()[id].removable = false,
                                    a => panic!("{a}"),
                                }
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
pub fn LeftPanel(
    state: Signal<State>,
    points: Signal<Vec<Point>>,
    great_circles: Signal<Vec<GreatCircle>>,
) -> Element {
    rsx! {
        div { class: "left-info-boxes-container",
            {
                if let &[pole] = state.read().selected() {
                    if let Some(&GreatCircle { ref name, divisions, .. }) = great_circles()
                        .iter()
                        .find(|gc| gc.pole == pole)
                    {
                        rsx! {
                            div { class: "info-box",
                                h3 { "Great Circle" }
                                "Pole ID: {pole}"
                                br {}
                                "Name: {name}"
                                br {}
                                div {
                                    input {
                                        r#type: "checkbox",
                                        checked: "{divisions}",
                                        onchange: move |event| {
                                            match event.value().as_str() {
                                                "true" => great_circles.write()[pole].divisions = true,
                                                "false" => great_circles.write()[pole].divisions = false,
                                                a => panic!("{a}"),
                                            }
                                        },
                                    }
                                    span { "Divisions" }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                } else if let &[a, b, c] = state.read().selected() {
                    let a_pos = points()[a].absolute;
                    let b_pos = points()[b].absolute;
                    let c_pos = points()[c].absolute;
                    let side_a = arc_distance(b_pos, c_pos);
                    let side_b = arc_distance(a_pos, c_pos);
                    let side_c = arc_distance(a_pos, b_pos);
                    let [angle_a, angle_b, angle_c] = calculate_angle(side_a, side_b, side_c);
                    let side_a = side_a.to_degrees();
                    let side_b = side_b.to_degrees();
                    let side_c = side_c.to_degrees();
                    let angle_a = angle_a.to_degrees();
                    let angle_b = angle_b.to_degrees();
                    let angle_c = angle_c.to_degrees();
                    let area = angle_a + angle_b + angle_c - 180.0;
                    rsx! {
                        div { class: "info-box",
                            h3 { "Spherical Triangle" }
                            "Side a: {side_a:.4}°"
                            br {}
                            "Side b: {side_b:.4}°"
                            br {}
                            "Side c: {side_c:.4}°"
                            br {}
                            br {}
                            "Angle A: {angle_a:.4}°"
                            br {}
                            "Angle B: {angle_b:.4}°"
                            br {}
                            "Angle C: {angle_c:.4}°"
                            br {}
                            br {}
                            "Spherical Excess: {area:.4}°"
                        }
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}

use dioxus::web::WebEventExt;
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsCast;
use web_sys::{window, HtmlInputElement, Url};

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
    great_circles: Signal<Vec<GreatCircle>>,
    scale: Signal<(f64, Vec3, Quaternion)>,
    state: Signal<State>,
) -> Element {
    let save_to_file = move || {
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
            great_circles: great_circles().iter().map(|gc| gc.pole).collect(),
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

            todo!("{input:?}");
        }
    };

    let mut new_file = move || {
        points.set(Vec::new());
        arcs.set(Vec::new());
        great_circles.set(Vec::new());
        scale.set((1.0, [0.0, 0.0, 0.0], Quaternion::identity()));
        state.write().clear_selection();
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
