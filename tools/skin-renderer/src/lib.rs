mod mouse;
#[cfg(feature = "software-rendering")]
mod nmsr_rendering_compat;

#[cfg(feature = "software-rendering")]
use nmsr_rendering_compat as nmsr_rendering;
#[cfg(feature = "software-rendering")]
use send_wrapper::SendWrapper;
#[cfg(feature = "software-rendering")]
use std::sync::Arc;

use glam::Vec3A;

use ears_rs::{
    alfalfa::read_alfalfa,
    features::{self, EarsFeatures},
    parser::EarsParser,
};
use image::{ImageFormat, RgbaImage};
use js_utils::JsResult;
use nmsr_player_parts::{
    model::PlayerModel,
    parts::{part, provider::PlayerPartProviderContext},
    types::{PlayerBodyPartType, PlayerPartTextureType},
    IntoEnumIterator,
};

#[cfg(not(feature = "software-rendering"))]
use nmsr_rendering::high_level::pipeline::{
    Features, GraphicsContext, GraphicsContextDescriptor, SceneContext, SceneContextWrapper,
};
use nmsr_rendering::{
    high_level::{
        camera::{Camera, CameraRotation, ProjectionParameters},
        pipeline::scene::{Scene, Size, SunInformation},
    },
    low_level::Vec3,
};

use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};
use web_sys::HtmlCanvasElement;
#[cfg(not(feature = "software-rendering"))]
use wgpu::{Backends, BlendState, CompositeAlphaMode, Limits};

#[cfg(all(
    not(feature = "webgl"),
    not(feature = "webgpu"),
    not(feature = "software-rendering")
))]
compile_error!("At least one of the following features must be enabled: 'webgl', 'webgpu', 'software-rendering'");

#[cfg(not(feature = "software-rendering"))]
type SceneType = Scene<SceneContextWrapper>;
#[cfg(feature = "software-rendering")]
type SceneType = Scene<()>;

#[cfg(not(feature = "software-rendering"))]
static mut GRAPHICS_CONTEXT: Option<GraphicsContext> = None;
#[cfg(not(feature = "software-rendering"))]
fn graphics_context() -> Option<&'static GraphicsContext<'static>> {
    unsafe { GRAPHICS_CONTEXT.as_ref() }
}

#[cfg(feature = "software-rendering")]
static mut CANVAS: Option<SendWrapper<web_sys::CanvasRenderingContext2d>> = None;
#[cfg(feature = "software-rendering")]
fn canvas() -> Option<&'static SendWrapper<web_sys::CanvasRenderingContext2d>> {
    unsafe { CANVAS.as_ref() }
}

#[cfg(feature = "software-rendering")]
fn graphics_context() -> Option<&'static ()> {
    Some(&())
}

static mut SCENE: Option<SceneType> = None;
fn scene() -> Option<&'static mut SceneType> {
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

impl From<WasmVec3> for Vec3A {
    fn from(WasmVec3(x, y, z): WasmVec3) -> Self {
        Vec3A::from([x, y, z])
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
    pub has_cape: bool,
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
            has_cape: false,
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
    #[cfg(not(feature = "software-rendering"))]
    let graphics_context = graphics_context().expect_throw("Graphics context not initialized");
    #[cfg(not(feature = "software-rendering"))]
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
        has_deadmau5_ears: false,
        is_flipped_upside_down: false,
        has_cape: ears_features.is_some_and(|f| f.cape_enabled && model.has_cape),
        arm_rotation: 10.0,
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features,
    };

    let parts: Vec<_> = PlayerBodyPartType::iter().into_iter().collect();

    cleanup_invalid_ears_data(&skin_image, &mut part_context)?;

    let mut scene: SceneType = SceneType::new(
        #[cfg(not(feature = "software-rendering"))]
        graphics_context,
        #[cfg(not(feature = "software-rendering"))]
        scene_context.into(),
        camera,
        lighting,
        size,
        #[cfg(not(feature = "software-rendering"))]
        &part_context,
        &parts,
    );

    add_scene_texture(
        &mut scene,
        &mut part_context,
        PlayerPartTextureType::Skin,
        skin_image,
        model.has_ears,
        &model,
    )?;

    unsafe {
        SCENE.replace(scene);
    }

    Ok(())
}

