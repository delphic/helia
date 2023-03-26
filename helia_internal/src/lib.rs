use glam::*;
use instant::Instant;
use slotmap::SlotMap;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use camera::*;
use entity::*;
use material::*;
use mesh::*;
use scene::*;
use shader::*;

pub mod entity;
pub mod prefab;
pub mod scene;

pub mod camera_controller;

pub mod camera;
pub mod material;
pub mod mesh;
pub mod shader;
pub mod texture;

pub struct Resources {
    pub meshes: SlotMap<MeshId, Mesh>,
    pub materials: SlotMap<MaterialId, Material>,
    pub shaders: SlotMap<ShaderId, Shader>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            meshes: SlotMap::with_key(),
            materials: SlotMap::with_key(),
            shaders: SlotMap::with_key(), 
        }
    }
}

pub struct BuildInShaders {
    pub unlit_textured: ShaderId,
}

pub struct State {
    last_update_time: Instant,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    depth_texture: texture::Texture,
    pub scene: Scene,
    pub resources: Resources,
    pub shaders: BuildInShaders,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window).unwrap() };
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
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync, // May want to auto v-sync
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        // can find valid present modes via: surface.get_supported_modes(&adapter);
        surface.configure(&device, &config);

        let mut resources = Resources::new();

        // Depth Texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let texture_bind_group_layout = Material::create_bind_group_layout(&device);
        let camera_bind_group = CameraBindGroup::new(&device, None);

        let entity_bind_group = EntityBindGroup::new(&device);

        // Makin' shaders
        // Currently 'sprite' shader which is used for everything
        // although more accurately it's just UnlitTextured (w/ tint)
        // but we intend for it to become a sprite!
        let shader = Shader::new(
            &device,
            wgpu::include_wgsl!("shaders/unlit_textured.wgsl"),
            config.format,
            &texture_bind_group_layout,
            &camera_bind_group.layout,
            &entity_bind_group.layout,
        );
        let unlit_textured = resources.shaders.insert(shader);

        let scene = Scene::new(camera_bind_group, entity_bind_group);

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
            resources,
            shaders: BuildInShaders { unlit_textured },
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

        // todo: would prefer camera,render(scene)
        self.scene.render(
            &view,
            &self.depth_texture.view,
            &mut encoder,
            &self.resources,
        );

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
    fn resize(&mut self, state: &mut State);
}

pub async fn run(mut game: Box<dyn Game>) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::builder().filter_level(log::LevelFilter::Warn).init();
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
                        if state.resize(*physical_size) {
                            game.resize(&mut state);
                        }
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        if state.resize(**new_inner_size) {
                            game.resize(&mut state);
                        }
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
                Err(wgpu::SurfaceError::Lost) => {
                    state.resize(state.size);
                },
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
