use glam::*;
use instant::Instant;
use slotmap::DenseSlotMap;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use camera::*;
use material::*;
use shader::*;
use entity::*;
use scene::*;

pub mod entity;
pub mod prefab;
pub mod scene;

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
    texture_bind_group_layout: wgpu::BindGroupLayout,
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
        let camera_bind_group = CameraBindGroup::new(&device, None);

        let entity_bind_group = EntityBindGroup::new(&device);

        // Makin' shaders
        // Currently 'sprite' shader which is used for everything
        let shader_render_pipeline = ShaderRenderPipeline::new(
            &device,
            wgpu::include_wgsl!("shaders/sprite.wgsl"),
            config.format,
            &texture_bind_group_layout,
            &camera_bind_group.layout,
            &entity_bind_group.layout,
        );

        let scene = Scene {
            shader_render_pipeline,
            camera_bind_group,
            camera: Camera::default(),
            prefabs: DenseSlotMap::with_key(),
            entity_bind_group,
            entity_count: 0,
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

    // HACK: ideally wouldn't have to have an accessor like this, could probably
    // 'fix' this by having a renderer module, which has methods for creating texture bindgroups
    // may also sort itself out once we remove the bind group from the public Material struct
    pub fn get_texture_bind_group_layout_ref(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
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

    fn update(&mut self, elapsed: f32) {
        self.scene.update(elapsed, &self.queue, &self.device);
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
            let entity_aligment = self.scene.entity_bind_group.alignment;
            let entity_bind_group = &self.scene.entity_bind_group.bind_group;
            for prefab in self.scene.prefabs.values() {
                if prefab.entities.is_empty() {
                    continue;
                }

                let shader = &self.scene.shader_render_pipeline;
                // want to move this to something the shader does
                render_pass.set_pipeline(&shader.render_pipeline);
                // ^^ todo: move to material

                render_pass.set_bind_group(0, &prefab.material.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.scene.camera_bind_group.bind_group, &[]);
                // Q: How do we coordinate what bind groups to set when the bind groups themselves aren't per shader?
                // but the locations are

                render_pass.set_vertex_buffer(0, prefab.mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    prefab.mesh.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );

                // using uniform with offset approach of
                // https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples/shadow
                for i in 0..prefab.entities.len() {
                    let offset = (i + running_offset) as u64 * entity_aligment;
                    render_pass.set_bind_group(2, entity_bind_group, &[offset as wgpu::DynamicOffset]);
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
        window.set_inner_size(PhysicalSize::new(960, 540));

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
