use glam::*;
use instant::Instant;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use crate::camera::*;
use crate::camera_controller::*;
use crate::material::*;
use crate::mesh::*;
use crate::shader::*;

mod camera;
mod camera_controller;
mod material;
mod mesh;
mod shader;
mod texture;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vec3 = Vec3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

struct State {
    last_update_time: Instant,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    depth_texture: texture::Texture,
    scene: Scene,
    camera_controller: Option<CameraController>, // this should be in game
}

struct Scene {
    shader_render_info: ShaderRenderInfo,
    camera_render_info: CameraRenderInfo,
    camera: Camera,
    prefabs: Vec<Prefab>
}

struct Prefab {
    mesh: Mesh,
    material: Material,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

// okay so resource id for mesh and material
// then render object (?) with mesh, and material id
// and an instance
// we have a list of render objects, nope this doesn't map well to instances
// we want a 'prefab'ish concept. which can store instances as well as the mesh / material
// data

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

        // This the initial wait so arguably this is where the first callback should happen, although I think we probably want
        // the surface and depth texture

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

        // This is probably where we want to actually do our first callback - we have a device, surface, queue etc

        // This should be in game code, setup our window and surface now we want to load our texture
        // setup our camera et al

        // Arguably we could set up the camera bind group (we're defo going to need one) and maybe the shader

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

                // Makin' Camera
        let camera = Camera {
            eye: (-0.5, 1.0, 2.0).into(),
            target: (-0.5, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: config.width as f32 / config.height as f32,
            fov: 45.0 * std::f32::consts::PI / 180.0,
            near: 0.1,
            far: 100.0,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        };
        // ^^ todo: Camera::Default instead and move this to game

        let mut scene = Scene {
            shader_render_info,
            camera_render_info,
            camera,
            prefabs: Vec::new()
        };

        // Here is the final point we could do the call back
        // Makin' Textures
        let diffuse_bytes = include_bytes!("../assets/lena.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "lena.png").unwrap();

        let material = Material::new(diffuse_texture, &texture_bind_group_layout, &device);
        // ^^ arguably material should contain a link to the shader it executes (an id)


        // TODO: Going to probably want to convert this to position / rotation for our sanity :P
        let mesh = Mesh::new(VERTICES, INDICES, &device);

        // prefab / scene
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = Vec3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position == Vec3::ZERO {
                        Quat::from_axis_angle(Vec3::Z, 0.0)
                    } else {
                        Quat::from_axis_angle(
                            position.normalize(),
                            45.0 * std::f32::consts::PI / 180.0,
                        )
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let prefab = Prefab {
            mesh,
            material,
            instances,
            instance_buffer,
        };

        scene.prefabs.push(prefab);

        Self {
            last_update_time: Instant::now(),
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            scene,
            camera_controller: Some(CameraController::new(1.5)),
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

    fn input(&mut self, event: &WindowEvent) -> bool {
        if let Some(camera_controller) = &mut self.camera_controller {
            camera_controller.process_events(event);
        }
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.scene.camera.clear_color = wgpu::Color {
                    r: position.x / self.size.width as f64,
                    g: 0.2,
                    b: position.y / self.size.height as f64,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, elapsed: f32) {
        if let Some(camera_controller) = &self.camera_controller {
            camera_controller
                .update_camera(&mut self.scene.camera, elapsed);
        }
        self.scene.camera_render_info
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
                render_pass
                    .set_index_buffer(prefab.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..prefab.mesh.index_count, 0, 0..prefab.instances.len() as _);
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

pub fn run() {
    pollster::block_on(run_internal());
    // Q: how does macroquad manage to make main async?
    // consider use of https://docs.rs/tokio or https://docs.rs/async-std over pollster
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run_internal() {
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

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
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
            let elapsed = state.last_update_time.elapsed();
            state.update(elapsed.as_secs_f32());
            state.last_update_time = Instant::now();
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
