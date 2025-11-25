pub use dioxus::{html::input_data::MouseButton, prelude::*};

pub mod circle;
pub mod event;
pub mod file;
pub mod panels;
pub mod point;

pub use circle::*;
pub use event::*;
pub use file::*;
pub use panels::*;
pub use point::*;

pub const FAVICON: Asset = asset!("/assets/triangle.ico");
pub const MAIN_CSS: Asset = asset!("/assets/main.css");
pub const SAVE: Asset = asset!("/assets/save.ico");
pub const LOAD: Asset = asset!("/assets/load.ico");
pub const NEW_FILE: Asset = asset!("/assets/new.ico");

pub struct State {
    selected: Vec<usize>,
    pub zoom: f64,
    pub rotation: Vec3,
    pub quaternion: Quaternion,
    pub show_grid: bool,
    pub show_hidden: bool,
    pub show_center: bool,
    pub groups: Vec<Vec<usize>>,
}

impl State {
    pub fn initialize() -> Self {
        Self {
            selected: vec![],
            zoom: 1.0,
            rotation: [0.0, 0.0, 0.0],
            quaternion: Quaternion::identity(),
            show_grid: false,
            show_hidden: false,
            show_center: false,
            groups: vec![],
        }
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

    pub fn find_group_containing(&self, point_id: usize) -> Option<usize> {
        self.groups
            .iter()
            .position(|group| group.contains(&point_id))
    }

    pub fn get_group_members(&self, point_id: usize) -> Vec<usize> {
        if let Some(group_idx) = self.find_group_containing(point_id) {
            self.groups[group_idx].clone()
        } else {
            vec![point_id]
        }
    }

    pub fn toggle_select_group(&mut self, multi: bool, id: usize) -> bool {
        let group_members = self.get_group_members(id);

        if multi {
            let all_selected = group_members
                .iter()
                .all(|&member_id| self.selected.contains(&member_id));
            if all_selected {
                for &member_id in &group_members {
                    self.selected.retain(|&x| x != member_id);
                }
                false
            } else {
                for &member_id in &group_members {
                    if !self.selected.contains(&member_id) {
                        self.selected.push(member_id);
                    }
                }
                true
            }
        } else {
            let all_selected = group_members.len() == self.selected.len()
                && group_members
                    .iter()
                    .all(|&member_id| self.selected.contains(&member_id));

            if all_selected {
                self.selected.clear();
                false
            } else {
                self.selected.clear();
                self.selected.extend(group_members);
                true
            }
        }
    }

    pub fn create_group_from_selected(&mut self) {
        if self.selected.len() < 2 {
            return;
        }

        for point_id in self.selected.clone() {
            self.remove_from_group(point_id);
        }

        self.groups.push(self.selected.clone());
    }

    pub fn remove_from_group(&mut self, point_id: usize) {
        if let Some(group_idx) = self.find_group_containing(point_id) {
            self.groups[group_idx].retain(|&id| id != point_id);
            if self.groups[group_idx].len() <= 1 {
                self.groups.remove(group_idx);
            }
        }
    }

    pub fn ungroup_selected(&mut self) {
        for point_id in self.selected.clone() {
            self.remove_from_group(point_id);
        }
    }

    pub fn update_group_indices(&mut self, removed_idx: usize) {
        for group in &mut self.groups {
            group.retain(|&id| id != removed_idx);
            for id in group.iter_mut() {
                if *id > removed_idx {
                    *id -= 1;
                }
            }
        }
        self.groups.retain(|group| group.len() > 1);
    }
}
