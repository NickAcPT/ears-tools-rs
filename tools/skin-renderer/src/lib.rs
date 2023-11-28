mod mouse;

use ears_rs::parser::EarsParser;
use glam::{Vec3, Vec3A};
use image::{ImageFormat, RgbaImage};
use js_utils::JsResult;
use nmsr_player_parts::{
    model::PlayerModel,
    parts::provider::{ears::PlayerPartEarsTextureType, PlayerPartProviderContext},
    types::PlayerPartTextureType,
};
use nmsr_rasterizer_test::{
    camera::{Camera, CameraRotation, ProjectionParameters, self},
    model::{RenderEntry, Size},
    shader::{ShaderState, SunInformation},
};
use send_wrapper::SendWrapper;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue, UnwrapThrowExt, closure::Closure};
use wasm_bindgen_futures::js_sys::Object;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

static mut RENDER_ENTRY: Option<RenderEntry> = None;
static mut SCENE: Option<ShaderState> = None;
static mut CANVAS: Option<SendWrapper<CanvasRenderingContext2d>> = None;

fn scene() -> Option<&'static ShaderState> {
    unsafe { SCENE.as_ref() }
}

fn scene_mut() -> Option<&'static mut ShaderState> {
    unsafe { SCENE.as_mut() }
}

fn render_entry() -> Option<&'static mut RenderEntry> {
    unsafe { RENDER_ENTRY.as_mut() }
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

impl From<WasmVec3> for Vec3A {
    fn from(WasmVec3(x, y, z): WasmVec3) -> Self {
        Vec3A::from([x, y, z])
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

    let mut camera = Camera::new_orbital(
        look_at.into(),
        distance,
        CameraRotation { yaw, pitch, roll },
        ProjectionParameters::Perspective { fov: 45.0 },
        Some(size),
    );

    let lighting = SunInformation {
        direction: direction.into(),
        intensity,
        ambient,
    };

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
        has_cape: ears_features.is_some_and(|f| f.cape_enabled && model.has_cape),
        arm_rotation: 10.0,
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features,
    };

    let scene_context = RenderEntry::new(size, &part_context);

    let scene = add_scene_texture(
        &mut camera,
        lighting,
        &mut part_context,
        PlayerPartTextureType::Skin,
        skin_image,
        model.has_ears,
        &model,
    )?;

    unsafe {
        SCENE.replace(scene);
        RENDER_ENTRY.replace(scene_context);
    }

    Ok(())
}

fn add_scene_texture(
    camera: &mut Camera,
    lighting: SunInformation,
    part_context: &mut PlayerPartProviderContext,
    texture_type: PlayerPartTextureType,
    mut texture: RgbaImage,
    do_ears_processing: bool,
    model: &SceneCharacterSettings,
) -> JsResult<ShaderState> {
    if do_ears_processing {
            use ears_rs::alfalfa::AlfalfaDataKey;

        if texture_type == PlayerPartTextureType::Skin {
            if let Ok(Some(alfalfa)) = ears_rs::alfalfa::read_alfalfa(&texture) {
                if let Some(wings) = alfalfa.get_data(AlfalfaDataKey::Wings) {
                    add_scene_texture(
                        camera,
                        lighting,
                        part_context,
                        PlayerPartEarsTextureType::Wings.into(),
                        image::load_from_memory(wings)
                            .expect_throw("Failed to load wings")
                            .to_rgba8(),
                        false,
                        model,
                    )?;
                }

                if let Some(cape) = alfalfa.get_data(AlfalfaDataKey::Cape) {
                    add_scene_texture(
                        camera,
                        lighting,
                        part_context,
                        PlayerPartEarsTextureType::Cape.into(),
                        image::load_from_memory(cape)
                            .expect_throw("Failed to load cape")
                            .to_rgba8(),
                        true,
                        model,
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
        part_context.has_cape = model.has_cape;
    }

    Ok(ShaderState::new(*camera, texture, lighting))
}

#[wasm_bindgen]
pub fn get_camera() -> SceneCameraSettings {
    let camera = scene().expect_throw("Scene not initialized").camera;
    
    //let mut binding = camera.clone();
    //let look_at = binding.get_look_at_mut().expect("Failed to get look at");

    let CameraRotation { yaw, pitch, roll } = camera.get_rotation();

    SceneCameraSettings {
        distance: camera.get_distance().unwrap_or(45.0),
        rotation: WasmVec3(*yaw, *pitch, *roll),
        size: WasmVec2(0., 0.),
        look_at: WasmVec3(0., 0., 0.),
    }
}

#[wasm_bindgen]
pub fn get_sun() -> SceneLightingSettings {
    let sun = &scene_mut().expect_throw("Scene not initialized").sun;

    SceneLightingSettings {
        direction: WasmVec3(sun.direction.x, sun.direction.y, sun.direction.z),
        intensity: sun.intensity,
        ambient: sun.ambient,
    }
}

#[wasm_bindgen]
pub fn set_camera_rotation(yaw: f32, pitch: f32, roll: f32) {
    if let Some(scene) = scene_mut() {
        let rotation = scene.camera.get_rotation_mut();
        
        *rotation = CameraRotation {
            yaw,
            pitch,
            roll,
        };
        
        scene.update();
        
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
    if let (Some(scene), Some(ctx)) = (scene_mut(), render_entry()) {
        mouse::handle_mouse_move(scene, ctx, x, y);
    }
}

#[wasm_bindgen]
pub async fn notify_mouse_scroll(delta: f32) {
    if let (Some(scene), Some(ctx)) = (scene_mut(), render_entry()) {
        mouse::handle_mouse_scroll(scene, ctx, delta);
    }
}

#[wasm_bindgen]
pub fn render_frame() -> JsResult<()> {
    if let (Some(scene), Some(ctx)) = (scene_mut(), render_entry()) {
        ctx.textures.depth_buffer.fill(1.0);
        ctx.textures.output.fill(0);

        ctx.draw(scene);

        let context = unsafe { CANVAS.as_ref().expect_throw("Canvas not initialized") };

        let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(&ctx.textures.output.as_raw()),
            ctx.size.width,
            ctx.size.height,
        )
        .expect("Failed to create image data");

        context
            .put_image_data(&image_data, 0 as f64, 0 as f64)
            .expect("Failed to put image data");
    }

    Ok(())
}

#[wasm_bindgen]
pub async fn initialize(canvas: HtmlCanvasElement, width: u32, height: u32) -> JsResult<()> {
    console_error_panic_hook::set_once();

    unsafe {
        let context: Result<Option<Object>, JsValue> = canvas.get_context("2d");
        let context = context.expect("Failed to get context");
        let context = context.expect("Failed to get context");
        let context: CanvasRenderingContext2d = context
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("Failed to get context");

        CANVAS.replace(SendWrapper::new(context));
    }

    Ok(())
}
