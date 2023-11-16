use ears_rs::parser::EarsParser;
use image::ImageFormat;
use js_utils::JsResult;
use nmsr_player_parts::{
    model::PlayerModel,
    parts::provider::PlayerPartProviderContext,
    types::{PlayerBodyPartType, PlayerPartTextureType},
    IntoEnumIterator,
};
use nmsr_rendering::{
    high_level::{
        camera::{Camera, CameraRotation, ProjectionParameters},
        pipeline::{
            scene::{Scene, Size, SunInformation},
            Features, GraphicsContext, GraphicsContextDescriptor, SceneContext,
            SceneContextWrapper,
        },
    },
    low_level::Vec3,
};
use send_wrapper::SendWrapper;
use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};
use web_sys::{console, HtmlCanvasElement};
use wgpu::{Backends, BlendState, Limits, CompositeAlphaMode};
use winit::{event_loop::EventLoop, platform::web::WindowBuilderExtWebSys, window::WindowBuilder};

static mut GRAPHICS_CONTEXT: Option<GraphicsContext> = None;
static mut SCENE: Option<Scene<SceneContextWrapper>> = None;

fn graphics_context() -> Option<&'static GraphicsContext> {
    unsafe { GRAPHICS_CONTEXT.as_ref() }
}

fn scene() -> Option<&'static mut Scene<SceneContextWrapper>> {
    unsafe { SCENE.as_mut() }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmVec2(pub f32, pub f32);

#[wasm_bindgen]
impl WasmVec2 {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32) -> Self {
        Self(x, y)
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct WasmVec3(pub f32, pub f32, pub f32);

#[wasm_bindgen]
impl WasmVec3 {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(x, y, z)
    }
}

impl From<WasmVec2> for (f32, f32) {
    fn from(WasmVec2(x, y): WasmVec2) -> Self {
        (x, y)
    }
}

impl From<WasmVec3> for Vec3 {
    fn from(WasmVec3(x, y, z): WasmVec3) -> Self {
        Vec3::from([x, y, z])
    }
}

#[wasm_bindgen]
pub struct SceneCameraSettings {
    pub size: WasmVec2,
    pub look_at: WasmVec3,
    pub distance: f32,
    pub rotation: WasmVec3,
}

#[wasm_bindgen]
impl SceneCameraSettings {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            size: WasmVec2(0.0, 0.0),
            look_at: WasmVec3(0.0, 0.0, 0.0),
            distance: 0.0,
            rotation: WasmVec3(0.0, 0.0, 0.0),
        }
    }
}

#[wasm_bindgen]
pub struct SceneLightingSettings {
    pub direction: WasmVec3,
    pub intensity: f32,
    pub ambient: f32,
}

#[wasm_bindgen]
impl SceneLightingSettings {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            direction: WasmVec3(0.0, 0.0, 0.0),
            intensity: 0.0,
            ambient: 0.0,
        }
    }
}

#[wasm_bindgen]
pub struct SceneCharacterSettings {
    pub is_slim: bool,
    pub has_hat_layer: bool,
    pub has_layers: bool,
}

#[wasm_bindgen]
impl SceneCharacterSettings {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            is_slim: false,
            has_hat_layer: false,
            has_layers: false,
        }
    }
}

