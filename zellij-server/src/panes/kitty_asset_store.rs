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
    decoded_bytes_total: usize,
    decoded_byte_quota: usize,
}

impl Default for KittyAssetStore {
    fn default() -> Self {
        Self {
            next_asset_id: 1,
            assets: HashMap::new(),
            asset_order: Vec::new(),
            decoded_bytes_total: 0,
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
        if let Some(existing_asset) = self.assets.get(&image_id) {
            self.decoded_bytes_total = self
                .decoded_bytes_total
                .saturating_sub(decoded_byte_size(&existing_asset.image_data));
        }
        self.decoded_bytes_total = self
            .decoded_bytes_total
            .saturating_add(decoded_byte_size(&image_data));
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
        let removed_asset = self.assets.remove(&image_id);
        if let Some(asset) = removed_asset.as_ref() {
            self.decoded_bytes_total = self
                .decoded_bytes_total
                .saturating_sub(decoded_byte_size(&asset.image_data));
        }
        removed_asset
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

    fn evict_to_decoded_byte_quota(
        &mut self,
        newest_image_id: u32,
        protected_image_ids: &HashSet<u32>,
    ) -> Vec<u32> {
        let mut evicted_image_ids = Vec::new();
        while self.decoded_bytes_total > self.decoded_byte_quota {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(width: u32, height: u32) -> KittyImageData {
        KittyImageData::Rgba {
            data: vec![0; (width * height * 4) as usize],
            width,
            height,
        }
    }

    #[test]
    fn decoded_byte_total_tracks_insert_replace_remove_and_evict() {
        let mut store = KittyAssetStore::with_decoded_byte_quota(20);

        store.insert_asset(1, rgba(1, 1));
        assert_eq!(store.decoded_bytes_total, 4);

        store.insert_asset(1, rgba(2, 2));
        assert_eq!(store.decoded_bytes_total, 16);

        store.insert_asset(2, rgba(1, 1));
        assert_eq!(store.decoded_bytes_total, 20);

        let evicted = store.insert_asset(3, rgba(1, 1));
        assert_eq!(evicted, vec![1]);
        assert_eq!(store.decoded_bytes_total, 8);

        store.remove_asset(2);
        assert_eq!(store.decoded_bytes_total, 4);
    }
}
