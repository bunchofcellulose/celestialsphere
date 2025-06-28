use crate::*;
use dioxus::web::WebEventExt;
use serde::{Deserialize, Serialize};
use web_sys::{
    wasm_bindgen::{closure, JsCast},
    window, HtmlInputElement, Url,
};

// Constants for SVG generation
const SVG_WIDTH: f64 = 600.0;
const SVG_HEIGHT: f64 = 600.0;
const SVG_RADIUS: f64 = 250.0;
const GRID_RESOLUTION: usize = 128;
const ARC_RESOLUTION: usize = 64;
const POINT_RADIUS_SMALL: f64 = 6.0;
const POINT_RADIUS_LARGE: f64 = 12.0;

/// Data structure for saving/loading celestial sphere state
#[derive(Serialize, Deserialize)]
struct SaveData {
    points: Vec<(Vec3, String, bool, bool)>,
    arcs: Vec<(usize, usize)>,
    great_circles: Vec<(usize, String)>,
    small_circles: Vec<(usize, f64, String)>,
}

/// Saves the current celestial sphere state to a JSON file
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
        download_blob(&json, "celestial_data.json");
    } else {
        web_sys::console::error_1(&"Failed to serialize data".into());
    }
}

/// Clears all data and resets the celestial sphere to initial state
pub fn new_file(
    mut points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    mut great_circles: Signal<Vec<GreatCircle>>,
    mut small_circles: Signal<Vec<SmallCircle>>,
    mut state: Signal<State>,
) {
    points.write().clear();
    arcs.write().clear();
    great_circles.write().clear();
    small_circles.write().clear();

    state.write().zoom = 1.0;
    state.write().rotation = [0.0, 0.0, 0.0];
    state.write().quaternion = Quaternion::identity();

    state.write().clear_selection();
}

/// Exports the current celestial sphere as an SVG file
pub fn save_svg(
    points: Signal<Vec<Point>>,
    arcs: Signal<Vec<(usize, usize)>>,
    great_circles: Signal<Vec<GreatCircle>>,
    small_circles: Signal<Vec<SmallCircle>>,
    state: Signal<State>,
) {
    let config = SvgConfig::new();
    let q = state.read().quaternion;
    let pts = points.read();

    let mut svg = create_svg_header(&config);

    if state.read().show_grid {
        svg.push_str(&generate_latitude_lines(q, &SvgConfig::new()));
        svg.push_str(&generate_longitude_lines(q, &SvgConfig::new()));
    }

    // Add grid if enabled
    if state.read().show_grid {
        svg.push_str(&generate_latitude_lines(q, &config));
        svg.push_str(&generate_longitude_lines(q, &config));
    }

    // Add geometric elements
    svg.push_str(&generate_great_circles(
        &great_circles.read(),
        &pts,
        q,
        &config,
    ));
    svg.push_str(&generate_small_circles(
        &small_circles.read(),
        &pts,
        q,
        &config,
    ));
    svg.push_str(&generate_arcs(&arcs.read(), &pts, q, &config));

    // Add sphere boundary
    svg.push_str(&format!(
        r#"<circle class="sphere" cx="{:.2}" cy="{:.2}" r="{:.2}"/>"#,
        config.center.0, config.center.1, config.radius
    ));

    // Add points and labels
    svg.push_str(&generate_points(&pts, q, &config));

    svg.push_str(&generate_circle_labels(
        &great_circles.read(),
        &small_circles.read(),
        pts.as_slice(),
        q,
        &config,
    ));

    svg.push_str("\n</svg>");

    download_blob(&svg, "celestial_sphere.svg");
}

