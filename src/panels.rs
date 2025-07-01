use crate::*;

#[component]
pub fn SelectionBox(points: Signal<Vec<Point>>, state: Signal<State>) -> Element {
    let selected_points: Vec<Point> = state
        .read()
        .selected()
        .iter()
        .map(|&id| points.read()[id].clone())
        .collect();

    rsx! {
        div { class: "right-info-boxes-container",
            if selected_points.len() > 3 {
                div { class: "info-box compact-selection",
                    h3 { "Multiple Selection ({selected_points.len()} points)" }
                    div { class: "selection-summary",
                        {
                            let ids = selected_points
                                .iter()
                                .map(|p| p.id.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("IDs: {}", ids)
                        }
                        br {}
                        {
                            let names = selected_points
                                .iter()
                                .map(|p| {
                                    if p.name.is_empty() { "[unnamed]".to_string() } else { p.name.clone() }
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("Names: {}", names)
                        }
                    }
                    div { class: "selection-controls",
                        h4 { "Bulk Actions:" }
                        div { class: "checkbox-row",
                            input {
                                r#type: "checkbox",
                                id: "bulk-movable",
                                checked: selected_points.iter().all(|p| p.movable),
                                onchange: move |event| {
                                    let new_value = event.value() == "true";
                                    for &id in state.read().selected() {
                                        points.write()[id].movable = new_value;
                                    }
                                },
                            }
                            label { r#for: "bulk-movable", "All Movable" }
                        }
                        div { class: "checkbox-row",
                            input {
                                r#type: "checkbox",
                                id: "bulk-removable",
                                checked: selected_points.iter().all(|p| p.removable),
                                onchange: move |event| {
                                    let new_value = event.value() == "true";
                                    for &id in state.read().selected() {
                                        points.write()[id].removable = new_value;
                                    }
                                },
                            }
                            label { r#for: "bulk-removable", "All Removable" }
                        }
                        div { class: "checkbox-row",
                            input {
                                r#type: "checkbox",
                                id: "bulk-hidden",
                                checked: selected_points.iter().all(|p| p.hidden),
                                onchange: move |event| {
                                    let new_value = event.value() == "true";
                                    for &id in state.read().selected() {
                                        points.write()[id].hidden = new_value;
                                    }
                                },
                            }
                            label { r#for: "bulk-hidden", "All Hidden" }
                        }
                    }
                }
            } else {
                div {
                    class: "selection-details",
                    style: "max-height: 80vh; overflow-y: auto;",
                    for Point { id , absolute : [x , y , z] , name , movable , removable , hidden , abs_polar : [theta , phi] , rotated : [rx , ry , rz] , rot_polar : [rtheta , rphi] } in selected_points {
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
                            if name.is_empty() {
                                " [unnamed]"
                            }
                            br {}
                            br {}
                            div { class: "point-controls",
                                div { class: "checkbox-row",
                                    input {
                                        r#type: "checkbox",
                                        id: "movable-{id}",
                                        checked: "{movable}",
                                        onchange: move |event| {
                                            points.write()[id].movable = event.value() == "true";
                                        },
                                    }
                                    label { r#for: "movable-{id}", "Movable" }
                                }
                                div { class: "checkbox-row",
                                    input {
                                        r#type: "checkbox",
                                        id: "removable-{id}",
                                        checked: "{removable}",
                                        onchange: move |event| {
                                            points.write()[id].removable = event.value() == "true";
                                        },
                                    }
                                    label { r#for: "removable-{id}", "Removable" }
                                }
                                div { class: "checkbox-row",
                                    input {
                                        r#type: "checkbox",
                                        id: "hidden-{id}",
                                        checked: "{hidden}",
                                        onchange: move |event| {
                                            points.write()[id].hidden = event.value() == "true";
                                        },
                                    }
                                    label { r#for: "hidden-{id}", "Hidden" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SlidersPanel(points: Signal<Vec<Point>>, state: Signal<State>) -> Element {
    let mut change = move || {
        let q = Quaternion::from_euler_deg(state.read().rotation);
        state.write().quaternion = q;
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
                    value: "{state.read().rotation[0]}",
                    oninput: move |evt| {
                        state.write().rotation[0] = evt.value().parse::<f64>().unwrap_or(0.0);
                        change();
                    },
                }
                span { "{state.read().rotation[0]:.1}°" }
            }
            div {
                span { "Y rotation: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    step: "1",
                    value: "{state.read().rotation[1]}",
                    oninput: move |evt| {
                        state.write().rotation[1] = evt.value().parse::<f64>().unwrap_or(0.0);
                        change();
                    },
                }
                span { "{state.read().rotation[1]:.1}°" }
            }
            div {
                span { "Z rotation: " }
                input {
                    r#type: "range",
                    min: "0",
                    max: "360",
                    step: "1",
                    value: "{state.read().rotation[2]}",
                    oninput: move |evt| {
                        state.write().rotation[2] = evt.value().parse::<f64>().unwrap_or(0.0);
                        change();
                    },
                }
                span { "{state.read().rotation[2]:.1}°" }
            }
            div {
                span { "Zoom: " }
                input {
                    r#type: "range",
                    min: "0.5",
                    max: "2.0",
                    step: "0.01",
                    value: "{state.read().zoom}",
                    oninput: move |evt| {
                        state.write().zoom = evt.value().parse::<f64>().unwrap_or(1.0);
                    },
                }
                span { "{(state.read().zoom * 100.0).round()}%" }
            }
            div { class: "checkbox-control",
                input {
                    r#type: "checkbox",
                    id: "grid-toggle",
                    checked: "{state.read().show_grid}",
                    onchange: move |evt| {
                        state.write().show_grid = evt.value() == "true";
                    },
                }
                label { r#for: "grid-toggle", "Show coordinate grid" }
            }
            div { class: "checkbox-control",
                input {
                    r#type: "checkbox",
                    id: "hide-toggle",
                    checked: "{state.read().show_hidden}",
                    onchange: move |evt| {
                        state.write().show_hidden = evt.value() == "true";
                    },
                }
                label { r#for: "hide-toggle", "Show hidden points" }
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
            // Group information box
            if state.read().selected().len() > 1 {
                if let Some(group_members) = state
                    .read()
                    .find_group_containing(state.read().selected()[0])
                    .map(|group_idx| state.read().groups[group_idx].clone())
                {
                    if group_members.iter().all(|&id| state.read().selected().contains(&id)) {
                        div { class: "info-box",
                            h3 { "Group Information" }
                            "Group size: {group_members.len()} points"
                            br {}
                            {
                                let member_ids = group_members
                                    .iter()
                                    .map(|id| id.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                format!("Group member ids: {}", member_ids)
                            }
                            br {}
                            {
                                let member_names = group_members
                                    .iter()
                                    .map(|&id| {
                                        let name = &points.read()[id].name;
                                        if name.is_empty() { "[unnamed]".to_string() } else { name.clone() }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                format!("Group member names: {}", member_names)
                            }
                        }
                    }
                }
            }

            if let &[pole] = state.read().selected() {
                if let Some(gc) = great_circles().iter().find(|gc| gc.pole == pole) {
                    div { class: "info-box",
                        h3 { "Great Circle" }
                        "Pole ID: {pole}"
                        br {}
                        "Name: {gc.name}"
                        if gc.name.is_empty() {
                            " [unnamed]"
                        }
                    }
                }
                if let Some(sc) = small_circles.read().iter().find(|sc| sc.pole == pole) {
                    div { class: "info-box",
                        h3 { "Small Circle" }
                        "Pole ID: {pole}"
                        br {}
                        "Name: {sc.name}"
                        if sc.name.is_empty() {
                            " [unnamed]"
                        }
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