fn cleanup_invalid_ears_data(
    skin_image: &RgbaImage,
    part_context: &mut PlayerPartProviderContext<()>,
) -> JsResult<()> {
    if let Some(features) = part_context.ears_features.as_mut() {
        let alfalfa = read_alfalfa(&skin_image)?;

        // If features has wings but the alfalfa data does not contain wings, remove the wings
        if features.wing.is_some()
            && alfalfa.as_ref()
                .and_then(|a| a.get_data(ears_rs::alfalfa::AlfalfaDataKey::Wings))
                .is_none()
        {
            features.wing.take();
        }
        
        
        // If features has cape but the alfalfa data does not contain cape, remove the cape
        if features.cape_enabled
            && alfalfa.as_ref()
                .and_then(|a| a.get_data(ears_rs::alfalfa::AlfalfaDataKey::Cape))
                .is_none()
        {
            features.cape_enabled = false;
            part_context.has_cape = false;
        }
        
        // If features has emissives, but palette is empty, remove the emissives
        
        if features.emissive
            && ears_rs::utils::extract_emissive_palette(&skin_image)?.is_none()
        {
            features.emissive = false;
        }
    }

    Ok(())
}

fn add_scene_texture(
    scene: &mut SceneType,
    part_context: &mut PlayerPartProviderContext,
    texture_type: PlayerPartTextureType,
    mut texture: RgbaImage,
    do_ears_processing: bool,
    model: &SceneCharacterSettings,
) -> JsResult<()> {
    if do_ears_processing {
        {
            use ears_rs::alfalfa::AlfalfaDataKey;
            use nmsr_rendering::high_level::parts::provider::ears::PlayerPartEarsTextureType;

            if texture_type == PlayerPartTextureType::Skin {
                let emissive_palette = ears_rs::utils::extract_emissive_palette(&texture);

                if let Ok(Some(alfalfa)) = ears_rs::alfalfa::read_alfalfa(&texture) {
                    if let Some(wings) = alfalfa.get_data(AlfalfaDataKey::Wings) {
                        let mut wings_texture = image::load_from_memory(wings)
                            .expect_throw("Failed to load wings")
                            .to_rgba8();

                        if let Ok(Some(ref emissive_palette)) = emissive_palette {
                            let emissive_wings = ears_rs::utils::apply_emissive_palette(
                                &mut wings_texture,
                                &emissive_palette,
                            );

                            if let Ok(emissive_wings) = emissive_wings {
                                add_scene_texture(
                                    scene,
                                    part_context,
                                    PlayerPartEarsTextureType::EmissiveWings.into(),
                                    emissive_wings,
                                    false,
                                    model,
                                )?;
                            }
                        }

                        add_scene_texture(
                            scene,
                            part_context,
                            PlayerPartEarsTextureType::Wings.into(),
                            wings_texture,
                            false,
                            model,
                        )?;
                    } else {
                        // Clean-up invalid state with missing data for wings
                        // This is a workaround.
                        if let Some(features) = part_context.ears_features.as_mut() {
                            features.wing.take();
                        }
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
                            model,
                        )?;
                    } else {
                        // Clean-up invalid state with missing data for cape
                        // This is a workaround.
                        if let Some(features) = part_context.ears_features.as_mut() {
                            features.cape_enabled = false;
                        }
                    }
                } else {
                    // Clean-up invalid state with missing alfalfa data for wings and cape
                    // No alfalfa means that there is no wings and/or cape
                    // This is a workaround.
                    if let Some(features) = part_context.ears_features.as_mut() {
                        features.wing.take();
                        features.cape_enabled = false;
                    }
                }

                ears_rs::utils::process_erase_regions(&mut texture)?;

                if let Ok(Some(ref emissive_palette)) = &emissive_palette {
                    if let Ok(emissive_skin) =
                        ears_rs::utils::apply_emissive_palette(&mut texture, &emissive_palette)
                    {
                        add_scene_texture(
                            scene,
                            part_context,
                            PlayerPartEarsTextureType::EmissiveSkin.into(),
                            emissive_skin,
                            false,
                            model,
                        )?;
                    }
                }
            } else if texture_type == PlayerPartEarsTextureType::Cape.into() {
                texture = ears_rs::utils::convert_ears_cape_to_mojang_cape(texture);
            }
        }
    }

    if texture_type == PlayerPartTextureType::Skin {
        ears_rs::utils::strip_alpha(&mut texture);
    } else if texture_type == PlayerPartTextureType::Cape {
        part_context.has_cape = model.has_cape;
    }

    #[cfg(feature = "software-rendering")]
    let texture = Arc::new(texture);
    #[cfg(not(feature = "software-rendering"))]
    let texture = &texture;

    scene.set_texture(
        #[cfg(not(feature = "software-rendering"))]
        graphics_context().expect_throw("Context"),
        texture_type,
        texture,
        #[cfg(feature = "software-rendering")]
        &part_context,
    );

    #[cfg(not(feature = "software-rendering"))]
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
    if let (Some(scene), Some(ctx)) = (scene(), graphics_context()) {
        scene
            .camera_mut()
            .set_rotation(CameraRotation { yaw, pitch, roll });

        scene.update(
            #[cfg(not(feature = "software-rendering"))]
            ctx,
        );
    }
}

