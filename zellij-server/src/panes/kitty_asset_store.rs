use std::collections::{HashMap, HashSet};

use crate::output::KittyImageData;

fn kitty_image_dimensions(image_data: &KittyImageData) -> (u32, u32) {
    match image_data {
        KittyImageData::Png { width, height, .. }
        | KittyImageData::Rgb { width, height, .. }
        | KittyImageData::Rgba { width, height, .. } => (*width, *height),
    }
}

fn decoded_byte_size(image_data: &KittyImageData) -> usize {
    let (width, height) = kitty_image_dimensions(image_data);
    (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4)
}

const KITTY_DEFAULT_DECODED_BYTE_QUOTA: usize = 320 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct KittyAsset {
    pub generation: u64,
    pub image_data: KittyImageData,
}

#[derive(Clone, Debug)]
pub struct KittyAssetStore {
    next_asset_id: u32,
    assets: HashMap<u32, KittyAsset>,
    asset_order: Vec<u32>,
    decoded_byte_quota: usize,
}

impl Default for KittyAssetStore {
    fn default() -> Self {
        Self {
            next_asset_id: 1,
            assets: HashMap::new(),
            asset_order: Vec::new(),
            decoded_byte_quota: KITTY_DEFAULT_DECODED_BYTE_QUOTA,
        }
    }
}

impl KittyAssetStore {
    #[cfg(test)]
    pub fn with_decoded_byte_quota(decoded_byte_quota: usize) -> Self {
        Self {
            decoded_byte_quota,
            ..Default::default()
        }
    }

    pub fn next_asset_id(&mut self) -> u32 {
        let next_asset_id = self.next_asset_id;
        self.next_asset_id = self.next_asset_id.saturating_add(1);
        next_asset_id
    }

    pub fn insert_asset(&mut self, image_id: u32, image_data: KittyImageData) -> Vec<u32> {
        self.insert_asset_protecting(image_id, image_data, &HashSet::new())
    }

    pub fn insert_asset_protecting(
        &mut self,
        image_id: u32,
        image_data: KittyImageData,
        protected_image_ids: &HashSet<u32>,
    ) -> Vec<u32> {
        let next_generation = self
            .assets
            .get(&image_id)
            .map(|asset| asset.generation.saturating_add(1))
            .unwrap_or(1);
        self.asset_order
            .retain(|existing_id| *existing_id != image_id);
        self.assets.insert(
            image_id,
            KittyAsset {
                generation: next_generation,
                image_data,
            },
        );
        self.asset_order.push(image_id);
        self.evict_to_decoded_byte_quota(image_id, protected_image_ids)
    }

    pub fn generation(&self, image_id: u32) -> Option<u64> {
        self.assets.get(&image_id).map(|asset| asset.generation)
    }

    pub fn asset(&self, image_id: u32) -> Option<&KittyAsset> {
        self.assets.get(&image_id)
    }

    pub fn remove_asset(&mut self, image_id: u32) -> Option<KittyAsset> {
        self.asset_order
            .retain(|existing_id| *existing_id != image_id);
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

    fn total_decoded_bytes(&self) -> usize {
        self.assets
            .values()
            .map(|asset| decoded_byte_size(&asset.image_data))
            .sum()
    }

    fn evict_to_decoded_byte_quota(
        &mut self,
        newest_image_id: u32,
        protected_image_ids: &HashSet<u32>,
    ) -> Vec<u32> {
        let mut evicted_image_ids = Vec::new();
        while self.total_decoded_bytes() > self.decoded_byte_quota {
            let Some(candidate_image_id) = self.asset_order.iter().copied().find(|image_id| {
                *image_id != newest_image_id && !protected_image_ids.contains(image_id)
            }) else {
                break;
            };
            self.remove_asset(candidate_image_id);
            evicted_image_ids.push(candidate_image_id);
        }
        evicted_image_ids
    }
}
