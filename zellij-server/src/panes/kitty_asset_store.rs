use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use crate::output::KittyImageData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyAssetFormat {
    Png,
    Rgb,
    Rgba,
}

impl KittyAssetFormat {
    pub fn bytes_per_pixel(self) -> Option<usize> {
        match self {
            KittyAssetFormat::Png => None,
            KittyAssetFormat::Rgb => Some(3),
            KittyAssetFormat::Rgba => Some(4),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyRegularFileSource {
    pub path: PathBuf,
    pub offset: u64,
    pub size: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KittyAssetData {
    Image(KittyImageData),
    RegularFile {
        source: KittyRegularFileSource,
        format: KittyAssetFormat,
        width: u32,
        height: u32,
    },
}

impl KittyAssetData {
    pub fn image_data(&self) -> Option<KittyImageData> {
        match self {
            KittyAssetData::Image(image_data) => Some(image_data.clone()),
            KittyAssetData::RegularFile {
                source,
                format,
                width,
                height,
            } => {
                let payload = read_regular_file_source(source)?;
                if let Some(bytes_per_pixel) = format.bytes_per_pixel() {
                    let expected_len = expected_raw_payload_size(*width, *height, bytes_per_pixel)?;
                    if payload.len() != expected_len {
                        return None;
                    }
                }
                match format {
                    KittyAssetFormat::Png => Some(KittyImageData::Png {
                        data: payload,
                        width: *width,
                        height: *height,
                    }),
                    KittyAssetFormat::Rgb => Some(KittyImageData::Rgb {
                        data: payload,
                        width: *width,
                        height: *height,
                    }),
                    KittyAssetFormat::Rgba => Some(KittyImageData::Rgba {
                        data: payload,
                        width: *width,
                        height: *height,
                    }),
                }
            },
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            KittyAssetData::Image(image_data) => kitty_image_dimensions(image_data),
            KittyAssetData::RegularFile { width, height, .. } => (*width, *height),
        }
    }

    pub fn decoded_byte_size(&self) -> usize {
        let (width, height) = self.dimensions();
        (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4)
    }

    pub fn format(&self) -> KittyAssetFormat {
        match self {
            KittyAssetData::Image(image_data) => KittyAssetFormat::from(image_data),
            KittyAssetData::RegularFile { format, .. } => *format,
        }
    }
}

impl From<&KittyImageData> for KittyAssetFormat {
    fn from(image_data: &KittyImageData) -> Self {
        match image_data {
            KittyImageData::Png { .. } => KittyAssetFormat::Png,
            KittyImageData::Rgb { .. } => KittyAssetFormat::Rgb,
            KittyImageData::Rgba { .. } => KittyAssetFormat::Rgba,
        }
    }
}

pub fn regular_file_source_len(source: &KittyRegularFileSource) -> Option<usize> {
    let metadata = fs::metadata(&source.path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let total_len = metadata.len();
    let available_len = total_len.saturating_sub(source.offset);
    let payload_len = source.size.map(|size| size as u64).unwrap_or(available_len);
    usize::try_from(payload_len.min(available_len)).ok()
}

pub fn read_regular_file_source(source: &KittyRegularFileSource) -> Option<Vec<u8>> {
    let metadata = fs::metadata(&source.path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let mut file = fs::File::open(&source.path).ok()?;
    if source.offset > 0 {
        file.seek(SeekFrom::Start(source.offset)).ok()?;
    }
    let mut payload = Vec::new();
    match source.size {
        Some(size) => {
            let mut reader = file.take(size as u64);
            reader.read_to_end(&mut payload).ok()?;
        },
        None => {
            file.read_to_end(&mut payload).ok()?;
        },
    }
    Some(payload)
}

fn expected_raw_payload_size(width: u32, height: u32, bytes_per_pixel: usize) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(bytes_per_pixel)
}

fn kitty_image_dimensions(image_data: &KittyImageData) -> (u32, u32) {
    match image_data {
        KittyImageData::Png { width, height, .. }
        | KittyImageData::Rgb { width, height, .. }
        | KittyImageData::Rgba { width, height, .. } => (*width, *height),
    }
}

const KITTY_DEFAULT_DECODED_BYTE_QUOTA: usize = 320 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct KittyAsset {
    pub generation: u64,
    pub data: KittyAssetData,
}

#[derive(Clone, Debug)]
pub struct KittyAssetStore {
    next_asset_id: u32,
    assets: HashMap<u32, KittyAsset>,
    asset_order: Vec<u32>,
    placement_ref_counts: HashMap<u32, usize>,
    decoded_bytes_total: usize,
    decoded_byte_quota: usize,
}

impl Default for KittyAssetStore {
    fn default() -> Self {
        Self {
            next_asset_id: 1,
            assets: HashMap::new(),
            asset_order: Vec::new(),
            placement_ref_counts: HashMap::new(),
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
        self.insert_asset_data_protecting(
            image_id,
            KittyAssetData::Image(image_data),
            &HashSet::new(),
        )
    }

    pub fn insert_asset_protecting(
        &mut self,
        image_id: u32,
        image_data: KittyImageData,
        protected_image_ids: &HashSet<u32>,
    ) -> Vec<u32> {
        self.insert_asset_data_protecting(
            image_id,
            KittyAssetData::Image(image_data),
            protected_image_ids,
        )
    }

    pub fn insert_asset_data_protecting(
        &mut self,
        image_id: u32,
        data: KittyAssetData,
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
                .saturating_sub(existing_asset.data.decoded_byte_size());
        }
        self.decoded_bytes_total = self
            .decoded_bytes_total
            .saturating_add(data.decoded_byte_size());
        self.assets.insert(
            image_id,
            KittyAsset {
                generation: next_generation,
                data,
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
        self.placement_ref_counts.remove(&image_id);
        let removed_asset = self.assets.remove(&image_id);
        if let Some(asset) = removed_asset.as_ref() {
            self.decoded_bytes_total = self
                .decoded_bytes_total
                .saturating_sub(asset.data.decoded_byte_size());
        }
        removed_asset
    }

    pub fn image_data(&self, image_id: u32) -> Option<KittyImageData> {
        self.assets.get(&image_id)?.data.image_data()
    }

    pub fn image_dimensions(&self, image_id: u32) -> Option<(u32, u32)> {
        self.assets
            .get(&image_id)
            .map(|asset| asset.data.dimensions())
    }

    pub fn add_placement_reference(&mut self, image_id: u32) {
        *self.placement_ref_counts.entry(image_id).or_default() += 1;
    }

    pub fn remove_placement_reference(&mut self, image_id: u32) {
        let Some(ref_count) = self.placement_ref_counts.get_mut(&image_id) else {
            return;
        };
        *ref_count = ref_count.saturating_sub(1);
        if *ref_count == 0 {
            self.placement_ref_counts.remove(&image_id);
        }
    }

    pub fn has_placement_references(&self, image_id: u32) -> bool {
        self.placement_ref_counts
            .get(&image_id)
            .copied()
            .unwrap_or_default()
            > 0
    }

    fn evict_to_decoded_byte_quota(
        &mut self,
        newest_image_id: u32,
        protected_image_ids: &HashSet<u32>,
    ) -> Vec<u32> {
        let mut evicted_image_ids = Vec::new();
        while self.decoded_bytes_total > self.decoded_byte_quota {
            let Some(candidate_image_id) = self.asset_order.iter().copied().find(|image_id| {
                *image_id != newest_image_id
                    && !protected_image_ids.contains(image_id)
                    && !self.has_placement_references(*image_id)
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
