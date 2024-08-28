use std::sync::Arc;

use glam::Vec3A;
use image::RgbaImage;
use nmsr_rendering::{
    errors::NMSRRenderingError,
    high_level::{
        model::ArmorMaterial,
        parts::{
            part::Part,
            provider::{PartsProvider, PlayerPartProviderContext, PlayerPartsProvider},
        },
        types::{PlayerBodyPartType, PlayerPartTextureType},
        utils::parts::primitive_convert,
    },
    low_level::primitives::mesh::{Mesh, PrimitiveDispatch},
};
use std::fmt::Debug;

use nmsr_rasterizer_test::{
    camera::Camera,
    model::{RenderEntry, Size},
    shader::{ShaderState, SunInformation},
};

use crate::nmsr_rendering_compat::high_level::pipeline::GraphicsContext;

pub struct Scene<M: ArmorMaterial> {
    camera: Camera,
    lighting: SunInformation,
    size: Size,
    entry: RenderEntry,
    parts: Vec<PlayerBodyPartType>,
    shader_states: Vec<ShaderState>,
    phantom: std::marker::PhantomData<M>,
}

impl<M: ArmorMaterial> Scene<M> {
    pub fn get_size(&self) -> Size {
        self.size
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn sun_information_mut(&mut self) -> &mut SunInformation {
        &mut self.lighting
    }
}

impl<M: ArmorMaterial + Debug> Debug for Scene<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("camera", &self.camera)
            .field("lighting", &self.lighting)
            .field("size", &self.size)
            .field("entry", &self.entry)
            .field("parts", &self.parts)
            .field("shader_states", &self.shader_states)
            .finish()
    }
}

impl<M: ArmorMaterial + Debug> Scene<M> {
    pub fn new(
        mut camera: Camera,
        lighting: SunInformation,
        size: Size,
        parts: &[PlayerBodyPartType],
    ) -> Self {
        if let None = camera.get_size() {
            camera.set_size(Some(size));
        }

        Self {
            camera,
            lighting,
            size,
            entry: RenderEntry::new(camera.get_size().unwrap()),
            parts: parts.to_vec(),
            shader_states: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn set_texture(
        &mut self,
        texture: PlayerPartTextureType,
        image: Arc<RgbaImage>,
        parts_context: &PlayerPartProviderContext<M>,
    ) {
        let providers = [
            PlayerPartsProvider::Minecraft,
            #[cfg(feature = "ears")]
            PlayerPartsProvider::Ears,
        ];

        let parts = providers
            .iter()
            .flat_map(|provider| {
                self.parts
                    .iter()
                    .flat_map(|part| provider.get_parts(parts_context, *part))
            })
            .filter(|p| p.get_texture() == texture)
            .collect::<Vec<Part>>();

        let parts = parts
            .into_iter()
            .map(|p| primitive_convert(&p))
            .collect::<Vec<_>>();

        let value = ShaderState::new_with_primitive( 
            self.camera,
            image,
            texture.is_emissive(),
            if texture.is_emissive() {
                SunInformation::new(Vec3A::Y, 1.0, 1.0)
            } else {
                self.lighting
            },
            PrimitiveDispatch::Mesh(Mesh::new(parts)),
        );
        
        self.shader_states.push(value);
    }

    pub fn update(&mut self, _ctx: &GraphicsContext) {
        for state in &mut self.shader_states {
            state.camera = self.camera;
            state.sun = if state.emissive_texture { 
                SunInformation::new(Vec3A::Y, 1.0, 1.0)
            } else {
                self.lighting
            };
            
            state.update();
        }
    }

    pub fn render(&mut self, _ctx: &GraphicsContext) -> Result<(), NMSRRenderingError> {
        self.entry.textures.clear_depth();
        self.entry.textures.output.fill(0);

        for state in &mut self.shader_states {
            self.entry.draw(state);
        }

        Ok(())
    }

    pub fn copy_output_texture(&self) -> &[u8] {
        let size = self.size;
        &self
            .entry
            .textures
            .output
            .get(0..((size.width * size.height * 4) as usize))
            .expect("Failed to copy output texture")
    }
}
