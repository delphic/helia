use crate::entity::*;
use crate::mesh::*;
use crate::material::*;

slotmap::new_key_type! { pub struct PrefabId; }

pub struct Prefab {
    pub mesh: Mesh,
    pub material: Material,
    pub instances: Vec<Entity>,
}

impl Prefab {
    pub fn new(
        mesh: Mesh,
        material: Material,
    ) -> Self {        
        Self {
            mesh,
            material,
            instances: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, transform: glam::Mat4, color: wgpu::Color) {
        self.instances.push(Entity {
            transform,
            color,
            mesh: None,
            material: None,
        });
    }
}