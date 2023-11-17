mod mouse;

use ears_rs::parser::EarsParser;
use image::{ImageFormat, RgbaImage};
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
use wgpu::{Backends, BlendState, CompositeAlphaMode, Limits, RequestDeviceError};
use winit::{
    event_loop::EventLoop,
    platform::web::WindowBuilderExtWebSys,
    window::WindowBuilder,
};

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
    pub has_ears: bool,
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
            has_ears: false,
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

    let skin_image = image::load_from_memory_with_format(&skin, ImageFormat::Png)?.into_rgba8();

    let ears_features = EarsParser::parse(&skin_image)?.filter(|_| model.has_ears);

    let mut part_context: PlayerPartProviderContext<()> = PlayerPartProviderContext {
        model: if model.is_slim {
            PlayerModel::Alex
        } else {
            PlayerModel::Steve
        },
        has_hat_layer: model.has_hat_layer,
        has_layers: model.has_layers,
        has_cape: ears_features.is_some_and(|f| f.cape_enabled),
        arm_rotation: 10.0,
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features,
    };

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

    add_scene_texture(
        &mut scene,
        &mut part_context,
        PlayerPartTextureType::Skin,
        skin_image,
        true,
    )?;

    unsafe {
        SCENE.replace(scene);
    }

    Ok(())
}

fn add_scene_texture(
    scene: &mut Scene,
    part_context: &mut PlayerPartProviderContext,
    texture_type: PlayerPartTextureType,
    mut texture: RgbaImage,
    do_ears_processing: bool,
) -> JsResult<()> {
    if do_ears_processing {
        {
            use ears_rs::alfalfa::AlfalfaDataKey;
            use nmsr_rendering::high_level::parts::provider::ears::PlayerPartEarsTextureType;

            if texture_type == PlayerPartTextureType::Skin {
                if let Ok(Some(alfalfa)) = ears_rs::alfalfa::read_alfalfa(&texture) {
                    if let Some(wings) = alfalfa.get_data(AlfalfaDataKey::Wings) {
                        add_scene_texture(
                            scene,
                            part_context,
                            PlayerPartEarsTextureType::Wings.into(),
                            image::load_from_memory(wings)
                                .expect_throw("Failed to load wings")
                                .to_rgba8(),
                            false,
                        )?;
                    }

                    if let Some(cape) = alfalfa.get_data(AlfalfaDataKey::Cape) {
                        add_scene_texture(
                            scene,
                            part_context,
                            PlayerPartEarsTextureType::Cape.into(),
                            image::load_from_memory(cape)
                                .expect_throw("Failed to load cape")
                                .to_rgba8(),
                            true,
                        )?;
                    }
                }

                ears_rs::utils::process_erase_regions(&mut texture)?;
            } else if texture_type == PlayerPartEarsTextureType::Cape.into() {
                texture = ears_rs::utils::convert_ears_cape_to_mojang_cape(texture);
            }
        }

        if texture_type == PlayerPartTextureType::Skin {
            ears_rs::utils::strip_alpha(&mut texture);
        } else if texture_type == PlayerPartTextureType::Cape {
            part_context.has_cape = true;
        }
    }
    scene.set_texture(
        graphics_context().expect_throw("Context"),
        texture_type,
        &texture,
    );
    scene.rebuild_parts(&part_context, PlayerBodyPartType::iter().collect());

    Ok(())
}

#[wasm_bindgen]
pub fn get_camera() -> SceneCameraSettings {
    let camera = scene().expect_throw("Scene not initialized").camera_mut();

    let CameraRotation { yaw, pitch, roll } = camera.get_rotation();

    SceneCameraSettings {
        distance: camera.get_distance(),
        rotation: WasmVec3(yaw, pitch, roll),
        size: WasmVec2(0., 0.),
        look_at: WasmVec3(0., 0., 0.),
    }
}

