use std::{collections::HashMap, iter::repeat};

use ears_rs::{
    features::{
        data::{
            ear::{EarAnchor, EarMode},
            snout::SnoutData,
            tail::{TailData, TailMode},
        },
        EarsFeatures,
    },
    parser::EarsFeaturesWriter,
};
use glam::{Affine3A, IVec2, Vec3};
use hsl::HSL;
use image::{Rgba, RgbaImage};
use itertools::Itertools;
use js_utils::JsResult;
use nmsr_player_parts::{
    parts::{
        part::Part,
        provider::{
            ears::PlayerPartEarsTextureType, PartsProvider, PlayerPartProviderContext,
            PlayerPartsProvider,
        },
        uv::{FaceUv, FaceUvPoint},
    },
    types::{PlayerBodyPartType, PlayerPartTextureType},
    IntoEnumIterator,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FaceOrientation {
    Up,
    Down,
    North,
    South,
    East,
    West,
}

impl FaceOrientation {
    /// The player faces -Z (North). -X is West (left), +X is East (right), +Y is Up.
    fn from_normal(normal: Vec3) -> Self {
        let abs = normal.abs();

        if abs.y >= abs.x && abs.y >= abs.z {
            if normal.y > 0.0 {
                Self::Up
            } else {
                Self::Down
            }
        } else if abs.z >= abs.x {
            if normal.z < 0.0 {
                Self::North
            } else {
                Self::South
            }
        } else if normal.x > 0.0 {
            Self::East
        } else {
            Self::West
        }
    }

    /// World direction pointing to the top of the face when looking at it, matching
    /// how cube faces are laid out (the front edge sits at the bottom of the top face
    /// and at the top of the bottom face).
    fn up(self) -> Vec3 {
        match self {
            Self::Up => Vec3::Z,
            Self::Down => Vec3::NEG_Z,
            Self::North | Self::South | Self::East | Self::West => Vec3::Y,
        }
    }

    /// 3x4 letter drawn in the bottom-left corner of the face, rows from top to bottom.
    const fn glyph(self) -> [[u8; GLYPH_WIDTH]; GLYPH_HEIGHT] {
        match self {
            // Front
            Self::North => [[0, 1, 1], [1, 0, 0], [1, 1, 0], [1, 0, 0]],
            // Back
            Self::South => [[0, 0, 0], [1, 0, 0], [1, 1, 1], [1, 1, 1]],
            Self::Up => [[0, 0, 0], [1, 0, 1], [1, 0, 1], [1, 1, 1]],
            Self::Down => [[0, 0, 0], [1, 1, 0], [1, 0, 1], [1, 1, 0]],
            // Left
            Self::West => [[0, 0, 0], [1, 0, 0], [1, 0, 0], [1, 1, 1]],
            // Right
            Self::East => [[0, 0, 0], [0, 1, 1], [1, 0, 0], [1, 0, 0]],
        }
    }
}

const GLYPH_WIDTH: usize = 3;
const GLYPH_HEIGHT: usize = 4;

struct PartPalette {
    border: Rgba<u8>,
    center: Rgba<u8>,
    glyph: Rgba<u8>,
}

impl PartPalette {
    fn new(part: PlayerBodyPartType, texture: PlayerPartTextureType) -> Self {
        use PlayerBodyPartType::*;

        // (hue, saturation multiplier); layers share the hue of their base part.
        // Wings and cape are provided for the body, but get their own colours.
        let (h, saturation) = if texture == PlayerPartEarsTextureType::Wings.into() {
            (320.0, 1.0)
        } else if texture == PlayerPartTextureType::Cape {
            (30.0, 1.0)
        } else {
            match part {
                Head | HeadLayer => (0.0, 0.0),
                Body | BodyLayer => (220.0, 1.0),
                RightArm | RightArmLayer => (0.0, 1.0),
                LeftArm | LeftArmLayer => (55.0, 1.0),
                RightLeg | RightLegLayer => (270.0, 1.0),
                LeftLeg | LeftLegLayer => (150.0, 1.0),
            }
        };

        Self::from_hue(h, saturation, part)
    }

    /// Ears quads drawn on the skin each get their own hue, picked from their texture position:
    /// stepping by the golden ratio keeps nearby positions far apart on the colour wheel.
    fn for_ears_quad(part: PlayerBodyPartType, uv: FaceUv) -> Self {
        const GOLDEN_RATIO_CONJUGATE: f64 = 0.618_033_988_749_895;

        let x = uv.top_left.x.min(uv.bottom_right.x);
        let y = uv.top_left.y.min(uv.bottom_right.y);
        let key = f64::from(x) + f64::from(y) * 64.0;

        Self::from_hue((key * GOLDEN_RATIO_CONJUGATE).fract() * 360.0, 1.0, part)
    }

    fn from_hue(h: f64, saturation: f64, part: PlayerBodyPartType) -> Self {
        let color = |s: f64, l: f64, a: u8| {
            let (r, g, b) = HSL {
                h,
                s: s * saturation,
                l,
            }
            .to_rgb();
            Rgba([r, g, b, a])
        };

        if part.is_layer() || part.is_hat_layer() {
            Self {
                border: color(0.65, 0.35, 255),
                center: color(0.55, 0.45, 128),
                glyph: Rgba([255, 255, 255, 255]),
            }
        } else {
            Self {
                border: color(0.55, 0.45, 255),
                center: color(0.40, 0.68, 255),
                glyph: Rgba([0, 0, 0, 255]),
            }
        }
    }
}

/// Walks the texels of a face in the orientation given by its UV corners,
/// so rotated and flipped UVs are handled.
struct FaceTexels {
    origin: IVec2,
    right: IVec2,
    down: IVec2,
    width: i32,
    height: i32,
}

impl FaceTexels {
    fn new(uv: FaceUv) -> Option<Self> {
        let point = |p: FaceUvPoint| IVec2::new(p.x as i32, p.y as i32);

        let top_left = point(uv.top_left);
        let across = point(uv.top_right) - top_left;
        let along = point(uv.bottom_left) - top_left;

        // Texture regions are axis-aligned, so each edge moves along exactly one axis.
        let is_axis_aligned = |v: IVec2| (v.x == 0) != (v.y == 0);
        if !is_axis_aligned(across) || !is_axis_aligned(along) {
            return None;
        }

        let right = across.signum();
        let down = along.signum();

        Some(Self {
            // UV corners sit on texel edges; step into the texel when walking towards negative.
            origin: top_left + right.min(IVec2::ZERO) + down.min(IVec2::ZERO),
            right,
            down,
            width: across.abs().element_sum(),
            height: along.abs().element_sum(),
        })
    }

    fn texel(&self, x: i32, y: i32) -> IVec2 {
        self.origin + self.right * x + self.down * y
    }
}

/// Paints the template for `part` onto `image`, which holds the `texture` the part is drawn with.
/// Parts drawn with other textures are skipped.
pub(crate) fn apply_part_template(
    body_part: PlayerBodyPartType,
    part: &Part,
    texture: PlayerPartTextureType,
    image: &mut RgbaImage,
) {
    let palette = PartPalette::new(body_part, texture);

    paint_part(body_part, &palette, part, texture, Affine3A::IDENTITY, image);
}

fn paint_part(
    body_part: PlayerBodyPartType,
    palette: &PartPalette,
    part: &Part,
    texture: PlayerPartTextureType,
    parent: Affine3A,
    image: &mut RgbaImage,
) {
    if !matches!(part, Part::Group { .. }) && part.get_texture() != texture {
        return;
    }

    match part {
        Part::Cube {
            face_uvs,
            transformation,
            ..
        } => {
            let transformation = parent * *transformation;

            // UVs as seen when looking at each face, with the glyph corner at the bottom left,
            // and each face's normal before the cube is transformed.
            let faces = [
                (face_uvs.north, Vec3::NEG_Z),
                (face_uvs.south, Vec3::Z),
                (face_uvs.east, Vec3::X),
                (face_uvs.west, Vec3::NEG_X),
                (face_uvs.up, Vec3::Y),
                // Seen from below, the bottom face's texture is upside down (front edge on top).
                (face_uvs.down.flip_vertically(), Vec3::NEG_Y),
            ];

            for (uv, normal) in faces {
                // Rotated cubes (like the cape) are labelled by where their faces point.
                let orientation =
                    FaceOrientation::from_normal(transformation.transform_vector3(normal));

                paint_face(palette, uv, orientation, image);
            }
        }
        Part::Quad {
            face_uv,
            normal,
            transformation,
            size,
            ..
        } => {
            // The normal already has the part's own transformation applied.
            let normal = parent.transform_vector3(*normal);
            let transformation = parent * *transformation;

            // Edges from the top-left vertex to the top-right and bottom-left vertices.
            let mut across = transformation.transform_vector3(Vec3::new(size.x, 0.0, 0.0));
            let down = transformation.transform_vector3(Vec3::new(0.0, -size.y, size.z));

            let mut uv = *face_uv;

            // Seen from the normal's side, the vertex order may be mirrored; flip to match what's visible.
            if normal.dot(down.cross(across)) < 0.0 {
                uv = uv.flip_horizontally();
                across = -across;
            }

            let orientation = FaceOrientation::from_normal(normal);

            // Turn the face so its top points the same way as a cube face with this orientation.
            // After `n` clockwise turns, the face's up direction is `ups[n]`.
            let ups = [-down, across, down, -across];
            let target_up = orientation.up();
            let turns = (0..ups.len())
                .max_by(|&a, &b| ups[a].dot(target_up).total_cmp(&ups[b].dot(target_up)))
                .unwrap_or(0);

            for _ in 0..turns {
                uv = uv.rotate_cw();
            }

            // Wings keep their texture's colour; Ears parts on the skin are told apart per quad.
            if texture == PlayerPartTextureType::Skin {
                paint_face(&PartPalette::for_ears_quad(body_part, *face_uv), uv, orientation, image);
            } else {
                paint_face(palette, uv, orientation, image);
            }
        }
        Part::Group {
            parts,
            transformation,
            ..
        } => {
            let transformation = parent * *transformation;

            for part in parts {
                paint_part(body_part, palette, part, texture, transformation, image);
            }
        }
    }
}

fn paint_face(
    palette: &PartPalette,
    uv: FaceUv,
    orientation: FaceOrientation,
    texture: &mut RgbaImage,
) {
    let Some(face) = FaceTexels::new(uv) else {
        return;
    };

    let glyph = orientation.glyph();
    let glyph_fits = face.width >= GLYPH_WIDTH as i32 && face.height >= GLYPH_HEIGHT as i32;
    let glyph_top = face.height - GLYPH_HEIGHT as i32;

    for y in 0..face.height {
        for x in 0..face.width {
            let is_glyph = glyph_fits
                && x < GLYPH_WIDTH as i32
                && (glyph_top..glyph_top + GLYPH_HEIGHT as i32).contains(&y)
                && glyph[(y - glyph_top) as usize][x as usize] == 1;
            let is_border = x == 0 || y == 0 || x == face.width - 1 || y == face.height - 1;

            let color = if is_glyph {
                palette.glyph
            } else if is_border {
                palette.border
            } else {
                palette.center
            };

            let texel = face.texel(x, y);
            let (Ok(tx), Ok(ty)) = (u32::try_from(texel.x), u32::try_from(texel.y)) else {
                continue;
            };

            if let Some(pixel) = texture.get_pixel_mut_checked(tx, ty) {
                if pixel.0[3] == 0 {
                    *pixel = color;
                }
            }
        }
    }
}

fn main() -> JsResult<()> {
    let ears_features = EarsFeatures {
        ear_mode: EarMode::Around,
        ear_anchor: EarAnchor::Center,
        tail: Some(TailData {
            mode: TailMode::Down,
            segments: 1,
            ..Default::default()
        }),
        snout: Some(SnoutData {
            offset: 2,
            width: 4,
            height: 2,
            depth: 2,
        }),
        wing: None,
        protrusions: ears_rs::features::data::protrusions::Protrusions::Claws,
        chest_size: 0f32,
        cape_enabled: false,
        emissive: false,
        ..Default::default()
    };

    let context: PlayerPartProviderContext<()> = PlayerPartProviderContext {
        model: nmsr_player_parts::model::PlayerModel::Alex,
        has_hat_layer: true,
        has_layers: true,
        has_deadmau5_ears: false,
        is_flipped_upside_down: false,
        has_cape: false,
        custom_arm_rotation_z: Some(0f32),
        shadow_y_pos: None,
        shadow_is_square: false,
        armor_slots: None,
        ears_features: Some(ears_features),
        movement: Default::default(),
    };

    let parts = [PlayerPartsProvider::Ears]
        .iter()
        .flat_map(|provider| {
            PlayerBodyPartType::iter()
                .flat_map(|p| provider.get_parts(&context, p).into_iter().zip(repeat(p)))
        })
        .collect::<Vec<_>>();

    let mut out_textures: HashMap<PlayerPartTextureType, RgbaImage> = HashMap::new();

    let grouped_parts = parts
        .into_iter()
        .sorted_by_key(|(p, _)| p.get_texture())
        .group_by(|(p, _)| p.get_texture());

    for (texture, parts) in grouped_parts.into_iter() {
        let part_texture = out_textures.entry(texture).or_insert_with(|| {
            let (width, height) = texture.get_texture_size();
            RgbaImage::new(width, height)
        });

        for (part, body_part) in parts {
            apply_part_template(body_part, &part, texture, part_texture);
        }

        if texture == PlayerPartTextureType::Skin {
            ears_rs::utils::strip_alpha(part_texture);

            ears_rs::parser::v1::writer::EarsWriterV1::write(part_texture, &ears_features)?;
        }
    }

    out_textures.into_iter().for_each(|(texture, img)| {
        img.save(format!("template-{}.png", texture.to_string()))
            .unwrap();
    });

    Ok(())
}
