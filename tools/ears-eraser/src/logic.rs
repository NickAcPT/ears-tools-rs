use std::io::Cursor;

use ears_rs::alfalfa::{
    utils::EraseRegionsProvider,
    AlfalfaData,
};

use image::ImageFormat;
use crate::{errors::*, models::EarsImageWorkspace};
pub use ears_rs;

#[inline(never)]
pub fn decode_ears_image(skin_bytes: &[u8]) -> Result<EarsImageWorkspace> {
    let image = image::load_from_memory_with_format(skin_bytes, ImageFormat::Png)?.into_rgba8();

    let data = ears_rs::alfalfa::read_alfalfa(&image)?;
    let alfalfa = data.unwrap_or_else(|| AlfalfaData::new());

    let regions = alfalfa
        .get_erase_regions()?
        .unwrap_or_else(|| vec![])
        .into_iter()
        .map(|r| r.into())
        .collect();

    Ok(EarsImageWorkspace {
        alfalfa,
        regions,
    })
}

#[inline(never)]
pub fn encode_ears_image(skin_bytes: &[u8], workspace: &mut EarsImageWorkspace) -> Result<Vec<u8>> {
    let mut image = image::load_from_memory_with_format(skin_bytes, ImageFormat::Png)?.into_rgba8();

    let data = &mut workspace.alfalfa;
    data.set_erase_regions(&workspace.regions)?;

    ears_rs::alfalfa::write_alfalfa(&data, &mut image)?;

    if data.is_empty() {
        ears_rs::utils::strip_alpha(&mut image);
    }

    let mut bytes = Vec::new();
    {
        image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    }

    Ok(bytes)
}