#[wasm_bindgen]
pub fn get_sun() -> SceneLightingSettings {
    let sun = scene()
        .expect_throw("Scene not initialized")
        .sun_information_mut();

    SceneLightingSettings {
        direction: WasmVec3(sun.direction.x, sun.direction.y, sun.direction.z),
        intensity: sun.intensity,
        ambient: sun.ambient,
    }
}

#[wasm_bindgen]
pub fn set_camera_rotation(yaw: f32, pitch: f32, roll: f32) {
    if let Some(scene) = scene() {
        scene
            .camera_mut()
            .set_rotation(CameraRotation { yaw, pitch, roll });
        scene.update(graphics_context().expect_throw("Graphics context not initialized"));
    }
}

#[wasm_bindgen]
pub async fn run_event_loop(canvas: HtmlCanvasElement, size: WasmVec2) -> JsResult<()> {
    let size = winit::dpi::PhysicalSize::new(size.0, size.1);
    
    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_inner_size(size)
        .with_canvas(Some(canvas))
        .with_prevent_default(true)
        .with_resizable(false)
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
                winit::event::WindowEvent::MouseInput { state, .. } => {
                    if state == winit::event::ElementState::Pressed {
                        mouse::handle_mouse_down();
                    } else {
                        mouse::handle_mouse_up();
                    }
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    if let Some(scene) = scene() {
                        if let Some(context) = graphics_context() {
                            mouse::handle_mouse_move(
                                scene,
                                context,
                                position.x as f32,
                                position.y as f32,
                            );
                        }
                    }
                }
                winit::event::WindowEvent::MouseWheel { delta, .. } => {
                    if let Some(scene) = scene() {
                        if let Some(context) = graphics_context() {
                            let delta = match delta {
                                winit::event::MouseScrollDelta::LineDelta(_, delta) => delta,
                                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                                    delta.y as f32 / 50.0
                                }
                            };

                            mouse::handle_mouse_scroll(scene, context, delta);
                        }
                    }
                }
                winit::event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    // Revert the scale factor change
                    new_inner_size.width = size.width as u32;
                    new_inner_size.height = size.height as u32;
                    
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
                        .unwrap();
                }
            }
            _ => {}
        }
    });
}

#[wasm_bindgen]
pub async fn initialize(canvas: HtmlCanvasElement, width: u32, height: u32) -> JsResult<()> {
    console_error_panic_hook::set_once();

    let canvas = SendWrapper::new(canvas);

    #[cfg(feature = "webgl")]
    let backend = Backends::GL;
    #[cfg(not(feature = "webgl"))]
    let backend = Backends::BROWSER_WEBGPU;

    #[cfg(not(feature = "webgl"))]
    let limits = Limits::downlevel_defaults();
    #[cfg(feature = "webgl")]
    let limits = Limits::downlevel_webgl2_defaults();

    let mut context = GraphicsContext::new(GraphicsContextDescriptor {
        backends: Some(backend),
        surface_provider: Box::new(|i| {
            Some(
                i.create_surface_from_canvas(canvas.take())
                    .expect_throw("Failed to create surface from Canvas"),
            )
        }),
        default_size: (width, height),
        texture_format: Some(wgpu::TextureFormat::Rgba8Unorm),
        features: Features::empty(),
        limits: Some(limits),
        blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        sample_count: Some(1),
        use_smaa: Some(false),
    })
    .await?;

    {
        if let Ok(config_option) = context.surface_config.as_mut() {
            if let Some(config) = config_option.as_mut() {
                #[cfg(not(feature = "webgl"))]
                let alpha_mode = CompositeAlphaMode::PreMultiplied;
                #[cfg(feature = "webgl")]
                let alpha_mode = CompositeAlphaMode::Opaque;

                config.alpha_mode = alpha_mode;

                if let Some(surface) = context.surface.as_mut() {
                    surface.configure(&context.device, config);
                }
            }
        }
    }

    unsafe {
        GRAPHICS_CONTEXT.replace(context);
    }

    Ok(())
}
