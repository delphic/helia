use std::{collections::HashMap, sync::Arc};

use entity::{Entity, InstanceProperties};
use glam::*;
use slotmap::SlotMap;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::*, event_loop::{EventLoop, EventLoopProxy}, keyboard::{KeyCode, PhysicalKey}, window::Window
};

use material::*;
use mesh::*;
use scene::*;
use shader::*;
use texture::*;

pub type Color = wgpu::Color;

pub mod entity;
pub mod game_object;
pub mod input;
pub mod prefab;
pub mod scene;
pub mod time;
pub mod transform;
pub mod transform_hierarchy;

pub mod orbit_camera;

pub mod atlas;
pub mod camera;
pub mod material;
pub mod mesh;
pub mod shader;
pub mod texture;

pub struct Resources {
    pub meshes: SlotMap<MeshId, Mesh>,
    pub materials: SlotMap<MaterialId, Material>,
    pub shaders: SlotMap<ShaderId, Shader>,
    pub textures: SlotMap<TextureId, Texture>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            meshes: SlotMap::with_key(),
            materials: SlotMap::with_key(),
            shaders: SlotMap::with_key(),
            textures: SlotMap::with_key(),
        }
    }
}

pub struct BuildInShaders {
    pub unlit_textured: ShaderId,
    pub sprite: ShaderId,
}

