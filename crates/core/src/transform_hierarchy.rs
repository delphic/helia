use std::{collections::HashSet, hash::Hash};

use glam::{Mat4, Quat, Vec3};
use slotmap::{self, SecondaryMap, SlotMap};

use crate::transform::Transform;

slotmap::new_key_type! { pub struct TransformId; }

pub struct HierarchyNode {
    pub parent: Option<TransformId>,
    pub children: Vec<TransformId>,
}

/// Stores a hierarchy of transforms and maintains an accurate set of world matrices
/// NOTE: Does not prevent circular references on insertion
pub struct TransformHierarchy {
    hierarchy: SlotMap<TransformId, HierarchyNode>,
    transforms: SecondaryMap<TransformId, Transform>,
    world_matrices: SecondaryMap<TransformId, Mat4>,
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

    pub fn insert(&mut self, transform: Transform, parent: Option<TransformId>) -> TransformId {
        let node = HierarchyNode { parent: parent, children: Vec::new() };
        let hierarchy_id = self.hierarchy.insert(node);
        self.transforms.insert(hierarchy_id, transform);
        self.world_matrices.insert(hierarchy_id, self.get_parent_matrix(parent) * transform.to_local_matrix());
        hierarchy_id
    }

    /// Remove a transform and all it's descendants from the hierarchy
    pub fn remove(&mut self, id: TransformId) {
        self.deattach_parent(id);
        if let Some(node) = self.hierarchy.get(id) {
            if node.children.is_empty() {
                self.hierarchy.remove(id);
                self.transforms.remove(id);
                self.world_matrices.remove(id);
            } else {
                let mut to_remove = HashSet::new();
                let mut pending = Vec::new();
                to_remove.insert(id);
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

    pub fn parent(&mut self, id: TransformId, parent: Option<TransformId>) {
        if self.hierarchy.get(id).and_then(|node| node.parent) != parent {
            self.deattach_parent(id);
            if let Some(node) = self.hierarchy.get_mut(id) {
                node.parent = parent;
            }
            self.set_transform(id, self.transforms[id]);
        }
    }

    pub fn get_transform(&self, id: TransformId) -> Option<Transform> {
        self.transforms.get(id).copied()
    }

    /// Set transform and update relevant hierarchy world matrices
    pub fn set_transform(&mut self, id: TransformId, transform: Transform) {
        self.transforms[id] = transform;
        if let Some(node) = self.hierarchy.get(id) {
            let world_matrix = self.get_parent_matrix(node.parent) * transform.to_local_matrix();
            self.world_matrices[id] = world_matrix;
            if !node.children.is_empty() {
                let mut touched = HashSet::new();
                touched.insert(id);
                Self::update_decendant_matrices(
                    id,
                    world_matrix,
                    &self.hierarchy,
                    &self.transforms,
                    &mut self.world_matrices,
                    &mut touched);
            }

        } else {
            self.world_matrices[id] = transform.into();
        }
    }

    pub fn get_world_matrix(&self, id: TransformId) -> Option<Mat4> {
        self.world_matrices.get(id).copied()
    }

    pub fn get_world_scale_rotation_position(&self, id: TransformId) -> Option<(Vec3, Quat, Vec3)> {
        self.get_world_matrix(id).and_then(|matrix| Some(matrix.to_scale_rotation_translation()))
    }

    fn deattach_parent(&mut self, id: TransformId) {
        if let Some(parent_node) = self.hierarchy.get(id)
            .and_then(|node| node.parent)
            .and_then(|parent| self.hierarchy.get_mut(parent)) {
            if let Some(index) = parent_node.children.iter().position(|child_id| *child_id == id) {
                parent_node.children.remove(index);
            }
        }
    }

    fn get_parent_matrix(&self, parent: Option<TransformId>) -> Mat4 {
        if let Some(id) = parent {
            self.world_matrices.get(id).copied().unwrap_or(Mat4::IDENTITY)
        } else {
            Mat4::IDENTITY
        }
    }

    fn update_decendant_matrices(
        id: TransformId,
        parent_matrix: Mat4,
        hierarchy: &SlotMap<TransformId, HierarchyNode>,
        transforms: &SecondaryMap<TransformId, Transform>,
        matrices: &mut SecondaryMap<TransformId, Mat4>,
        touched: &mut HashSet<TransformId>,
    ) {
        if let Some(node) = hierarchy.get(id) {
            for child in node.children.iter() {
                if touched.insert(*child) {
                    // Could avoid the need to store transforms if we did inver_previous_parent_matrix * parent_matrix * local
                    // However this could potentially introduce drift from floating point precision
                    let world_matrix = parent_matrix * transforms[*child].to_local_matrix();
                    matrices[*child] = world_matrix;
                    Self::update_decendant_matrices(*child, world_matrix, hierarchy, transforms, matrices, touched);
                } else {
                    log::warn!("Cyclical transform hierarchy detected {child:?} already touched");
                }
            }
        }
    }
}