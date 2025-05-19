use dioxus::{html::input_data::MouseButton, prelude::*};

mod circle;
mod event;
mod panels;
mod point;
use circle::*;
use event::*;
use panels::*;
use point::*;

const FAVICON: Asset = asset!("/assets/triangle.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const SAVE: Asset = asset!("/assets/save.ico");
const LOAD: Asset = asset!("/assets/load.ico");
const NEW_FILE: Asset = asset!("/assets/new.ico");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let points = use_signal(Vec::<Point>::new);
    let arcs = use_signal(Vec::<(usize, usize)>::new);
    let great_circles = use_signal(Vec::<GreatCircle>::new);
    let small_circles = use_signal(Vec::<SmallCircle>::new);
    let scale = use_signal(|| (1.0, [0.0, 0.0, 0.0], Quaternion::identity()));
    let state = use_signal(State::initialize);
    let show_grid = use_signal(|| false);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        SelectionBox { points, state }
        SlidersPanel { points, scale, show_grid }
        LeftPanel {
            state,
            points,
            great_circles,
            small_circles,
        }
        GitHubIcon {}
        FilePanel {
            points,
            arcs,
            great_circles,
            small_circles,
            scale,
            state,
        }
        Sphere {
            points,
            arcs,
            great_circles,
            small_circles,
            state,
            scale,
            show_grid,
        }
    }
}

#[component]
pub fn Sphere(
    mut points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    mut great_circles: Signal<Vec<GreatCircle>>,
    mut small_circles: Signal<Vec<SmallCircle>>,
    mut state: Signal<State>,
    mut scale: Signal<(f64, Vec3, Quaternion)>,
    show_grid: Signal<bool>,
) -> Element {
    let dragged_point = use_signal(|| None::<usize>);
    let is_rotating = use_signal(|| false);
    let last_rotation_pos = use_signal(|| (0.0, 0.0));

    let primary_click = move |event: Event<MouseData>| {
        handle_primary_click(event, points, great_circles, state, scale, dragged_point)
    };
    let secondary_click =
        move |event: Event<MouseData>| handle_secondary_click(event, points, arcs, state);
    let middle_click =
        move |event: Event<MouseData>| handle_middle_click(event, is_rotating, last_rotation_pos);
    let scroll = move |event: Event<WheelData>| handle_scroll(event, scale);
    let mouse_move = move |event: Event<MouseData>| {
        handle_mouse_move(
            event,
            points,
            great_circles,
            state,
            scale,
            dragged_point,
            is_rotating,
            last_rotation_pos,
        )
    };
    let mouse_up =
        move |event: Event<MouseData>| handle_mouse_up(event, dragged_point, is_rotating);
    let key_event = move |event: Event<KeyboardData>| {
        handle_key_event(
            event,
            points,
            arcs,
            great_circles,
            small_circles,
            scale,
            state,
        )
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
                oncontextmenu: move |event| event.prevent_default(),
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
                        stroke_width: "0.2",
                        fill: "rgba(0, 0, 0, 0.4)",
                    }
                    if show_grid() {
                        CoordinateGrid { scale }
                    }
                    GreatCircleDrawer { great_circles, points }
                    SmallCircleDrawer { small_circles, points }
                    GreatCircleLabels { great_circles, points }
                    SmallCircleLabels { small_circles, points }
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

pub struct State {
    selected: Vec<usize>,
}

impl State {
    pub fn initialize() -> Self {
        Self { selected: vec![] }
    }

    pub fn selected(&self) -> &[usize] {
        self.selected.as_slice()
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    pub fn pop_selected(&mut self) -> Option<usize> {
        self.selected.pop()
    }

    pub fn toggle_select(&mut self, multi: bool, id: usize) -> bool {
        if multi {
            if self.selected.contains(&id) {
                self.selected.retain(|&x| x != id);
                false
            } else {
                self.selected.push(id);
                true
            }
        } else if self.selected.len() == 1 && self.selected[0] == id {
            self.selected.clear();
            false
        } else {
            self.selected.clear();
            self.selected.push(id);
            true
        }
    }

    pub fn select(&mut self, id: usize) -> bool {
        if self.selected.contains(&id) {
            false
        } else {
            self.selected.push(id);
            true
        }
    }
}
