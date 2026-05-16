use std::{
    collections::{HashMap, HashSet},
    ffi::CString,
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::{output::KittyImageData, panes::kitty_image_id_allocator::KittyHostImageIdAllocator};

const KITTY_TEMP_FILE_MARKER: &str = "tty-graphics-protocol";

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
pub struct KittyByteRange {
    pub offset: u64,
    pub size: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KittyExternalMediaLocation {
    RegularFile(PathBuf),
    TemporaryFile(PathBuf),
    SharedMemory(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyExternalMedia {
    pub location: KittyExternalMediaLocation,
    pub range: KittyByteRange,
    cached_payload: Option<Box<[u8]>>,
}

impl KittyExternalMedia {
    pub fn new(location: KittyExternalMediaLocation, range: KittyByteRange) -> Self {
        Self {
            location,
            range,
            cached_payload: None,
        }
    }

    pub fn materialized_payload(&mut self) -> Option<&[u8]> {
        if self.cached_payload.is_none() {
            let payload = self.read_payload()?;
            self.cleanup_after_successful_read();
            self.cached_payload = Some(payload.into_boxed_slice());
        }
        self.cached_payload.as_deref()
    }

    pub fn into_payload(self) -> Option<Vec<u8>> {
        let payload = self.read_payload()?;
        self.cleanup_after_successful_read();
        Some(payload)
    }

    pub fn payload_len(&self) -> Option<usize> {
        match &self.location {
            KittyExternalMediaLocation::RegularFile(path)
            | KittyExternalMediaLocation::TemporaryFile(path) => {
                external_file_payload_len(path, &self.range)
            },
            KittyExternalMediaLocation::SharedMemory(name) => {
                shared_memory_payload_len(name, &self.range)
            },
        }
    }

    fn read_payload(&self) -> Option<Vec<u8>> {
        match &self.location {
            KittyExternalMediaLocation::RegularFile(path)
            | KittyExternalMediaLocation::TemporaryFile(path) => {
                read_external_file_payload(path, &self.range)
            },
            KittyExternalMediaLocation::SharedMemory(name) => {
                read_shared_memory_payload(name, &self.range)
            },
        }
    }

    fn cleanup_after_successful_read(&self) {
        match &self.location {
            KittyExternalMediaLocation::TemporaryFile(path) => {
                if is_safe_kitty_temporary_file_path(path) {
                    fs::remove_file(path).ok();
                }
            },
            KittyExternalMediaLocation::SharedMemory(name) => unlink_shared_memory(name),
            KittyExternalMediaLocation::RegularFile(_) => {},
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KittyAssetData {
    Image(KittyImageData),
    External {
        media: KittyExternalMedia,
        format: KittyAssetFormat,
        width: u32,
        height: u32,
    },
}

pub struct KittyImagePayload<'a> {
    pub format: KittyAssetFormat,
    pub width: u32,
    pub height: u32,
    pub bytes: &'a [u8],
}

impl<'a> From<&'a KittyImageData> for KittyImagePayload<'a> {
    fn from(image_data: &'a KittyImageData) -> Self {
        match image_data {
            KittyImageData::Png {
                data,
                width,
                height,
            } => Self {
                format: KittyAssetFormat::Png,
                width: *width,
                height: *height,
                bytes: data,
            },
            KittyImageData::Rgb {
                data,
                width,
                height,
            } => Self {
                format: KittyAssetFormat::Rgb,
                width: *width,
                height: *height,
                bytes: data,
            },
            KittyImageData::Rgba {
                data,
                width,
                height,
            } => Self {
                format: KittyAssetFormat::Rgba,
                width: *width,
                height: *height,
                bytes: data,
            },
        }
    }
}

impl KittyAssetData {
    pub fn materialized_image_payload(&mut self) -> Option<KittyImagePayload<'_>> {
        match self {
            KittyAssetData::Image(image_data) => Some(KittyImagePayload::from(&*image_data)),
            KittyAssetData::External {
                media,
                format,
                width,
                height,
            } => {
                let format = *format;
                let width = *width;
                let height = *height;
                let payload = media.materialized_payload()?;
                if let Some(bytes_per_pixel) = format.bytes_per_pixel() {
                    let expected_len = expected_raw_payload_size(width, height, bytes_per_pixel)?;
                    if payload.len() != expected_len {
                        return None;
                    }
                }
                Some(KittyImagePayload {
                    format,
                    width,
                    height,
                    bytes: payload,
                })
            },
        }
    }

    pub fn image_data(&mut self) -> Option<KittyImageData> {
        match self {
            KittyAssetData::Image(image_data) => Some(image_data.clone()),
            KittyAssetData::External {
                media,
                format,
                width,
                height,
            } => {
                let payload = media.materialized_payload()?;
                if let Some(bytes_per_pixel) = format.bytes_per_pixel() {
                    let expected_len = expected_raw_payload_size(*width, *height, bytes_per_pixel)?;
                    if payload.len() != expected_len {
                        return None;
                    }
                }
                let payload = payload.to_vec();
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
            KittyAssetData::External { width, height, .. } => (*width, *height),
        }
    }

    pub fn decoded_byte_size(&self) -> usize {
        let (width, height) = self.dimensions();
        let bytes_per_pixel = match self.format() {
            KittyAssetFormat::Rgb => 3,
            KittyAssetFormat::Png | KittyAssetFormat::Rgba => 4,
        };
        (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(bytes_per_pixel)
    }

    pub fn format(&self) -> KittyAssetFormat {
        match self {
            KittyAssetData::Image(image_data) => KittyAssetFormat::from(image_data),
            KittyAssetData::External { format, .. } => *format,
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

pub fn external_file_payload_len(path: &Path, range: &KittyByteRange) -> Option<usize> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let total_len = metadata.len();
    let available_len = total_len.saturating_sub(range.offset);
    let payload_len = range.size.map(|size| size as u64).unwrap_or(available_len);
    usize::try_from(payload_len.min(available_len)).ok()
}

pub fn read_external_file_payload(path: &Path, range: &KittyByteRange) -> Option<Vec<u8>> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let mut file = fs::File::open(path).ok()?;
    if range.offset > 0 {
        file.seek(SeekFrom::Start(range.offset)).ok()?;
    }
    let mut payload = Vec::new();
    match range.size {
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

fn known_kitty_temp_dirs() -> Vec<PathBuf> {
    let mut temp_dirs = vec![std::env::temp_dir()];
    temp_dirs.push(PathBuf::from("/tmp"));
    temp_dirs.push(PathBuf::from("/dev/shm"));
    temp_dirs
        .into_iter()
        .filter_map(|path| path.canonicalize().ok())
        .collect()
}

fn is_safe_kitty_temporary_file_path(path: &Path) -> bool {
    if !path.to_string_lossy().contains(KITTY_TEMP_FILE_MARKER) {
        return false;
    }
    let Some(parent) = path.parent().and_then(|parent| parent.canonicalize().ok()) else {
        return false;
    };
    known_kitty_temp_dirs()
        .iter()
        .any(|temp_dir| parent.starts_with(temp_dir))
}

#[cfg(unix)]
fn shared_memory_payload_len(name: &str, range: &KittyByteRange) -> Option<usize> {
    let c_name = CString::new(name).ok()?;
    unsafe {
        let fd = libc::shm_open(c_name.as_ptr(), libc::O_RDONLY, 0o600);
        if fd < 0 {
            return None;
        }
        let len = shared_memory_payload_len_from_fd(fd, range);
        libc::close(fd);
        len
    }
}

#[cfg(not(unix))]
fn shared_memory_payload_len(_name: &str, _range: &KittyByteRange) -> Option<usize> {
    None
}

#[cfg(unix)]
fn shared_memory_payload_len_from_fd(fd: libc::c_int, range: &KittyByteRange) -> Option<usize> {
    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::fstat(fd, &mut stat) != 0 || stat.st_size < 0 {
            return None;
        }
        let total_len = usize::try_from(stat.st_size).ok()?;
        let offset = usize::try_from(range.offset).ok()?;
        let available_len = total_len.saturating_sub(offset);
        Some(range.size.unwrap_or(available_len).min(available_len))
    }
}

#[cfg(unix)]
fn read_shared_memory_payload(name: &str, range: &KittyByteRange) -> Option<Vec<u8>> {
    let c_name = CString::new(name).ok()?;
    unsafe {
        let fd = libc::shm_open(c_name.as_ptr(), libc::O_RDONLY, 0o600);
        if fd < 0 {
            return None;
        }
        let payload = read_shared_memory_payload_from_fd(fd, range);
        libc::close(fd);
        payload
    }
}

#[cfg(not(unix))]
fn read_shared_memory_payload(_name: &str, _range: &KittyByteRange) -> Option<Vec<u8>> {
    None
}

#[cfg(unix)]
fn read_shared_memory_payload_from_fd(fd: libc::c_int, range: &KittyByteRange) -> Option<Vec<u8>> {
    unsafe {
        let mut stat: libc::stat = std::mem::zeroed();
        if libc::fstat(fd, &mut stat) != 0 || stat.st_size < 0 {
            return None;
        }
        let total_len = usize::try_from(stat.st_size).ok()?;
        let offset = usize::try_from(range.offset).ok()?;
        let copy_len = shared_memory_payload_len_from_fd(fd, range)?;

        if copy_len == 0 || total_len == 0 {
            return Some(Vec::new());
        }
        let mapping = libc::mmap(
            std::ptr::null_mut(),
            total_len,
            libc::PROT_READ,
            libc::MAP_SHARED,
            fd,
            0,
        );
        if mapping == libc::MAP_FAILED {
            return None;
        }
        let bytes =
            std::slice::from_raw_parts((mapping as *const u8).add(offset), copy_len).to_vec();
        libc::munmap(mapping, total_len);
        Some(bytes)
    }
}

#[cfg(unix)]
fn unlink_shared_memory(name: &str) {
    let Some(c_name) = CString::new(name).ok() else {
        return;
    };
    unsafe {
        libc::shm_unlink(c_name.as_ptr());
    }
}

#[cfg(not(unix))]
fn unlink_shared_memory(_name: &str) {}

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
    image_id_allocator: KittyHostImageIdAllocator,
    assets: HashMap<u32, KittyAsset>,
    asset_order: Vec<u32>,
    placement_ref_counts: HashMap<u32, usize>,
    decoded_bytes_total: usize,
    decoded_byte_quota: usize,
}

impl Default for KittyAssetStore {
    fn default() -> Self {
        Self {
            image_id_allocator: KittyHostImageIdAllocator::default(),
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

    pub fn next_asset_id(&mut self) -> Option<u32> {
        Some(self.next_host_image_id())
    }

    pub fn next_host_image_id(&mut self) -> u32 {
        self.image_id_allocator.next()
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
            .map(|asset| asset.generation.wrapping_add(1))
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

    pub fn asset_mut(&mut self, image_id: u32) -> Option<&mut KittyAsset> {
        self.assets.get_mut(&image_id)
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

    pub fn image_data(&mut self, image_id: u32) -> Option<KittyImageData> {
        self.assets.get_mut(&image_id)?.data.image_data()
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

    fn rgb(width: u32, height: u32) -> KittyImageData {
        KittyImageData::Rgb {
            data: vec![0; (width * height * 3) as usize],
            width,
            height,
        }
    }

    fn png(width: u32, height: u32, payload_len: usize) -> KittyImageData {
        KittyImageData::Png {
            data: vec![0; payload_len],
            width,
            height,
        }
    }

    #[test]
    fn materialized_image_payload_borrows_resident_image_bytes() {
        let data = vec![1, 2, 3, 4];
        let expected_ptr = data.as_ptr();
        let mut asset_data = KittyAssetData::Image(KittyImageData::Rgba {
            data,
            width: 1,
            height: 1,
        });

        let payload = asset_data
            .materialized_image_payload()
            .expect("resident image payload should be available");

        assert_eq!(payload.format, KittyAssetFormat::Rgba);
        assert_eq!(payload.width, 1);
        assert_eq!(payload.height, 1);
        assert_eq!(payload.bytes, &[1, 2, 3, 4]);
        assert_eq!(
            payload.bytes.as_ptr(),
            expected_ptr,
            "resident payload should borrow the stored bytes instead of cloning them"
        );
    }

    #[test]
    fn decoded_byte_size_tracks_decoded_pixel_size_by_format() {
        assert_eq!(KittyAssetData::Image(rgba(2, 2)).decoded_byte_size(), 16);
        assert_eq!(KittyAssetData::Image(rgb(2, 2)).decoded_byte_size(), 12);
        assert_eq!(KittyAssetData::Image(png(2, 2, 1)).decoded_byte_size(), 16);
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

    #[test]
    fn next_asset_id_wraps_without_returning_zero() {
        let mut store = KittyAssetStore::default();
        store.image_id_allocator = KittyHostImageIdAllocator::new(u32::MAX);

        assert_eq!(store.next_asset_id(), Some(u32::MAX));
        assert_eq!(store.next_asset_id(), Some(1));
    }

    #[test]
    fn replacing_asset_wraps_generation_instead_of_saturating() {
        let mut store = KittyAssetStore::default();
        store.insert_asset(1, rgba(1, 1));
        store.assets.get_mut(&1).unwrap().generation = u64::MAX;

        store.insert_asset(1, rgba(2, 2));

        assert_eq!(store.generation(1), Some(0));
    }
}
