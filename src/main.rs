use dioxus::prelude::*;
use web_sys::window;

mod quaternion;
use quaternion::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

type Vec2 = (f64, f64);
type Vec3 = (f64, f64, f64);

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let coordinates = use_signal(|| ((0.0, 0.0), (0.0, 0.0, 0.0)));
    let rotation = use_signal(|| (0.0, 0.0, 0.0));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        CoordinatesBox { coordinates }
        SlidersPanel { rotation }
        Sphere { coordinates, rotation }
    }
}

#[component]
pub fn Sphere(coordinates: Signal<(Vec2, Vec3)>, rotation: Signal<Vec3>) -> Element {
    let mut points = use_signal(|| Vec::new());

    let mut clickmouse = move |coords: Vec2| {
        let (x, y) =
            transform_viewport_to_circle(coords.0, coords.1).unwrap_or((f64::NAN, f64::NAN));

        let r2 = x.powi(2) + y.powi(2);

        let normalized_coords = (x, y, (1.0 - r2).sqrt());
        coordinates.set((coords, normalized_coords));

        let (x, y, z) = rotation();

        if r2 < 1.0 {
            points.write().push((normalized_coords, (x.to_radians(), y.to_radians(), z.to_radians())));
        }
    };

    rsx! {
        div { id: "sphere",
            div {
                onmousedown: move |event| {
                    let viewport_x = event.client_coordinates().x;
                    let viewport_y = event.client_coordinates().y;
                    clickmouse((viewport_x, viewport_y));
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

                    for (i , rotated) in points
                        .read()
                        .iter()
                        .map(|&(point, rot)| {
                            let rot1 = Quaternion::from_euler_angles(
                                rotation().0.to_radians(),
                                rotation().1.to_radians(),
                                rotation().2.to_radians(),
                            );
                            let rot2 = Quaternion::from_euler_angles(rot.0, rot.1, rot.2);
                            rot1.rotate_point_active(rot2.rotate_point_passive(point))
                        })
                        .map(|(x, y, z)| {
                            let color = if z > 0.0 {
                                "rgba(255, 0, 0, 1.0)"
                            } else {
                                "rgba(255, 0, 0, 0.4)"
                            };
                            (x, y, z, color)
                        })
                        .enumerate()
                    {
                        circle {
                            key: "{i}",
                            cx: "{rotated.0 * 25.0 + 50.0}",
                            cy: "{rotated.1 * 25.0 + 50.0}",
                            r: "0.6",
                            fill: "{rotated.3}",
                        }
                    }
                }
            }
        }
    }
}

fn transform_viewport_to_circle(viewport_x: f64, viewport_y: f64) -> Option<Vec2> {
    let document = window()?.document()?;
    let circle_element = document.query_selector("circle").ok()??;
    let rect = circle_element.get_bounding_client_rect();

    let (circle_left, circle_top, circle_width, circle_height) =
        (rect.left(), rect.top(), rect.width(), rect.height());

    let circle_x = (viewport_x - circle_left - circle_width / 2.0) / circle_width * 2.0;
    let circle_y = (viewport_y - circle_top - circle_height / 2.0) / circle_height * 2.0;

    Some((circle_x, circle_y))
}

#[component]
fn CoordinatesBox(coordinates: Signal<(Vec2, Vec3)>) -> Element {
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
            "Viewport Coordinates: x={coordinates().0.0.to_string()}, y={coordinates().0.1.to_string()}"
            br {}
            "Circle Coordinates: x={(coordinates().1.0).to_string()}, y={(coordinates().1.1).to_string()}"
        }
    }
}

#[component]
fn SlidersPanel(rotation: Signal<Vec3>) -> Element {
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
                    value: "{rotation().0}",
                    oninput: move |evt| rotation.write().0 = evt.value().parse::<f64>().unwrap_or(0.0),
                }
                span { "{rotation().0}" }
            }
            div { style: "display: flex; align-items: center; gap: 10px;",
                span { "Y rotation: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    value: "{rotation().1}",
                    oninput: move |evt| rotation.write().1 = evt.value().parse::<f64>().unwrap_or(0.0),
                }
                span { "{rotation().1}" }
            }
            div { style: "display: flex; align-items: center; gap: 10px;",
                span { "Z rotation: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    value: "{rotation().2}",
                    oninput: move |evt| rotation.write().2 = evt.value().parse::<f64>().unwrap_or(0.0),
                }
                span { "{rotation().2}" }
            }
        }
    }
}
