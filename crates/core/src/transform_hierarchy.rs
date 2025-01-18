use std::collections::HashSet;

use glam::{Mat4, Quat, Vec3};
use slotmap::{self, SecondaryMap, SlotMap};

use crate::transform::Transform;

slotmap::new_key_type! { pub struct HierarchyId; }

pub struct HierarchyNode {
    pub parent: Option<HierarchyId>,
    pub children: Vec<HierarchyId>,
}

/// Stores a hierarchy of transforms and maintains an accurate set of world matrices
/// NOTE: Currently does not protect against circular references
pub struct TransformHierarchy {
    hierarchy: SlotMap<HierarchyId, HierarchyNode>,
    transforms: SecondaryMap<HierarchyId, Transform>,
    world_matrices: SecondaryMap<HierarchyId, Mat4>,
}

impl TransformHierarchy {
    pub fn new() -> Self {
        Self {
            hierarchy: SlotMap::with_key(),
            transforms: SecondaryMap::new(),
            world_matrices: SecondaryMap::new()
        }
    }

    pub fn clear(&mut self) {
        self.hierarchy.clear();
        self.transforms.clear();
        self.world_matrices.clear();
    }

    pub fn insert(&mut self, transform: Transform, parent: Option<HierarchyId>) -> HierarchyId {
        let node = HierarchyNode { parent: parent, children: Vec::new() };
        let hierarchy_id = self.hierarchy.insert(node);
        self.transforms.insert(hierarchy_id, transform);
        self.world_matrices.insert(hierarchy_id, self.get_parent_matrix(parent) * transform.to_local_matrix());
        hierarchy_id
    }

    /// Remove a transform and all it's descendants from the hierarchy
    pub fn remove(&mut self, hierarchy_id: HierarchyId) {
        self.deattach_parent(hierarchy_id);
        if let Some(node) = self.hierarchy.get(hierarchy_id) {
            if node.children.is_empty() {
                self.hierarchy.remove(hierarchy_id);
                self.transforms.remove(hierarchy_id);
                self.world_matrices.remove(hierarchy_id);
            } else {
                let mut to_remove = HashSet::new();
                let mut pending = Vec::new();
                to_remove.insert(hierarchy_id);
                pending.push(node);
                while let Some(node) = pending.pop() {
                    for child in node.children.iter() {
                        if to_remove.insert(*child) {
                            if let Some(node) = self.hierarchy.get(*child) {
                                pending.push(node);
                            }
                        }
                    }
                }
                for id in to_remove.iter() {
                    self.hierarchy.remove(*id);
                    self.transforms.remove(*id);
                    self.world_matrices.remove(*id);
                }
            }
        }
    }

    pub fn parent(&mut self, hierarchy_id: HierarchyId, parent: Option<HierarchyId>) {
        if self.hierarchy.get(hierarchy_id).and_then(|node| node.parent) != parent {
            self.deattach_parent(hierarchy_id);
            if let Some(node) = self.hierarchy.get_mut(hierarchy_id) {
                node.parent = parent;
            }
            self.set_transform(hierarchy_id, self.transforms[hierarchy_id]);
        }
    }

    pub fn get_transform(&self, hierarchy_id: HierarchyId) -> Option<Transform> {
        self.transforms.get(hierarchy_id).copied()
    }

    /// Set transform and update relevant hierarchy world matrices
    pub fn set_transform(&mut self, hierarchy_id: HierarchyId, transform: Transform) {
        self.transforms[hierarchy_id] = transform;
        if let Some(node) = self.hierarchy.get(hierarchy_id) {
            let world_matrix = self.get_parent_matrix(node.parent) * transform.to_local_matrix();
            self.world_matrices[hierarchy_id] = world_matrix;
            Self::update_decendant_matrices(
                hierarchy_id,
                world_matrix,
                &self.hierarchy,
                &self.transforms,
                &mut self.world_matrices);
        } else {
            self.world_matrices[hierarchy_id] = transform.into();
        }
    }

    pub fn get_world_matrix(&self, hierarchy_id: HierarchyId) -> Option<Mat4> {
        self.world_matrices.get(hierarchy_id).copied()
    }

    pub fn get_world_scale_rotation_position(&self, hierarchy_id: HierarchyId) -> Option<(Vec3, Quat, Vec3)> {
        self.get_world_matrix(hierarchy_id).and_then(|matrix| Some(matrix.to_scale_rotation_translation()))
    }

    fn deattach_parent(&mut self, hierarchy_id: HierarchyId) {
        if let Some(node) = self.hierarchy.get(hierarchy_id)
            .and_then(|node| node.parent)
            .and_then(|parent| self.hierarchy.get_mut(parent)) {
            if let Some(index) = node.children.iter().position(|id| *id == hierarchy_id) {
                node.children.remove(index);
            }
        }
    }

    fn get_parent_matrix(&self, parent: Option<HierarchyId>) -> Mat4 {
        if let Some(id) = parent {
            self.world_matrices.get(id).copied().unwrap_or(Mat4::IDENTITY)
        } else {
            Mat4::IDENTITY
        }
    }

    fn update_decendant_matrices(
        hierarchy_id: HierarchyId,
        parent_matrix: Mat4,
        hierarchy: &SlotMap<HierarchyId, HierarchyNode>,
        transforms: &SecondaryMap<HierarchyId, Transform>,
        matrices: &mut SecondaryMap<HierarchyId, Mat4>) {
        if let Some(node) = hierarchy.get(hierarchy_id) {
            for child in node.children.iter() {
                // Could avoid the need to store transforms if we did inver_previous_parent_matrix * parent_matrix * local
                // However this could potentially introduce drift from floating point precision
                let world_matrix = parent_matrix * transforms[*child].to_local_matrix();
                matrices[*child] = world_matrix;
                // BUG: This will stack overflow if you create a circular loop of transforms
                Self::update_decendant_matrices(*child, world_matrix, hierarchy, transforms, matrices);
            }
        }
    }
}