#[wasm_bindgen]
pub async fn setup_scene(
    settings: SceneCameraSettings,
    light: SceneLightingSettings,
    model: SceneCharacterSettings,
    skin: Vec<u8>,
) -> JsResult<()> {
    let SceneCameraSettings {
        size: WasmVec2(width, height),
        look_at,
        distance,
        rotation: WasmVec3(yaw, pitch, roll),
    } = settings;

    let SceneLightingSettings {
        direction,
        intensity,
        ambient,
    } = light;

    let size = Size {
        width: width as u32,
        height: height as u32,
    };

    let graphics_context = graphics_context().expect_throw("Graphics context not initialized");
    let scene_context = SceneContext::new(graphics_context);

    let camera = Camera::new_orbital(
        look_at.into(),
        distance,
        CameraRotation { yaw, pitch, roll },
        ProjectionParameters::Perspective { fov: 45.0 },
        Some(size),
    );

    let lighting = SunInformation::new(direction.into(), intensity, ambient);

    let mut part_context: PlayerPartProviderContext<()> = PlayerPartProviderContext {
        model: if model.is_slim {
            PlayerModel::Alex
        } else {
            PlayerModel::Steve
        },
        has_hat_layer: model.has_hat_layer,
        has_layers: model.has_layers,
        has_cape: false,
        arm_rotation: 10.0,
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features: None,
    };
    
    let mut skin_image = image::load_from_memory_with_format(&skin, ImageFormat::Png)?.into_rgba8();

    part_context.ears_features = EarsParser::parse(&skin_image)?;
    
    ears_rs::utils::process_erase_regions(&mut skin_image)?;
    ears_rs::utils::strip_alpha(&mut skin_image);
    
    let parts: Vec<_> = PlayerBodyPartType::iter().into_iter().collect();

    let mut scene: Scene<SceneContextWrapper> = Scene::new(
        graphics_context,
        scene_context.into(),
        camera,
        lighting,
        size,
        &part_context,
        &parts,
    );
    
    scene.set_texture(graphics_context, PlayerPartTextureType::Skin, &skin_image);

    unsafe {
        SCENE.replace(scene);
    }

    Ok(())
}

#[wasm_bindgen]
pub fn set_camera_rotation(yaw: f32, pitch: f32, roll: f32) {
    if let Some(scene) = scene() {
        scene.camera_mut().set_rotation(CameraRotation { yaw, pitch, roll });
        scene.update(graphics_context().expect_throw("Graphics context not initialized"));
    }
}

#[wasm_bindgen]
pub async fn run_event_loop(canvas: HtmlCanvasElement, size: WasmVec2) -> JsResult<()> {
    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_inner_size(winit::dpi::LogicalSize::new(size.0, size.1))
        .with_canvas(Some(canvas))
        .with_focusable(true)
        .with_decorations(false);
    
    let event_loop = EventLoop::new();
    let window = window.build(&event_loop)?;
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => {}
            },
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                if let Some(scene) = scene() {
                    scene
                        .render(graphics_context().expect_throw("Graphics context not initialized"))
                        .inspect_err(inspect_err)
                        .unwrap();
                }
            }
            _ => {}
        }
    });
}

#[wasm_bindgen]
pub async fn initialize(canvas: HtmlCanvasElement) -> JsResult<()> {
    console_error_panic_hook::set_once();
    let canvas = SendWrapper::new(canvas);

    let mut context = GraphicsContext::new(GraphicsContextDescriptor {
        backends: Some(Backends::BROWSER_WEBGPU),
        surface_provider: Box::new(|i| {
            Some(
                i.create_surface_from_canvas(canvas.take())
                    .expect_throw("Failed to create surface from Canvas"),
            )
        }),
        default_size: (512, 512),
        texture_format: Some(wgpu::TextureFormat::Rgba8Unorm),
        features: Features::empty(),
        limits: Some(Limits::downlevel_defaults()),
        blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        sample_count: Some(1),
        use_smaa: Some(false),
    })
    .await?;

    if let Ok(config_option) = context.surface_config.as_mut() {
        if let Some(config) = config_option.as_mut() {
            config.alpha_mode = CompositeAlphaMode::PreMultiplied;
            
            if let Some(surface) = context.surface.as_mut() {
                surface.configure(&context.device, config);
            }
        }
    }
    
    unsafe {
        GRAPHICS_CONTEXT.replace(context);
    }

    Ok(())
}

fn inspect_err<T: std::fmt::Display>(err: &T) {
    unsafe {
        console::error_1(&format!("{}", err).into());
    }
}
fn log_dbg<T: std::fmt::Debug>(err: &T) {
    unsafe {
        console::error_1(&format!("{:?}", err).into());
    }
}
