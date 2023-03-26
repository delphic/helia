use crate::entity::*;
use crate::material::*;
use crate::mesh::*;

slotmap::new_key_type! { pub struct PrefabId; }

pub struct Prefab {
    pub mesh: MeshId,
    pub material: MaterialId,
    pub instances: Vec<EntityId>,
}

impl Prefab {
    pub fn new(mesh: MeshId, material: MaterialId) -> Self {
        Self {
            mesh,
            material,
            instances: Vec::new(),
        }
    }
}
