use glam::*;
use instant::Instant;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use crate::camera::*;
use crate::material::*;
use crate::mesh::*;
use crate::shader::*;


pub mod camera_controller; 

pub mod camera;
pub mod material;
pub mod mesh;
pub mod shader;
pub mod texture;

pub struct State {
    last_update_time: Instant,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    depth_texture: texture::Texture,
    pub scene: Scene,
    pub texture_bind_group_layout: wgpu::BindGroupLayout, // this shouldn't be public
}

pub struct Scene {
    shader_render_info: ShaderRenderInfo, // this feels like renderer / context internal state
    camera_render_info: CameraRenderInfo, // this feels like renderer / context internal state
    pub camera: Camera,
    pub prefabs: Vec<Prefab>,

    #[allow(dead_code)]
    entity_bind_group_layout: wgpu::BindGroupLayout,
    entity_bind_group: wgpu::BindGroup,
    entity_uniforms_buffer: wgpu::Buffer, // only applies to sprites atm
    entity_uniforms_alignment: wgpu::BufferAddress,
}

// Currently only applies to sprites, and has to be used with prefabs
// todo: option to provide your own mesh / maerial data 
struct Entity {
    transform: glam::Mat4,
    color: wgpu::Color,
    uniform_offset: usize, // needs to be converted into a wgpu::DynamnicOffset based on uniform_size / spacing
}

pub struct Prefab {
    pub mesh: Mesh,
    pub material: Material,
    entities: Vec<Entity>,
}

impl Prefab {
    pub fn new(
        mesh: Mesh,
        material: Material,
    ) -> Self {        
        Self {
            mesh,
            material,
            entities: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, transform: glam::Mat4, color: wgpu::Color) {
        self.entities.push(Entity {
            transform,
            color,
            uniform_offset: self.entities.len()
        });
    }
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync, // May want to auto v-sync
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        // can find valid present modes via: surface.get_supported_modes(&adapter);
        surface.configure(&device, &config);

        // Depth Texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let texture_bind_group_layout = Material::create_bind_group_layout(&device);

        let camera_render_info = CameraRenderInfo::new(&device, None);

        // todo: 'render_info' for entity - need bind group layout and bind group
        // we make the buffer below and I think we dynamically make the uniform data every frame
        // (as we have to pack it into a buffer w/ offsets)
        let entity_uniform_size = std::mem::size_of::<EntityUniforms>() as wgpu::BufferAddress;
        let entity_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(entity_uniform_size),
                },
                count: None,
            }],
            label: None,
        });

        // Makin' shaders
        // note this pipeline layout is specific per shader (although could potentially be shared)
        // in that the bind group layouts have to match the @group declarations in the shader
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_render_info.bind_group_layout,
                    &entity_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let shader_render_info = ShaderRenderInfo::new(
            &device,
            wgpu::include_wgsl!("shaders/sprite.wgsl"),
            config.format,
            &render_pipeline_layout,
        );
        // You could conceivably share pipeline layouts between shaders with similar bind group requirements
        // The bind group layouts dependency here mirrors dependency the bind groups in the render function

        // Make a buffer for potential entities
        // and store the uniform aligment as we'll need it
        let num_entities = 128 as wgpu::BufferAddress;
        // Make the `uniform_alignment` >= `entity_uniform_size` and aligned to `min_uniform_buffer_offset_alignment`.
        let entity_uniforms_alignment = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            wgpu::util::align_to(entity_uniform_size, alignment)
        };
        // Note: dynamic uniform offsets also have to be aligned to `Limits::min_uniform_buffer_offset_alignment`.
        let entity_uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: num_entities * entity_uniforms_alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });


        let entity_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &entity_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &entity_uniforms_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(entity_uniform_size),
                }),
            }],
            label: None,
        });

        let scene = Scene {
            shader_render_info,
            camera_render_info,
            camera: Camera::default(),
            prefabs: Vec::new(),
            // entity 'render info'
            entity_bind_group_layout,
            entity_bind_group,
            entity_uniforms_buffer,
            entity_uniforms_alignment,
        };

        Self {
            last_update_time: Instant::now(),
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            scene,
            texture_bind_group_layout,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn update(&mut self, _elapsed: f32) {
        // arguably this is all currently scene.update
        self.scene
            .camera_render_info
            .update(&self.scene.camera, &mut self.queue);
    
        let mut running_offset : usize = 0;
        for prefab in self.scene.prefabs.iter() {
            for entity in prefab.entities.iter() {
                let data = EntityUniforms {
                    model: entity.transform.to_cols_array_2d(),
                    color: [
                        entity.color.r as f32,
                        entity.color.g as f32,
                        entity.color.b as f32,
                        entity.color.a as f32,
                    ],
                };
                let offset = (entity.uniform_offset + running_offset) as u64 * self.scene.entity_uniforms_alignment;
                self.queue.write_buffer(
                    &self.scene.entity_uniforms_buffer,
                    offset as wgpu::BufferAddress,
                    bytemuck::bytes_of(&data),
                );
            }
            running_offset += prefab.entities.len();
        }    
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let camera = &self.scene.camera;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(camera.clear_color),
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            let mut running_offset = 0;
            for prefab in self.scene.prefabs.iter() {
                // todo: only do this if there are any instances

                let shader = &self.scene.shader_render_info;
                // want to move this to something the shader does
                render_pass.set_pipeline(&shader.render_pipeline);
                // ^^ todo: move to material

                render_pass.set_bind_group(0, &prefab.material.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.scene.camera_render_info.bind_group, &[]);
                // Q: How do we coordinate what bind groups to set when the bind groups themselves aren't per shader?
                // but the locations are

                render_pass.set_vertex_buffer(0, prefab.mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    prefab.mesh.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );

                // using uniform with offset approach of
                // https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples/shadow
                for entity in prefab.entities.iter() {
                    let offset = (entity.uniform_offset + running_offset) as u64 * self.scene.entity_uniforms_alignment;
                    render_pass.set_bind_group(2, &self.scene.entity_bind_group, &[offset as wgpu::DynamicOffset]);
                    render_pass.draw_indexed(0..prefab.mesh.index_count as u32, 0, 0..1);
                }
                running_offset += prefab.entities.len();
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}

pub trait Game {
    fn init(&mut self, state: &mut State);
    fn update(&mut self, state: &mut State, elapsed: f32);
    fn input(&mut self, state: &mut State, event: &WindowEvent) -> bool;
}

pub async fn run(mut game: Box<dyn Game>) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Helia")
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have
        // to set the size manually when on the web
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("helia")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await;

    game.init(&mut state);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !game.input(&mut state, event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let elapsed = state.last_update_time.elapsed().as_secs_f32();
            state.last_update_time = Instant::now();
            game.update(&mut state, elapsed);
            state.update(elapsed);

            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}
