use wgpu::util::DeviceExt;
use wgpu::Buffer;

use crate::shader::Vertex;

slotmap::new_key_type! { pub struct MeshId; }

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(vertices: &[Vertex], indices: &[u16], device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn from_arrays(positions: &[glam::Vec3], uvs: &[glam::Vec2], indicies: &[u16], device: &wgpu::Device) -> Self {
        let mut vertices = Vec::new();
        for i in 0..positions.len() {
            vertices.push(Vertex {
                position: positions[i].to_array(),
                tex_coords: uvs[i].to_array(),
            });
        }
        Mesh::new(vertices.as_slice(), indicies, &device)
    }
    // todo: generic on Vertex type
}
