use std::collections::HashMap;

use crate::output::KittyImageData;

fn kitty_image_dimensions(image_data: &KittyImageData) -> (u32, u32) {
    match image_data {
        KittyImageData::Png { width, height, .. }
        | KittyImageData::Rgb { width, height, .. }
        | KittyImageData::Rgba { width, height, .. } => (*width, *height),
    }
}

#[derive(Clone, Debug)]
pub struct KittyAsset {
    pub generation: u64,
    pub image_data: KittyImageData,
}

#[derive(Clone, Debug)]
pub struct KittyAssetStore {
    next_asset_id: u32,
    assets: HashMap<u32, KittyAsset>,
}

impl Default for KittyAssetStore {
    fn default() -> Self {
        Self {
            next_asset_id: 1,
            assets: HashMap::new(),
        }
    }
}

impl KittyAssetStore {
    pub fn next_asset_id(&mut self) -> u32 {
        let next_asset_id = self.next_asset_id;
        self.next_asset_id = self.next_asset_id.saturating_add(1);
        next_asset_id
    }

    pub fn insert_asset(&mut self, image_id: u32, image_data: KittyImageData) {
        let next_generation = self
            .assets
            .get(&image_id)
            .map(|asset| asset.generation.saturating_add(1))
            .unwrap_or(1);
        self.assets.insert(
            image_id,
            KittyAsset {
                generation: next_generation,
                image_data,
            },
        );
    }

    pub fn generation(&self, image_id: u32) -> Option<u64> {
        self.assets.get(&image_id).map(|asset| asset.generation)
    }

    pub fn asset(&self, image_id: u32) -> Option<&KittyAsset> {
        self.assets.get(&image_id)
    }

    pub fn remove_asset(&mut self, image_id: u32) -> Option<KittyAsset> {
        self.assets.remove(&image_id)
    }

    pub fn image_data(&self, image_id: u32) -> Option<KittyImageData> {
        self.assets
            .get(&image_id)
            .map(|asset| asset.image_data.clone())
    }

    pub fn image_dimensions(&self, image_id: u32) -> Option<(u32, u32)> {
        self.assets
            .get(&image_id)
            .map(|asset| kitty_image_dimensions(&asset.image_data))
    }
}