#[wasm_bindgen]
pub async fn notify_mouse_down() {
    mouse::handle_mouse_down();
}

#[wasm_bindgen]
pub async fn notify_mouse_up() {
    mouse::handle_mouse_up();
}

#[wasm_bindgen]
pub async fn notify_mouse_move(x: f32, y: f32) {
    if let (Some(scene), Some(ctx)) = (scene(), graphics_context()) {
        mouse::handle_mouse_move(
            scene,
            #[cfg(not(feature = "software-rendering"))]
            ctx,
            x,
            y,
        );
    }
}

#[wasm_bindgen]
pub async fn notify_mouse_scroll(delta: f32) {
    if let (Some(scene), Some(ctx)) = (scene(), graphics_context()) {
        mouse::handle_mouse_scroll(
            scene,
            #[cfg(not(feature = "software-rendering"))]
            ctx,
            delta,
        );
    }
}

#[wasm_bindgen]
pub async fn render_frame() -> JsResult<()> {
    if let (Some(scene), Some(ctx)) = (scene(), graphics_context()) {
        scene.render(
            #[cfg(not(feature = "software-rendering"))]
            ctx,
        )?;

        #[cfg(feature = "software-rendering")]
        {
            let size = scene.get_size();

            if let Some(context) = canvas() {
                let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                    wasm_bindgen::Clamped(scene.copy_output_texture()),
                    size.width,
                    size.height,
                )
                .expect("Failed to create image data");

                context
                    .put_image_data(&image_data, 0 as f64, 0 as f64)
                    .expect("Failed to put image data");
            }
        }
    }

    Ok(())
}

#[cfg(feature = "software-rendering")]
#[wasm_bindgen]
pub async fn initialize(canvas: HtmlCanvasElement, width: u32, height: u32) -> JsResult<()> {
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::js_sys::Object;
    use web_sys::CanvasRenderingContext2d;

    console_error_panic_hook::set_once();

    canvas.set_width(width);
    canvas.set_height(height);

    unsafe {
        let context: Result<Option<Object>, JsValue> = canvas.get_context("2d");
        let context = context.expect("Failed to get context");
        let context = context.expect("Failed to get context");
        let context: CanvasRenderingContext2d =
            JsCast::dyn_into::<CanvasRenderingContext2d>(context).expect("Failed to get context");

        CANVAS.replace(SendWrapper::new(context));
    }

    Ok(())
}

#[cfg(any(feature = "webgl", feature = "webgpu"))]
#[wasm_bindgen]
pub async fn initialize(canvas: HtmlCanvasElement, width: u32, height: u32) -> JsResult<()> {
    use send_wrapper::SendWrapper;
    use wasm_bindgen::JsError;
    use wgpu::SurfaceTarget;

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
        surface_provider: Box::new(|i| i.create_surface(SurfaceTarget::Canvas(canvas.take())).ok()),
        default_size: (width, height),
        texture_format: Some(wgpu::TextureFormat::Rgba8Unorm),
        features: Features::empty(),
        limits: Some(limits),
        blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        sample_count: Some(1),
        use_smaa: Some(false),
    })
    .await?;

    if context.surface.is_none() {
        return Err(JsError::new("Failed to create surface from canvas"));
    }

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
