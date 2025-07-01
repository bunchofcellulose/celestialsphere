use crate::circle::*;
use crate::point::*;
use crate::State;
use dioxus::prelude::*;

#[derive(Debug, Clone)]
pub enum Selected {
    Existing(usize),
    New(Point),
    None,
}

pub fn select_point(x: f64, y: f64, points: Vec<Point>, state: Quaternion) -> Selected {
    let [px, py, pz] = transform_viewport_to_sphere(x, y);
    if pz.is_nan() {
        return Selected::None;
    }
    for p in points.iter() {
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
    Selected::New(Point::from_vec3_rotated(points.len(), [px, py, pz], state))
}

pub fn handle_primary_click(
    event: Event<MouseData>,
    mut points: Signal<Vec<Point>>,
    great_circles: Signal<Vec<GreatCircle>>,
    mut state: Signal<State>,
    mut dragged_point: Signal<Option<usize>>,
) {
    let multi = event.modifiers().shift();
    let scale_val = state.read().quaternion;
    match select_point(
        event.client_coordinates().x,
        event.client_coordinates().y,
        points(),
        scale_val,
    ) {
        Selected::None => (),
        Selected::New(mut point) => {
            if event.modifiers().shift() {
                let threshold = 0.05;
                let snapped =
                    snap_to_great_circle(point.rotated, &great_circles(), &points(), threshold);
                point = Point::from_vec3_rotated(points().len(), snapped, scale_val);
                points.write().push(point);
                let new_point_idx = points().len() - 1;
                state.write().toggle_select_group(multi, new_point_idx);
            } else {
                points.write().push(point);
                state.write().toggle_select_group(multi, points().len() - 1);
            }
        }
        Selected::Existing(selected) => {
            if state.write().toggle_select_group(multi, selected) && points()[selected].movable {
                dragged_point.set(Some(selected));
            }
        }
    }
}

pub fn handle_secondary_click(
    event: Event<MouseData>,
    points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    state: Signal<State>,
) {
    if state.read().selected().is_empty() {
        return;
    }
    match select_point(
        event.client_coordinates().x,
        event.client_coordinates().y,
        points(),
        Quaternion::identity(),
    ) {
        Selected::None => (),
        Selected::New(_) => {}
        Selected::Existing(p) => {
            for selected in state.read().selected() {
                if p == *selected {
                    continue;
                }
                if arcs().contains(&(*selected, p)) {
                    arcs.write().retain(|&(p1, p2)| p1 != *selected || p2 != p);
                } else if arcs().contains(&(p, *selected)) {
                    arcs.write().retain(|&(p1, p2)| p1 != p || p2 != *selected);
                } else {
                    arcs.write().push((*selected, p));
                }
            }
        }
    }
}

pub fn handle_middle_click(
    event: Event<MouseData>,
    mut is_rotating: Signal<bool>,
    mut last_rotation_pos: Signal<(f64, f64)>,
) {
    is_rotating.set(true);
    last_rotation_pos.set((event.client_coordinates().x, event.client_coordinates().y));
    event.prevent_default();
}

pub fn handle_scroll(event: Event<WheelData>, mut state: Signal<State>) {
    let delta = event.delta().strip_units().y;
    let zoom_factor = 1.0 - delta * 0.001;
    let mut new_scale = state.read().zoom * zoom_factor;
    new_scale = new_scale.clamp(0.5, 2.0);
    state.write().zoom = new_scale;
}

pub fn handle_mouse_move(
    event: Event<MouseData>,
    mut points: Signal<Vec<Point>>,
    great_circles: Signal<Vec<GreatCircle>>,
    mut state: Signal<State>,
    dragged_point: Signal<Option<usize>>,
    is_rotating: Signal<bool>,
    mut last_rotation_pos: Signal<(f64, f64)>,
) {
    if let Some(dragged_idx) = dragged_point() {
        let viewport_x = event.client_coordinates().x;
        let viewport_y = event.client_coordinates().y;
        let [px, py, pz] = transform_viewport_to_sphere(viewport_x, viewport_y);
        if pz.is_nan() {
            return;
        }

        let group_members = state.read().get_group_members(dragged_idx);
        let original_pos = points()[dragged_idx].rotated;
        let new_pos = if event.modifiers().shift() {
            let threshold = 0.05;
            snap_to_great_circle([px, py, pz], &great_circles(), &points(), threshold)
        } else {
            [px, py, pz]
        };

        // Calculate rotation quaternion from original to new position
        let dot_product = original_pos[0] * new_pos[0]
            + original_pos[1] * new_pos[1]
            + original_pos[2] * new_pos[2];
        let dot_clamped = dot_product.clamp(-1.0, 1.0);

        // Only apply transformation if there's significant movement
        if (dot_clamped - 1.0).abs() > 1e-6 {
            let cross_product = [
                original_pos[1] * new_pos[2] - original_pos[2] * new_pos[1],
                original_pos[2] * new_pos[0] - original_pos[0] * new_pos[2],
                original_pos[0] * new_pos[1] - original_pos[1] * new_pos[0],
            ];

            let cross_magnitude =
                (cross_product[0].powi(2) + cross_product[1].powi(2) + cross_product[2].powi(2))
                    .sqrt();

            if cross_magnitude > 1e-10 {
                // Calculate rotation angle and axis
                let angle = dot_clamped.acos();
                let axis = [
                    cross_product[0] / cross_magnitude,
                    cross_product[1] / cross_magnitude,
                    cross_product[2] / cross_magnitude,
                ];

                // Create rotation quaternion
                let rotation_quat = Quaternion::from_axis_angle(axis, angle);

                // Apply rotation to all group members
                for &member_idx in &group_members {
                    if points()[member_idx].movable {
                        let current_rotated = points()[member_idx].rotated;
                        let new_rotated = rotation_quat.rotate_point_active(current_rotated);
                        points.write()[member_idx].move_to(new_rotated, state.read().quaternion);
                    }
                }
            }
        }

        // Update selection to include all group members
        for &member_idx in &group_members {
            state.write().select(member_idx);
        }
    }
    if is_rotating() {
        let current_x = event.client_coordinates().x;
        let current_y = event.client_coordinates().y;
        let (last_x, last_y) = last_rotation_pos();
        let sensitivity = 0.005;
        let delta_x = (current_x - last_x) * sensitivity;
        let delta_y = -(current_y - last_y) * sensitivity;
        let rotation_y = Quaternion::from_axis_angle([1.0, 0.0, 0.0], delta_y);
        let rotation_x = Quaternion::from_axis_angle([0.0, 1.0, 0.0], delta_x);
        let new_rotation = rotation_y
            .multiply(rotation_x)
            .multiply(state.read().quaternion);
        state.write().quaternion = new_rotation;
        state.write().rotation = new_rotation.to_euler_deg();
        last_rotation_pos.set((current_x, current_y));
        for point in points.write().iter_mut() {
            point.rotate(new_rotation);
        }
    }
}

pub fn handle_mouse_up(
    _event: Event<MouseData>,
    mut dragged_point: Signal<Option<usize>>,
    mut is_rotating: Signal<bool>,
) {
    dragged_point.set(None);
    is_rotating.set(false);
}

pub fn handle_key_event(
    event: Event<KeyboardData>,
    mut points: Signal<Vec<Point>>,
    mut arcs: Signal<Vec<(usize, usize)>>,
    mut great_circles: Signal<Vec<GreatCircle>>,
    mut small_circles: Signal<Vec<SmallCircle>>,
    mut state: Signal<State>,
) {
    event.prevent_default();
    let q = state.read().quaternion;
    let mut s = state.write();

    // Handle group operations first
    match event.key() {
        Key::Character(ref c)
            if (c.as_str() == "g" || c.as_str() == "G") && event.modifiers().ctrl() =>
        {
            s.create_group_from_selected();
            return;
        }
        Key::Character(ref c)
            if (c.as_str() == "u" || c.as_str() == "U") && event.modifiers().ctrl() =>
        {
            s.ungroup_selected();
            return;
        }
        Key::Character(ref c)
            if (c.as_str() == "h" || c.as_str() == "H") && event.modifiers().ctrl() =>
        {
            s.show_hidden = !s.show_hidden;
            return;
        }
        _ => {}
    }

    // Get all points that should be affected (including group members)
    let mut affected_points = Vec::new();
    for &selected_id in s.selected().iter() {
        let group_members = s.get_group_members(selected_id);
        for member in group_members {
            if !affected_points.contains(&member) {
                affected_points.push(member);
            }
        }
    }

    for i in affected_points.iter().rev().copied().collect::<Vec<_>>() {
        match event.key() {
            Key::Delete => {
                if !points()[i].removable {
                    continue;
                }

                // Get all group members before deletion and sort them in descending order
                let mut group_members = s.get_group_members(i);
                group_members.sort_by(|a, b| b.cmp(a)); // Sort in descending order for safe removal
                group_members.dedup(); // Remove duplicates

                // Remove all group members
                for &member_idx in &group_members {
                    if member_idx < points().len() && points()[member_idx].removable {
                        // Clean up references before removal
                        arcs.write()
                            .retain(|&(p1, p2)| p1 != member_idx && p2 != member_idx);
                        great_circles.write().retain(|x| x.pole != member_idx);
                        small_circles.write().retain(|sc| sc.pole != member_idx);

                        // Store the last index before removal
                        let last_idx = points().len() - 1;

                        // Remove the point
                        points.write().swap_remove(member_idx);

                        // Update the ID of the swapped point if it exists
                        if member_idx < points().len() {
                            points.write()[member_idx].id = member_idx;
                        }

                        // Fix indices after swap_remove - update references from last_idx to member_idx
                        if member_idx != last_idx {
                            for (p1, p2) in arcs.write().iter_mut() {
                                if *p1 == last_idx {
                                    *p1 = member_idx;
                                }
                                if *p2 == last_idx {
                                    *p2 = member_idx;
                                }
                            }
                            for x in great_circles.write().iter_mut() {
                                if x.pole == last_idx {
                                    x.pole = member_idx;
                                }
                            }
                            for sc in small_circles.write().iter_mut() {
                                if sc.pole == last_idx {
                                    sc.pole = member_idx;
                                }
                            }
                        }

                        s.update_group_indices(member_idx);
                    }
                }
                s.clear_selection();
                break;
            }
            Key::Escape => {
                s.clear_selection();
                break;
            }
            Key::Character(ref c) if c.as_str() == "." => {
                if great_circles().iter().all(|x| x.pole != i) {
                    great_circles.write().push(GreatCircle::new(i));
                } else {
                    great_circles.write().retain(|x| x.pole != i);
                }
            }
            Key::Character(ref c) if c.as_str() == ">" && event.modifiers().shift() => {
                let selected = s.selected();
                if selected.len() == 2 {
                    let p1 = points()[selected[0]].absolute;
                    let p2 = points()[selected[1]].absolute;
                    let cross = [
                        p1[1] * p2[2] - p1[2] * p2[1],
                        p1[2] * p2[0] - p1[0] * p2[2],
                        p1[0] * p2[1] - p1[1] * p2[0],
                    ];
                    let mag2 = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
                    let normal = if mag2 < 1e-10 {
                        let perp = if p1[0].abs() < p1[1].abs() && p1[0].abs() < p1[2].abs() {
                            [0.0, -p1[2], p1[1]]
                        } else if p1[1].abs() < p1[2].abs() {
                            [p1[2], 0.0, -p1[0]]
                        } else {
                            [-p1[1], p1[0], 0.0]
                        };
                        let mag =
                            (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt();
                        [perp[0] / mag, perp[1] / mag, perp[2] / mag]
                    } else {
                        let mag = mag2.sqrt();
                        [cross[0] / mag, cross[1] / mag, cross[2] / mag]
                    };
                    // Remove existing parallel/antiparallel pole
                    if let Some(idx) = points().iter().enumerate().find_map(|(idx, point)| {
                        if great_circles.read().iter().any(|gc| gc.pole == idx) {
                            let dot = normal[0] * point.absolute[0]
                                + normal[1] * point.absolute[1]
                                + normal[2] * point.absolute[2];
                            if (dot.abs() - 1.0).abs() < 1e-6 {
                                Some(idx)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }) {
                        great_circles.write().retain(|gc| gc.pole != idx);
                        return;
                    }
                    let new_point = Point::from_vec3_absolute(points().len(), normal, q);
                    points.write().push(new_point);
                    great_circles
                        .write()
                        .push(GreatCircle::new(points().len() - 1));
                }
                break;
            }
            Key::Character(ref c) if c.as_str() == "," => {
                let selected = s.selected();
                if selected.len() == 3 {
                    let p1 = points()[selected[0]].absolute;
                    let p2 = points()[selected[1]].absolute;
                    let p3 = points()[selected[2]].absolute;
                    let v1 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
                    let v2 = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];
                    let normal = [
                        v1[1] * v2[2] - v1[2] * v2[1],
                        v1[2] * v2[0] - v1[0] * v2[2],
                        v1[0] * v2[1] - v1[1] * v2[0],
                    ];
                    let mag2 =
                        normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
                    if mag2 < 1e-10 {
                        return;
                    }
                    let mag = mag2.sqrt();
                    let mut n = [normal[0] / mag, normal[1] / mag, normal[2] / mag];
                    let dots = [
                        n[0] * p1[0] + n[1] * p1[1] + n[2] * p1[2],
                        n[0] * p2[0] + n[1] * p2[1] + n[2] * p2[2],
                        n[0] * p3[0] + n[1] * p3[1] + n[2] * p3[2],
                    ];
                    if dots.iter().filter(|&&d| d < 0.0).count() >= 2 {
                        n = [-n[0], -n[1], -n[2]];
                    }
                    if let Some(idx) = points().iter().enumerate().find_map(|(idx, point)| {
                        if small_circles.read().iter().any(|sc| sc.pole == idx) {
                            let dot = n[0] * point.absolute[0]
                                + n[1] * point.absolute[1]
                                + n[2] * point.absolute[2];
                            if (dot.abs() - 1.0).abs() < 1e-6 {
                                Some(idx)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }) {
                        small_circles.write().retain(|sc| sc.pole != idx);
                        return;
                    }
                    let plane_distance = p1[0] * n[0] + p1[1] * n[1] + p1[2] * n[2];
                    let new_pole_idx = points().len();
                    let new_pole = Point::from_vec3_absolute(new_pole_idx, n, q);
                    points.write().push(new_pole);
                    small_circles
                        .write()
                        .push(SmallCircle::new(new_pole_idx, plane_distance));
                }
                break;
            }
            Key::Character(ref c) if c.as_str() == "<" => {
                let selected = s.selected();
                if selected.len() == 2 {
                    let pole_idx = selected[0];
                    if small_circles.read().iter().any(|sc| sc.pole == pole_idx) {
                        small_circles.write().retain(|sc| sc.pole != pole_idx);
                    } else {
                        let point_idx = selected[1];
                        let pole = points()[pole_idx].absolute;
                        let point = points()[point_idx].absolute;
                        let plane_distance =
                            pole[0] * point[0] + pole[1] * point[1] + pole[2] * point[2];
                        small_circles
                            .write()
                            .push(SmallCircle::new(pole_idx, plane_distance));
                    }
                }
                break;
            }
            Key::Character(ref c) if c.as_str() == "/" => {
                let new = points()[i].new_inverted(points().len());
                points.write().push(new);
            }
            Key::Character(c) => {
                if let Some(gc) = great_circles.write().iter_mut().find(|x| x.pole == i) {
                    if event.modifiers().shift() {
                        gc.name.push_str(&{
                            let up = c.to_uppercase();
                            if up == c {
                                c.to_lowercase()
                            } else {
                                c.to_uppercase()
                            }
                        });
                        continue;
                    }
                } else if let Some(sc) = small_circles.write().iter_mut().find(|x| x.pole == i) {
                    if event.modifiers().shift() {
                        sc.name.push_str(&{
                            let up = c.to_uppercase();
                            if up == c {
                                c.to_lowercase()
                            } else {
                                c.to_uppercase()
                            }
                        });
                        continue;
                    }
                }
                points.write()[i].name.push_str(&c);
            }
            Key::Backspace => {
                if let Some(gc) = great_circles.write().iter_mut().find(|x| x.pole == i) {
                    if event.modifiers().shift() {
                        gc.name.pop();
                        continue;
                    }
                } else if let Some(sc) = small_circles.write().iter_mut().find(|x| x.pole == i) {
                    if event.modifiers().shift() {
                        sc.name.pop();
                        continue;
                    }
                }
                points.write()[i].name.pop();
            }
            _ => {}
        }
    }
}