// ===== Vector utility functions =====
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale_vec(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: [f64; 3]) -> [f64; 3] {
    let len = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if len == 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [a[0] / len, a[1] / len, a[2] / len]
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// ===== SVG helper structures and functions =====
struct SvgConfig {
    width: f64,
    height: f64,
    radius: f64,
    center: (f64, f64),
}

impl SvgConfig {
    fn new() -> Self {
        Self {
            width: SVG_WIDTH,
            height: SVG_HEIGHT,
            radius: SVG_RADIUS,
            center: (SVG_WIDTH / 2.0, SVG_HEIGHT / 2.0),
        }
    }

    fn project_point(&self, point: [f64; 3]) -> (f64, f64) {
        (
            self.center.0 + point[0] * self.radius,
            self.center.1 + point[1] * self.radius,
        )
    }
}

struct PathBuilder {
    front_path: String,
    back_path: String,
    prev_z: f64,
}

impl PathBuilder {
    fn new() -> Self {
        Self {
            front_path: String::new(),
            back_path: String::new(),
            prev_z: 0.0,
        }
    }

    fn add_point(&mut self, x: f64, y: f64, z: f64, is_first: bool) {
        if is_first {
            if z >= 0.0 {
                self.front_path.push_str(&format!("M {:.2} {:.2} ", x, y));
            } else {
                self.back_path.push_str(&format!("M {:.2} {:.2} ", x, y));
            }
        } else {
            if z >= 0.0 {
                if self.prev_z < 0.0 {
                    self.front_path.push_str(&format!("M {:.2} {:.2} ", x, y));
                } else {
                    self.front_path.push_str(&format!("L {:.2} {:.2} ", x, y));
                }
            } else {
                if self.prev_z >= 0.0 {
                    self.back_path.push_str(&format!("M {:.2} {:.2} ", x, y));
                } else {
                    self.back_path.push_str(&format!("L {:.2} {:.2} ", x, y));
                }
            }
        }
        self.prev_z = z;
    }

    fn generate_svg_paths(&self, front_class: &str, back_class: &str) -> String {
        let mut result = String::new();
        if !self.back_path.is_empty() {
            result.push_str(&format!(
                r#"<path class="{}" d="{}"/>"#,
                back_class, self.back_path
            ));
        }
        if !self.front_path.is_empty() {
            result.push_str(&format!(
                r#"<path class="{}" d="{}"/>"#,
                front_class, self.front_path
            ));
        }
        result
    }

    fn generate_grid_paths(&self) -> String {
        let mut result = String::new();
        if !self.back_path.is_empty() {
            result.push_str(&format!(
                r#"<path d="{}" stroke='#fff' stroke-width='1' stroke-dasharray='6,6' opacity='0.08' fill='none'/>"#,
                self.back_path
            ));
        }
        if !self.front_path.is_empty() {
            result.push_str(&format!(
                r#"<path d="{}" stroke='#fff' stroke-width='1' stroke-dasharray='6,6' opacity='0.18' fill='none'/>"#,
                self.front_path
            ));
        }
        result
    }
}

fn create_svg_header(config: &SvgConfig) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<style>
.sphere {{ fill: none; stroke: #bbb; stroke-width: 4; }}
.arc {{ fill: none; stroke: #fbc02d; stroke-width: 4; }}
.arc-back {{ fill: none; stroke: #fbc02d; stroke-width: 4; opacity: 0.4; }}
.great-circle {{ fill: none; stroke: #00bcd4; stroke-width: 3; opacity: 0.9; }}
.great-circle-back {{ fill: none; stroke: #00bcd4; stroke-width: 3; opacity: 0.4; }}
.small-circle {{ fill: none; stroke: #e64a19; stroke-width: 3; opacity: 0.9; }}
.small-circle-back {{ fill: none; stroke: #e64a19; stroke-width: 3; opacity: 0.4; }}
.point {{ fill: #e53935; stroke: none; }}
.point-back {{ fill: #e53935; stroke: none; opacity: 0.4; }}
.label {{ fill: #fff; font-size: 28px; font-family: sans-serif; font-weight: bold; pointer-events: none; }}
.label-back {{ fill: #fff; font-size: 28px; font-family: sans-serif; font-weight: bold; pointer-events: none; opacity: 0.4; }}
.small-label {{ fill: #ffb74d; font-size: 18px; font-family: sans-serif; font-weight: 500; pointer-events: none; }}
.small-label-back {{ fill: #ffb74d; font-size: 18px; font-family: sans-serif; font-weight: 500; pointer-events: none; opacity: 0.4; }}
</style>
"#,
        w = config.width,
        h = config.height
    )
}

fn download_blob(content: &str, filename: &str) {
    if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&content.into())) {
        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
            let document = window().unwrap().document().unwrap();
            if let Ok(a) = document.create_element("a") {
                if a.set_attribute("href", &url).is_ok()
                    && a.set_attribute("download", filename).is_ok()
                {
                    a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
                    let _ = Url::revoke_object_url(&url);
                }
            }
        }
    }
}

/// File panel component providing save, load, and new file functionality
#[component]
pub fn FilePanel(
    mut points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    mut great_circles: Signal<Vec<GreatCircle>>,
    mut small_circles: Signal<Vec<SmallCircle>>,
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
                        if let Ok(result) = fr_c.result() {
                            if let Some(text) = result.as_string() {
                                if let Err(error) = restore_data_from_json(
                                    &text,
                                    points,
                                    arcs,
                                    great_circles,
                                    small_circles,
                                    state,
                                ) {
                                    web_sys::console::error_1(&error.into());
                                }
                            }
                        }
                    })
                        as Box<dyn FnMut(_)>);

                    file_reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
                    file_reader.read_as_text(&file).unwrap();
                    onloadend.forget();
                }
            }
        }
    };

    let mut show_save_dropdown = use_signal(|| false);

    rsx! {
        div { class: "file-panel",
            button {
                onclick: move |_| show_save_dropdown.set(!show_save_dropdown()),
                style: "background-image: url({SAVE});",
            }
            if show_save_dropdown() {
                div { class: "file-panel-dropdown",
                    button {
                        class: "file-panel-dropdown-btn",
                        onclick: move |_| {
                            save_to_file(points, arcs, great_circles, small_circles);
                            show_save_dropdown.set(false);
                        },
                        "Save as JSON"
                    }
                    button {
                        class: "file-panel-dropdown-btn",
                        onclick: move |_| {
                            save_svg(points, arcs, great_circles, small_circles, state);
                            show_save_dropdown.set(false);
                        },
                        "Save as SVG"
                    }
                }
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
                onclick: move |_| new_file(points, arcs, great_circles, small_circles, state),
                style: "background-image: url({NEW_FILE});",
            }
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
}

// ===== SVG Generation helpers =====
fn generate_latitude_lines(q: Quaternion, config: &SvgConfig) -> String {
    let mut svg = String::new();

    for lat_deg in [-60.0f64, -30.0, 0.0, 30.0, 60.0] {
        let lat = lat_deg.to_radians();
        let mut path_builder = PathBuilder::new();

        for i in 0..=GRID_RESOLUTION {
            let lon = (i as f64) * std::f64::consts::TAU / (GRID_RESOLUTION as f64);
            let x3 = lat.cos() * lon.cos();
            let y3 = lat.cos() * lon.sin();
            let z3 = lat.sin();
            let pt = q.rotate_point_active([x3, y3, z3]);
            let (x, y) = config.project_point(pt);
            let z = pt[2];
            path_builder.add_point(x, y, z, i == 0);
        }

        svg.push_str(&path_builder.generate_grid_paths());
    }
    svg
}

fn generate_longitude_lines(q: Quaternion, config: &SvgConfig) -> String {
    let mut svg = String::new();

    for lon_deg in [
        0.0f64, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
    ] {
        let lon = lon_deg.to_radians();
        let mut path_builder = PathBuilder::new();

        for i in 0..=GRID_RESOLUTION {
            let lat = -std::f64::consts::FRAC_PI_2
                + (i as f64) * std::f64::consts::PI / (GRID_RESOLUTION as f64);
            let x3 = lat.cos() * lon.cos();
            let y3 = lat.cos() * lon.sin();
            let z3 = lat.sin();
            let pt = q.rotate_point_active([x3, y3, z3]);
            let (x, y) = config.project_point(pt);
            let z = pt[2];
            path_builder.add_point(x, y, z, i == 0);
        }

        svg.push_str(&path_builder.generate_grid_paths());
    }
    svg
}

fn generate_great_circles(
    great_circles: &[GreatCircle],
    points: &[Point],
    q: Quaternion,
    config: &SvgConfig,
) -> String {
    let mut svg = String::new();

    for gc in great_circles {
        let pole = points[gc.pole].absolute;
        let mut path_builder = PathBuilder::new();

        for i in 0..=GRID_RESOLUTION {
            let theta = (i as f64) * std::f64::consts::TAU / (GRID_RESOLUTION as f64);
            let mut v = if pole[2].abs() < 0.99 {
                [pole[1], -pole[0], 0.0]
            } else {
                [1.0, 0.0, 0.0]
            };
            v = normalize(cross(pole, v));
            let w = normalize(cross(pole, v));
            let pt = add(scale_vec(v, theta.cos()), scale_vec(w, theta.sin()));
            let pt = normalize(pt);
            let pt = q.rotate_point_active(pt);
            let (x, y) = config.project_point(pt);
            let z = pt[2];
            path_builder.add_point(x, y, z, i == 0);
        }

        svg.push_str(&path_builder.generate_svg_paths("great-circle", "great-circle-back"));
    }
    svg
}

fn generate_small_circles(
    small_circles: &[SmallCircle],
    points: &[Point],
    q: Quaternion,
    config: &SvgConfig,
) -> String {
    let mut svg = String::new();

    for sc in small_circles {
        let pole = points[sc.pole].absolute;
        let d = sc.plane_distance;
        let mut path_builder = PathBuilder::new();

        for i in 0..=GRID_RESOLUTION {
            let theta = (i as f64) * std::f64::consts::TAU / (GRID_RESOLUTION as f64);
            let mut v = if pole[2].abs() < 0.99 {
                [pole[1], -pole[0], 0.0]
            } else {
                [1.0, 0.0, 0.0]
            };
            v = normalize(cross(pole, v));
            let w = normalize(cross(pole, v));
            let pt = add(
                scale_vec(v, theta.cos() * (1.0 - d * d).sqrt()),
                scale_vec(w, theta.sin() * (1.0 - d * d).sqrt()),
            );
            let pt = add(pt, scale_vec(pole, d));
            let pt = normalize(pt);
            let pt = q.rotate_point_active(pt);
            let (x, y) = config.project_point(pt);
            let z = pt[2];
            path_builder.add_point(x, y, z, i == 0);
        }

        svg.push_str(&path_builder.generate_svg_paths("small-circle", "small-circle-back"));
    }
    svg
}

fn generate_arcs(
    arcs: &[(usize, usize)],
    points: &[Point],
    q: Quaternion,
    config: &SvgConfig,
) -> String {
    let mut svg = String::new();

    for (a, b) in arcs {
        let pa = q.rotate_point_active(points[*a].absolute);
        let pb = q.rotate_point_active(points[*b].absolute);
        let mut path_builder = PathBuilder::new();

        let dot_product = dot(pa, pb);
        let angle = dot_product.clamp(-1.0, 1.0).acos();
        let sin_angle = angle.sin();

        for i in 0..=ARC_RESOLUTION {
            let t = i as f64 / ARC_RESOLUTION as f64;
            let coeff_a = ((1.0 - t) * angle).sin() / sin_angle;
            let coeff_b = (t * angle).sin() / sin_angle;
            let mut p = [
                pa[0] * coeff_a + pb[0] * coeff_b,
                pa[1] * coeff_a + pb[1] * coeff_b,
                pa[2] * coeff_a + pb[2] * coeff_b,
            ];
            let norm = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            p = [p[0] / norm, p[1] / norm, p[2] / norm];
            let (x, y) = config.project_point(p);
            let z = p[2];
            path_builder.add_point(x, y, z, i == 0);
        }

        svg.push_str(&path_builder.generate_svg_paths("arc", "arc-back"));
    }
    svg
}

fn generate_points(points: &[Point], q: Quaternion, config: &SvgConfig) -> String {
    let mut svg = String::new();

    // Render back points first
    for point in points {
        let p = q.rotate_point_active(point.absolute);
        let (x, y) = config.project_point(p);
        let z = p[2];

        if z < 0.0 {
            let r = if point.name.is_empty() {
                POINT_RADIUS_SMALL
            } else {
                POINT_RADIUS_LARGE
            };
            svg.push_str(&format!(
                r#"<circle class="point-back" cx="{:.2}" cy="{:.2}" r="{:.2}"/>"#,
                x, y, r
            ));
            if !point.name.is_empty() {
                svg.push_str(&format!(
                    r#"<text class="label-back" x="{:.2}" y="{:.2}">{}</text>"#,
                    x + 16.0,
                    y - 12.0,
                    point.name
                ));
            }
        }
    }

    // Render front points
    for point in points {
        let p = q.rotate_point_active(point.absolute);
        let (x, y) = config.project_point(p);
        let z = p[2];

        if z >= 0.0 {
            let r = if point.name.is_empty() {
                POINT_RADIUS_SMALL
            } else {
                POINT_RADIUS_LARGE
            };
            svg.push_str(&format!(
                r#"<circle class="point" cx="{:.2}" cy="{:.2}" r="{:.2}"/>"#,
                x, y, r
            ));
            if !point.name.is_empty() {
                svg.push_str(&format!(
                    r#"<text class="label" x="{:.2}" y="{:.2}">{}</text>"#,
                    x + 16.0,
                    y - 12.0,
                    point.name
                ));
            }
        }
    }

    svg
}

fn generate_circle_labels(
    great_circles: &[GreatCircle],
    small_circles: &[SmallCircle],
    points: &[Point],
    q: Quaternion,
    config: &SvgConfig,
) -> String {
    let mut svg = String::new();

    // Great circle labels
    for gc in great_circles {
        let pole = points[gc.pole].absolute;
        let mut v = if pole[2].abs() < 0.99 {
            [pole[1], -pole[0], 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        v = normalize(cross(pole, v));
        let w = normalize(cross(pole, v));

        // Try both theta=0 and theta=PI, pick the one with max z (frontmost)
        let mut best_pt = None;
        let mut best_z = -1.0;
        for &theta in &[0.0, std::f64::consts::PI] {
            let pt = normalize(add(scale_vec(v, theta.cos()), scale_vec(w, theta.sin())));
            let pt = q.rotate_point_active(pt);
            if pt[2] > best_z {
                best_z = pt[2];
                best_pt = Some(pt);
            }
        }

        if let Some(pt) = best_pt {
            let mut x = config.center.0 + pt[0] * config.radius;
            let mut y = config.center.1 + pt[1] * config.radius;
            // Offset label outward from the sphere edge
            let label_offset = 24.0;
            let label_width = 120.0;
            let label_height = 32.0;
            x += pt[0] * label_offset;
            y += pt[1] * label_offset;
            x = x.clamp(8.0, config.width - label_width - 8.0);
            y = y.clamp(label_height, config.height - 8.0);
            if !gc.name.is_empty() {
                svg.push_str(&format!(
                    r#"<text class="label" x="{:.2}" y="{:.2}">{}</text>"#,
                    x, y, gc.name
                ));
            }
        }
    }

    // Small circle labels
    for sc in small_circles {
        let pole = points[sc.pole].absolute;
        let d = sc.plane_distance;
        let mut v = if pole[2].abs() < 0.99 {
            [pole[1], -pole[0], 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        v = normalize(cross(pole, v));
        let w = normalize(cross(pole, v));
        let acos_d = (-d).clamp(-1.0, 1.0).acos();
        let mut best_pt: Option<([f64; 3], bool)> = None;
        let mut best_z_abs = f64::INFINITY;

        for &theta in &[acos_d, std::f64::consts::PI - acos_d] {
            let pt = add(
                scale_vec(v, theta.cos() * (1.0 - d * d).sqrt()),
                scale_vec(w, theta.sin() * (1.0 - d * d).sqrt()),
            );
            let pt = add(pt, scale_vec(pole, d));
            let pt = normalize(pt);
            let pt_rot = q.rotate_point_active(pt);
            let z_abs = pt_rot[2].abs();
            if z_abs < best_z_abs {
                best_z_abs = z_abs;
                best_pt = Some((pt_rot, pt_rot[2] < 0.0));
            }
        }

        if let Some((pt, is_back)) = best_pt {
            let mut x = config.center.0 + pt[0] * config.radius;
            let mut y = config.center.1 + pt[1] * config.radius;
            let label_offset = 20.0;
            let label_width = 80.0;
            let label_height = 20.0;
            x += pt[0] * label_offset;
            y += pt[1] * label_offset;
            x = x.clamp(8.0, config.width - label_width - 8.0);
            y = y.clamp(label_height, config.height - 8.0);
            if !sc.name.is_empty() {
                let class = if is_back {
                    "small-label-back"
                } else {
                    "small-label"
                };
                svg.push_str(&format!(
                    r#"<text class="{class}" x="{:.2}" y="{:.2}">{}</text>"#,
                    x,
                    y,
                    sc.name,
                    class = class
                ));
            }
        }
    }

    svg
}

// ===== File loading helpers =====
fn restore_data_from_json(
    text: &str,
    mut points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    mut great_circles: Signal<Vec<GreatCircle>>,
    mut small_circles: Signal<Vec<SmallCircle>>,
    mut state: Signal<State>,
) -> Result<(), String> {
    let data: SaveData =
        serde_json::from_str(text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

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

    // Restore arcs
    arcs.set(data.arcs);

    // Restore great circles
    let gcs = data
        .great_circles
        .into_iter()
        .map(|(pole, name)| {
            let mut gc = GreatCircle::new(pole);
            gc.name = name;
            gc
        })
        .collect();
    great_circles.set(gcs);

    // Restore small circles
    let scs = data
        .small_circles
        .into_iter()
        .map(|(pole, plane_distance, name)| {
            let mut sc = SmallCircle::new(pole, plane_distance);
            sc.name = name;
            sc
        })
        .collect();
    small_circles.set(scs);

    // Reset state
    state.set(State::initialize());
    state.write().clear_selection();

    Ok(())
}
