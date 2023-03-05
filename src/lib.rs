use glam::*;
use instant::Instant;
use wgpu::util::DeviceExt;
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
    shader_render_info: ShaderRenderInfo,
    camera_render_info: CameraRenderInfo,
    pub camera: Camera,
    pub prefabs: Vec<Prefab>,
}

pub struct Prefab {
    pub mesh: Mesh,
    pub material: Material,
    pub instances: Vec<Instance>,
    pub instance_buffer: wgpu::Buffer,
}

impl Prefab {
    pub fn new(
        mesh: Mesh,
        material: Material,
        instances: Vec<Instance>,
        device: &wgpu::Device,
    ) -> Self {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            mesh,
            material,
            instances,
            instance_buffer,
        }
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

        // Makin' shaders
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_render_info.bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let shader_render_info = ShaderRenderInfo::new(
            &device,
            wgpu::include_wgsl!("shader.wgsl"),
            config.format,
            &render_pipeline_layout,
        );
        // You could conceivably share pipeline layouts between shaders with similar bind group requirements
        // The bind group layouts dependency here mirrors dependency the bind groups in the render function

        let scene = Scene {
            shader_render_info,
            camera_render_info,
            camera: Camera::default(),
            prefabs: Vec::new(),
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
        self.scene
            .camera_render_info
            .update(&self.scene.camera, &mut self.queue);
        // ^^ todo: move to a scene update(?)
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
                render_pass.set_vertex_buffer(1, prefab.instance_buffer.slice(..));
                render_pass.set_index_buffer(
                    prefab.mesh.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(
                    0..prefab.mesh.index_count,
                    0,
                    0..prefab.instances.len() as _,
                );
                // Using the instance buffer is good for things which have all the same uniform properties
                // but for how Fury prefabs was set up would need to do a different approach (see below)
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        // So I was confused about how to just draw something else again with a different offset, in WebGL you'd just change the MV matrix (camera uniform)
        // and call draw again, but you can't do that with WebGPU, so...
        // okay heres a github question with my use pattern / case - https://github.com/gfx-rs/wgpu-rs/issues/542
        // seems answers followed my line of thinking - seperate bind groups and uniforms, but apparently didn't perform well in the users first attempt
        // so the example of how to do it well was linked as https://github.com/gfx-rs/wgpu-rs/tree/master/examples/shadow
        // a more up-to-date link would be: https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples/shadow
        // which tbf we were aware of the examples we just hadn't searched them
        // ^^ looks like for improved performance you need to use the dynamic offsets within a single large buffer with multiple values in
        // The only real issue is the buffer is statically sized but I suppose we can create a new one in the event of a new object being added.
        // (rather than creating new bind groups and buffer groups for each)
        // tbf the actual render code in that example could have the entities with individual bind groups and buffers, but they happen to be the same buffer, presumably
        // it doesn't cost as much to call set_bind_group with the same bind group but different offset. << Would be good to profile!
        // The example by its nature has all the entities of the same type together, but if you were adding them externally would want to order them by bind group presuming
        // that there is a performance benefit for doing so.
        // Of course you can potentially used draw instanced instead but it'll make shader code considerably more complex

        output.present();

        Ok(())
    }
}

pub trait Game {
    fn init(&mut self, state: &mut State);
    fn update(&mut self, state: &mut State, elapsed: f32);
    fn input(&mut self, state: &mut State, event: &WindowEvent) -> bool;
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn wasm_start() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            log::error!("Unable to run as no game info"); 
        }
    }
    // We have a bit of a problem here, wasm_bindgen(start) seems to only work from within lib.rs
    // but clearly the library isn't the game, previously we just ran run_internal directly
    // but that's when all logic was within lib.rs and main was just a trigger point
    // will moving to examples work? how does bevy do this?
}

pub fn run(game: Box<dyn Game>) {
    pollster::block_on(run_internal(game));
    // consider use of https://docs.rs/tokio or https://docs.rs/async-std over pollster
}

async fn run_internal(mut game: Box<dyn Game>) {
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