pub struct State {
    pub camera: camera::Camera,
    pub time: time::Time,
    surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    depth_texture: texture::Texture,
    pub input: input::InputState,
    pub scene: Scene,
    pub resources: Resources,
    pub shaders: BuildInShaders,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    pub window: Arc<Window>,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Arc<Window>, size: PhysicalSize<u32>) -> Self {
        // The instance is a handle to our GPU
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        log::info!("{:?}", surface);
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
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::downlevel_defaults()
                    },
                    label: None,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync, // May want to auto v-sync
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 1, // 2 is default
        };
        // can find valid present modes via: surface.get_supported_modes(&adapter);
        surface.configure(&device, &config);

        let mut resources = Resources::new();

        // Depth Texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let texture_bind_group_layout = Material::create_bind_group_layout(&device);

        // Makin' shaders
        let shader = Shader::new(
            &device,
            wgpu::include_wgsl!("shaders/unlit_textured.wgsl"),
            config.format,
            &texture_bind_group_layout,
            false,
            std::mem::size_of::<EntityUniforms>(),
            EntityUniforms::write_bytes,
        );
        let unlit_textured = resources.shaders.insert(shader);

        let sprite_shader = Shader::new(
            &device,
            wgpu::include_wgsl!("shaders/unlit_textured.wgsl"),
            config.format,
            &texture_bind_group_layout,
            true,
            std::mem::size_of::<EntityUniforms>(),
            EntityUniforms::write_bytes,
        );
        let sprite = resources.shaders.insert(sprite_shader);

        let scene = Scene::new();

        Self {
            camera: camera::Camera::default(),
            time: time::Time::default(),
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            scene,
            texture_bind_group_layout,
            resources,
            input: input::InputState::default(),
            shaders: BuildInShaders {
                unlit_textured,
                sprite,
            },
            window,
        }
    }

    // HACK: ideally wouldn't have to have an accessor like this, could probably
    // 'fix' this by having a renderer module, which has methods for creating texture bindgroups
    // may also sort itself out once we remove the bind group from the public Material struct
    pub fn get_texture_bind_group_layout_ref(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            return true;
        }
        false
    }

    fn update(&mut self) {
        // TODO: Make examples responsible for updating the scene themselves 
        self.scene.update(&self.camera, &self.resources);
    }

    fn render(&mut self, draw_commands: &Vec<DrawCommand>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        let mut entities = Vec::new();
        let mut entity_count_by_shader = HashMap::<ShaderId, u64>::new();
        for command in draw_commands.iter() {
            match command {
                DrawCommand::Draw(
                    mesh,
                    material,
                    transform) => {
                    let entity = Entity::new(
                        *mesh,
                        *material,
                        InstanceProperties::builder().with_matrix((*transform).into()).build());
                    
                    let shader = self.resources.materials.get(*material).unwrap().shader;
                    if let Some(count) = entity_count_by_shader.get(&shader) {
                        entity_count_by_shader.insert(shader, count + 1);
                    } else {
                        entity_count_by_shader.insert(shader, 1);
                    }
                    entities.push(entity);
                },
                DrawCommand::DrawPrefab(prefab_id, transform) => {
                    if let Some(prefab) = self.scene.prefabs.get(*prefab_id) {
                        let shader = self.resources.materials.get(prefab.material).unwrap().shader;
                        if let Some(count) = entity_count_by_shader.get(&shader) {
                            entity_count_by_shader.insert(shader, count + 1);
                        } else {
                            entity_count_by_shader.insert(shader, 1);
                        }
                        let entity = Entity::new(
                            prefab.mesh,
                            prefab.material,
                            InstanceProperties::builder().with_matrix((*transform).into()).build());
                        entities.push(entity);
                    }
                },
                DrawCommand::DrawEntity(entity) => {
                    let shader = self.resources.materials.get(entity.material).unwrap().shader;
                    if let Some(count) = entity_count_by_shader.get(&shader) {
                        entity_count_by_shader.insert(shader, count + 1);
                    } else {
                        entity_count_by_shader.insert(shader, 1);
                    }
                    entities.push(*entity);
                }
            }
        }
        
        for (shader_id, entity_count) in entity_count_by_shader.iter() {
            let shader = &mut self.resources.shaders[*shader_id];

            shader.reset_offset();
            // NOTE: camera dependency, see the render pass for more details
            shader.camera_bind_group.update(&self.camera, &self.queue);

            // Ensure sufficient capacity in each shader to be used for entity uniform data
            let capacity = shader.entity_bind_group.entity_capacity;
            if capacity < 2 * entity_count {
                let mut target_capacity = 2 * capacity;
                while target_capacity < 2 * entity_count {
                    target_capacity *= 2;
                }
                shader
                    .entity_bind_group
                    .recreate_entity_buffer(target_capacity, &self.device);
            }
        }

        // Write instance properties to shader
        for entity in entities.iter_mut() {
           let shader_id = self.resources.materials.get(entity.material).unwrap().shader; 
           self.resources.shaders[shader_id].write_entity_uniforms(entity, &self.queue);
        }
        // When we're copying all this entity data around, I'm not sure how much we care about this mut passing

        // This was scene render, but then that was pointless if we want to be able to mix and match draw commands
        // (though entites was a loop over the scene graph)
        // Adding scope so render pass is dropped when done
        {
            let camera = &self.camera;
            let view = &view;
            let depth_view = &self.depth_texture.view;
            // ^^ Arguably we don't need this and the attachment it's used in if we're rendering 2D
            // I guess the question is, are these separate render passes? 
            let resources = &self.resources;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(camera.clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            let mut currently_bound_shader_id: Option<ShaderId> = None;
            let mut currently_bound_mesh_id: Option<MeshId> = None;
            let mut currently_bound_material_id: Option<MaterialId> = None;

            for entity in entities.iter() {
                let mesh = &resources.meshes[entity.mesh];
                let material = &resources.materials[entity.material];
                let shader = &resources.shaders[material.shader];

                let entity_bind_group = &shader.entity_bind_group.bind_group;

                if currently_bound_material_id != Some(entity.material) {
                    currently_bound_material_id = Some(entity.material);

                    if currently_bound_shader_id != Some(material.shader) {
                        currently_bound_shader_id = Some(material.shader);
                        render_pass.set_pipeline(&shader.render_pipeline);
                        render_pass.set_bind_group(0, &shader.camera_bind_group.bind_group, &[]); 
                        // TODO: Should be asking shader for camera_bind_group for a particular camera
                        // This would require the shader to have an updated bind_group / buffer & uniform
                        // for each camera which it needs to render for
                    }

                    render_pass.set_bind_group(2, &material.diffuse_bind_group, &[]);
                    // We're presumably going to share the layout for textures across shaders
                    // therefore we can and should share texture bind groups across materials
                    // only rebind when appropriate, rather than rebinding per material
                    // however should only do this if we're bothering to order the scene graph
                    // to group materials with the same textures
                }

                if currently_bound_mesh_id != Some(entity.mesh) {
                    currently_bound_mesh_id = Some(entity.mesh);

                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                }

                // using uniform with offset approach of
                // https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples/shadow
                render_pass.set_bind_group(
                    1,
                    entity_bind_group,
                    &[entity.uniform_offset as wgpu::DynamicOffset],
                );
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}

// Consider implementing Drop for State 
// https://github.com/sotrh/learn-wgpu/issues/549#issuecomment-2445330937

// App and enum to support flow necessary to create
// window for both native and WASM export  
enum UserEvent {
    StateReady(State),
}

struct App {
    title: String,
    resizable: bool,
    window_size: PhysicalSize<u32>,
    state: Option<State>,
    event_loop_proxy: EventLoopProxy<UserEvent>,
    game: Box<dyn Game>,
}

impl App {
    fn new(
        game: Box<dyn Game>,
        title: String,
        resizable: bool,
        window_size: PhysicalSize<u32>,
        event_loop: &EventLoop<UserEvent>) -> Self {
        Self {
            game,
            title,
            resizable,
            window_size,
            state: None,
            event_loop_proxy: event_loop.create_proxy(),
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(
            Window::default_attributes().with_title(self.title.clone())
                .with_resizable(self.resizable)
                .with_inner_size(self.window_size)
            ).ok().unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("helia")?;
                    let canvas = window.canvas()?;
                    canvas.set_width(self.window_size.width);
                    canvas.set_height(self.window_size.height);
                    let canvas = web_sys::Element::from(canvas);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
            
            let state_future = State::new(Arc::new(window), self.window_size);
            let event_loop_proxy = self.event_loop_proxy.clone();
            let future = async move {
                let state = state_future.await;
                assert!(event_loop_proxy.send_event(UserEvent::StateReady(state)).is_ok());
            };
            wasm_bindgen_futures::spawn_local(future);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let state = pollster::block_on(State::new(Arc::new(window), self.window_size));
            assert!(self.event_loop_proxy.send_event(UserEvent::StateReady(state)).is_ok());
        }
    }

    fn user_event(&mut self, _: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        let UserEvent::StateReady(mut state) = event;
        self.game.init(&mut state);
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut state) = self.state else {
            return;
        };

        if window_id != state.window.id() {
            return;
        }

        state.input.process_events(&event);

        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                if state.resize(physical_size) {
                    self.game.resize(state);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // This used to resize as per resize but it no longer contains "new_inner_size",
                // although the documentation still refers to it
            }
            WindowEvent::RedrawRequested => {
                let elapsed = state.time.update();
                self.game.update(state, elapsed);
                state.update();
                state.input.frame_finished();

                let mut draw_commands = Vec::new(); // probably don't want a new one each frame but hey prototyping
                self.game.render(&mut draw_commands, state);

                match state.render(&draw_commands) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        state.resize(state.size);
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        };
    }

    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        if let Some(ref state) = self.state {
            state.window.request_redraw();
        }
    }
}

pub enum DrawCommand {
    Draw(MeshId, MaterialId, transform::Transform),
    DrawPrefab(prefab::PrefabId, transform::Transform),
    DrawEntity(Entity),
}
// TODO: ^^ would be good to support multiple cameras - but if we do that we want to only pass a cameraId rather than copying around all the data

pub trait Game {
    fn init(&mut self, state: &mut State);
    fn update(&mut self, state: &mut State, elapsed: f32);
    fn render(&mut self, commands: &mut Vec<DrawCommand>, state: &mut State); // Use of State is temporary whilst we decouple state.scene
    fn resize(&mut self, state: &mut State);
}

pub struct Helia {
    title: String,
    resizable: bool,
    window_size: PhysicalSize<u32>,
}

impl Helia {
    pub fn new() -> Self {
        Self {
            title: "Helia".to_string(),
            resizable: false,
            window_size: PhysicalSize::new(960, 540),
        }
    }

    // Note: if we want to support full_screen, then need to detect
    // resolution of the monitor and size the surface accoridngly

    pub fn with_title<T: Into<String>>(&mut self, title: T) -> &mut Self {
        self.title = title.into();
        self
    }

    pub fn with_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.window_size = PhysicalSize::new(width, height);
        self
    }

    pub fn with_resizable(&mut self, resizable: bool) -> &mut Self {
        self.resizable = resizable;
        self
    }

    pub async fn run(&self, game: Box<dyn Game>) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
            } else {
                env_logger::builder().filter(Some("wgpu"), log::LevelFilter::Warn).filter_level(log::LevelFilter::Info).init();
            }
        }

        let event_loop = EventLoop::<UserEvent>::with_user_event().build().ok().unwrap();
        // Consider ControlFlow::Poll and not using about_to_wait in AppHandler 
        // c.f. https://github.com/sotrh/learn-wgpu/issues/549#issuecomment-2570248027

        let mut app = App::new(game, self.title.clone(), self.resizable, self.window_size, &event_loop);
        event_loop.run_app(&mut app).ok();

        // Consider EventLoopExtWebSys::spawn_app for WASM to avoid exception
    }
}
