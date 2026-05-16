use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use zellij_utils::data::{
    PluginCellRect, PluginGraphicsOp, PluginGraphicsUpdate, PluginImageSource, PluginPixelRect,
};

use crate::output::{
    placement_id_allocator, KittyImageChunk, KittyImageData, PlacementId, PlacementIdAllocator,
};
use crate::panes::kitty_asset_store::KittyAssetStore;
use crate::panes::pane_image_scene::{KittyRenderBundle, LogicalPlacementId};

const PLUGIN_GRAPHICS_MAX_ENCODED_BYTES: usize = 80 * 1024 * 1024;
const PLUGIN_GRAPHICS_MAX_DECODED_BYTES: usize = 80 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PluginGraphicsError(String);

impl PluginGraphicsError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for PluginGraphicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PluginGraphicsAsset {
    host_image_id: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PluginGraphicsPlacement {
    host_placement_id: LogicalPlacementId,
    plugin_asset_id: u32,
    destination: PluginCellRect,
    source: PluginPixelRect,
    z_index: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct PluginGraphicsScene {
    kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
    assets: HashMap<u32, PluginGraphicsAsset>,
    placements: BTreeMap<u32, PluginGraphicsPlacement>,
}

impl PluginGraphicsScene {
    pub(crate) fn new(kitty_asset_store: Rc<RefCell<KittyAssetStore>>) -> Self {
        Self {
            kitty_asset_store,
            assets: HashMap::new(),
            placements: BTreeMap::new(),
        }
    }

    pub(crate) fn apply_update(
        &mut self,
        update: PluginGraphicsUpdate,
        plugin_cwd: &Path,
    ) -> Result<(), PluginGraphicsError> {
        let mut scratch_assets = self.assets.clone();
        let mut scratch_placements = self.placements.clone();
        let mut staged_image_data = HashMap::new();

        for op in update.ops {
            match op {
                PluginGraphicsOp::SetAsset { asset_id, source } => {
                    let image_data = image_data_from_source(source, plugin_cwd)?;
                    let (width, height) = image_dimensions(&image_data);
                    let host_image_id = scratch_assets
                        .get(&asset_id)
                        .map(|asset| asset.host_image_id)
                        .unwrap_or_else(|| {
                            self.kitty_asset_store.borrow_mut().next_host_image_id()
                        });
                    scratch_assets.insert(
                        asset_id,
                        PluginGraphicsAsset {
                            host_image_id,
                            width,
                            height,
                        },
                    );
                    staged_image_data.insert(asset_id, image_data);
                },
                PluginGraphicsOp::DeleteAsset { asset_id } => {
                    scratch_assets.remove(&asset_id);
                    scratch_placements.retain(|_, placement| placement.plugin_asset_id != asset_id);
                    staged_image_data.remove(&asset_id);
                },
                PluginGraphicsOp::PlaceImage {
                    placement_id,
                    asset_id,
                    destination,
                    source,
                    z_index,
                } => {
                    validate_destination(destination)?;
                    let asset = scratch_assets
                        .get(&asset_id)
                        .ok_or_else(|| PluginGraphicsError::new("unknown plugin graphics asset"))?;
                    let source = source.unwrap_or(PluginPixelRect {
                        x: 0,
                        y: 0,
                        width: asset.width,
                        height: asset.height,
                    });
                    validate_source(source, asset)?;
                    let host_placement_id = scratch_placements
                        .get(&placement_id)
                        .map(|placement| placement.host_placement_id)
                        .unwrap_or_else(|| {
                            placement_id_allocator().allocate_explicit_placement_id()
                        });
                    scratch_placements.insert(
                        placement_id,
                        PluginGraphicsPlacement {
                            host_placement_id,
                            plugin_asset_id: asset_id,
                            destination,
                            source,
                            z_index,
                        },
                    );
                },
                PluginGraphicsOp::DeletePlacement { placement_id } => {
                    scratch_placements.remove(&placement_id);
                },
                PluginGraphicsOp::ClearPlacements => {
                    scratch_placements.clear();
                },
                PluginGraphicsOp::ClearAssets => {
                    scratch_assets.clear();
                    scratch_placements.clear();
                    staged_image_data.clear();
                },
            }
        }

        let protected_image_ids = scratch_placements
            .values()
            .filter_map(|placement| scratch_assets.get(&placement.plugin_asset_id))
            .map(|asset| asset.host_image_id)
            .collect::<HashSet<_>>();

        for (asset_id, image_data) in staged_image_data {
            if let Some(asset) = scratch_assets.get(&asset_id) {
                self.kitty_asset_store.borrow_mut().insert_asset_protecting(
                    asset.host_image_id,
                    image_data,
                    &protected_image_ids,
                );
            }
        }

        for old_asset in self.assets.values() {
            if !scratch_assets
                .values()
                .any(|asset| asset.host_image_id == old_asset.host_image_id)
            {
                self.kitty_asset_store
                    .borrow_mut()
                    .remove_asset(old_asset.host_image_id);
            }
        }

        for old_placement in self.placements.values() {
            if let Some(asset) = self.assets.get(&old_placement.plugin_asset_id) {
                self.kitty_asset_store
                    .borrow_mut()
                    .remove_placement_reference(asset.host_image_id);
            }
        }
        for new_placement in scratch_placements.values() {
            if let Some(asset) = scratch_assets.get(&new_placement.plugin_asset_id) {
                self.kitty_asset_store
                    .borrow_mut()
                    .add_placement_reference(asset.host_image_id);
            }
        }

        self.assets = scratch_assets;
        self.placements = scratch_placements;
        Ok(())
    }

    pub(crate) fn visible_kitty_render_bundle(
        &self,
        content_x: usize,
        content_y: usize,
        viewport_width: usize,
        viewport_height: usize,
    ) -> KittyRenderBundle {
        let mut explicit_chunks = Vec::new();
        for placement in self.placements.values() {
            let Some(asset) = self.assets.get(&placement.plugin_asset_id) else {
                continue;
            };
            let Some(projected) = project_placement(
                placement,
                content_x,
                content_y,
                viewport_width,
                viewport_height,
            ) else {
                continue;
            };
            explicit_chunks.push(KittyImageChunk {
                stable_render_id: placement.host_placement_id.0,
                image_id: asset.host_image_id,
                placement_id: Some(PlacementId::Protocol(placement.host_placement_id.0 as u32)),
                cell_x: projected.cell_x,
                cell_y: projected.cell_y,
                columns: projected.columns,
                rows: projected.rows,
                columns_specified: true,
                rows_specified: true,
                source_x: projected.source.x,
                source_y: projected.source.y,
                source_width: projected.source.width,
                source_height: projected.source.height,
                z_index: placement.z_index,
                x_offset: 0,
                y_offset: 0,
            });
        }
        KittyRenderBundle {
            explicit_chunks,
            placeholder_renders: Vec::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.assets.is_empty() && self.placements.is_empty()
    }
}

impl Drop for PluginGraphicsScene {
    fn drop(&mut self) {
        for placement in self.placements.values() {
            if let Some(asset) = self.assets.get(&placement.plugin_asset_id) {
                self.kitty_asset_store
                    .borrow_mut()
                    .remove_placement_reference(asset.host_image_id);
            }
        }
        for asset in self.assets.values() {
            self.kitty_asset_store
                .borrow_mut()
                .remove_asset(asset.host_image_id);
        }
    }
}

#[derive(Debug)]
struct ProjectedPlacement {
    cell_x: usize,
    cell_y: usize,
    columns: usize,
    rows: usize,
    source: PluginPixelRect,
}

fn image_data_from_source(
    source: PluginImageSource,
    plugin_cwd: &Path,
) -> Result<KittyImageData, PluginGraphicsError> {
    match source {
        PluginImageSource::PngBytes(bytes) => png_image_data(bytes),
        PluginImageSource::RgbaBytes {
            width,
            height,
            bytes,
        } => rgba_image_data(width, height, bytes),
        PluginImageSource::PngFile(path) => {
            let path = if path.is_absolute() {
                path
            } else {
                plugin_cwd.join(path)
            };
            let bytes = fs::read(path)
                .map_err(|e| PluginGraphicsError::new(format!("failed to read png file: {e}")))?;
            png_image_data(bytes)
        },
    }
}

fn png_image_data(bytes: Vec<u8>) -> Result<KittyImageData, PluginGraphicsError> {
    if bytes.len() > PLUGIN_GRAPHICS_MAX_ENCODED_BYTES {
        return Err(PluginGraphicsError::new(
            "encoded plugin image is too large",
        ));
    }
    let (width, height) = parse_png_dimensions(&bytes)
        .ok_or_else(|| PluginGraphicsError::new("invalid png image"))?;
    if width == 0 || height == 0 {
        return Err(PluginGraphicsError::new(
            "png image dimensions must be non-zero",
        ));
    }
    validate_decoded_size(width, height)?;
    Ok(KittyImageData::Png {
        data: bytes,
        width,
        height,
    })
}

fn rgba_image_data(
    width: u32,
    height: u32,
    bytes: Vec<u8>,
) -> Result<KittyImageData, PluginGraphicsError> {
    if width == 0 || height == 0 {
        return Err(PluginGraphicsError::new(
            "rgba image dimensions must be non-zero",
        ));
    }
    let expected_len = decoded_size(width, height)?;
    if bytes.len() != expected_len {
        return Err(PluginGraphicsError::new("rgba image byte length mismatch"));
    }
    Ok(KittyImageData::Rgba {
        data: bytes,
        width,
        height,
    })
}

fn validate_decoded_size(width: u32, height: u32) -> Result<(), PluginGraphicsError> {
    let decoded_size = decoded_size(width, height)?;
    if decoded_size > PLUGIN_GRAPHICS_MAX_DECODED_BYTES {
        return Err(PluginGraphicsError::new(
            "decoded plugin image is too large",
        ));
    }
    Ok(())
}

fn decoded_size(width: u32, height: u32) -> Result<usize, PluginGraphicsError> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| PluginGraphicsError::new("plugin image dimensions overflow"))
}

fn validate_destination(destination: PluginCellRect) -> Result<(), PluginGraphicsError> {
    if destination.columns == 0 || destination.rows == 0 {
        return Err(PluginGraphicsError::new(
            "plugin graphics destination must be non-zero",
        ));
    }
    Ok(())
}

fn validate_source(
    source: PluginPixelRect,
    asset: &PluginGraphicsAsset,
) -> Result<(), PluginGraphicsError> {
    if source.width == 0 || source.height == 0 {
        return Err(PluginGraphicsError::new(
            "plugin graphics source must be non-zero",
        ));
    }
    let source_end_x = source
        .x
        .checked_add(source.width)
        .ok_or_else(|| PluginGraphicsError::new("plugin graphics source overflows"))?;
    let source_end_y = source
        .y
        .checked_add(source.height)
        .ok_or_else(|| PluginGraphicsError::new("plugin graphics source overflows"))?;
    if source_end_x > asset.width || source_end_y > asset.height {
        return Err(PluginGraphicsError::new(
            "plugin graphics source exceeds asset bounds",
        ));
    }
    Ok(())
}

fn project_placement(
    placement: &PluginGraphicsPlacement,
    content_x: usize,
    content_y: usize,
    viewport_width: usize,
    viewport_height: usize,
) -> Option<ProjectedPlacement> {
    let dest_x = placement.destination.x as usize;
    let dest_y = placement.destination.y as usize;
    let dest_columns = placement.destination.columns as usize;
    let dest_rows = placement.destination.rows as usize;
    let visible_left = dest_x.max(0);
    let visible_top = dest_y.max(0);
    let visible_right = dest_x.saturating_add(dest_columns).min(viewport_width);
    let visible_bottom = dest_y.saturating_add(dest_rows).min(viewport_height);
    if visible_left >= visible_right || visible_top >= visible_bottom {
        return None;
    }
    let clipped_left = visible_left.saturating_sub(dest_x);
    let clipped_top = visible_top.saturating_sub(dest_y);
    let visible_columns = visible_right - visible_left;
    let visible_rows = visible_bottom - visible_top;
    let source = PluginPixelRect {
        x: placement.source.x + scale_u32(placement.source.width, clipped_left, dest_columns),
        y: placement.source.y + scale_u32(placement.source.height, clipped_top, dest_rows),
        width: scale_u32(placement.source.width, visible_columns, dest_columns),
        height: scale_u32(placement.source.height, visible_rows, dest_rows),
    };
    if source.width == 0 || source.height == 0 {
        return None;
    }
    Some(ProjectedPlacement {
        cell_x: content_x + visible_left,
        cell_y: content_y + visible_top,
        columns: visible_columns,
        rows: visible_rows,
        source,
    })
}

fn scale_u32(value: u32, numerator: usize, denominator: usize) -> u32 {
    if denominator == 0 {
        return 0;
    }
    ((value as u64 * numerator as u64) / denominator as u64) as u32
}

fn image_dimensions(image_data: &KittyImageData) -> (u32, u32) {
    match image_data {
        KittyImageData::Png { width, height, .. }
        | KittyImageData::Rgb { width, height, .. }
        | KittyImageData::Rgba { width, height, .. } => (*width, *height),
    }
}

fn parse_png_dimensions(png_data: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIG: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if png_data.len() < 24 || &png_data[0..8] != PNG_SIG {
        return None;
    }
    if &png_data[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes([png_data[16], png_data[17], png_data[18], png_data[19]]);
    let height = u32::from_be_bytes([png_data[20], png_data[21], png_data[22], png_data[23]]);
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&width.to_be_bytes());
        png.extend_from_slice(&height.to_be_bytes());
        png
    }

    fn rgba(width: u32, height: u32) -> PluginImageSource {
        PluginImageSource::RgbaBytes {
            width,
            height,
            bytes: vec![0; width as usize * height as usize * 4],
        }
    }

    fn destination(columns: u32, rows: u32) -> PluginCellRect {
        PluginCellRect {
            x: 0,
            y: 0,
            columns,
            rows,
        }
    }

    fn source(width: u32, height: u32) -> PluginPixelRect {
        PluginPixelRect {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    fn scene() -> (PluginGraphicsScene, Rc<RefCell<KittyAssetStore>>) {
        let store = Rc::new(RefCell::new(KittyAssetStore::default()));
        (PluginGraphicsScene::new(store.clone()), store)
    }

    #[test]
    fn plugin_graphics_empty_scene_renders_empty_bundle() {
        let (scene, _) = scene();

        let bundle = scene.visible_kitty_render_bundle(1, 2, 10, 10);

        assert!(scene.is_empty());
        assert!(bundle.explicit_chunks.is_empty());
        assert!(bundle.placeholder_renders.is_empty());
    }

    #[test]
    fn plugin_graphics_asset_accepts_png_bytes() {
        let (mut scene, store) = scene();

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: PluginImageSource::PngBytes(png(2, 3)),
                    }],
                },
                Path::new("."),
            )
            .unwrap();

        let host_image_id = scene.assets.get(&1).unwrap().host_image_id;
        assert_eq!(store.borrow().image_dimensions(host_image_id), Some((2, 3)));
    }

    #[test]
    fn plugin_graphics_asset_accepts_rgba_bytes() {
        let (mut scene, store) = scene();

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: rgba(2, 1),
                    }],
                },
                Path::new("."),
            )
            .unwrap();

        let host_image_id = scene.assets.get(&1).unwrap().host_image_id;
        assert_eq!(store.borrow().image_dimensions(host_image_id), Some((2, 1)));
    }

    #[test]
    fn plugin_graphics_asset_accepts_png_file_relative_to_plugin_cwd() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("icon.png"), png(4, 5)).unwrap();
        let (mut scene, store) = scene();

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: PluginImageSource::PngFile("icon.png".into()),
                    }],
                },
                dir.path(),
            )
            .unwrap();

        let host_image_id = scene.assets.get(&1).unwrap().host_image_id;
        assert_eq!(store.borrow().image_dimensions(host_image_id), Some((4, 5)));
    }

    #[test]
    fn plugin_graphics_asset_rejects_invalid_batches_without_mutating_scene() {
        let (mut scene, store) = scene();
        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: rgba(1, 1),
                    }],
                },
                Path::new("."),
            )
            .unwrap();
        let original_host_image_id = scene.assets.get(&1).unwrap().host_image_id;

        let err = scene.apply_update(
            PluginGraphicsUpdate {
                ops: vec![
                    PluginGraphicsOp::SetAsset {
                        asset_id: 2,
                        source: rgba(1, 1),
                    },
                    PluginGraphicsOp::PlaceImage {
                        placement_id: 1,
                        asset_id: 99,
                        destination: destination(1, 1),
                        source: None,
                        z_index: 0,
                    },
                ],
            },
            Path::new("."),
        );

        assert!(err.is_err());
        assert!(scene.assets.contains_key(&1));
        assert!(!scene.assets.contains_key(&2));
        assert_eq!(
            store.borrow().image_dimensions(original_host_image_id),
            Some((1, 1))
        );
    }

    #[test]
    fn plugin_graphics_asset_rejects_zero_or_mismatched_rgba() {
        let (mut scene, _) = scene();

        assert!(scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: PluginImageSource::RgbaBytes {
                            width: 0,
                            height: 1,
                            bytes: vec![],
                        },
                    }],
                },
                Path::new("."),
            )
            .is_err());
        assert!(scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: PluginImageSource::RgbaBytes {
                            width: 2,
                            height: 2,
                            bytes: vec![0; 3],
                        },
                    }],
                },
                Path::new("."),
            )
            .is_err());
    }

    #[test]
    fn plugin_graphics_asset_rejects_invalid_png_dimensions() {
        let (mut scene, _) = scene();

        assert!(scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: PluginImageSource::PngBytes(png(0, 1)),
                    }],
                },
                Path::new("."),
            )
            .is_err());
    }

    #[test]
    fn plugin_graphics_placement_renders_explicit_chunk() {
        let (mut scene, _) = scene();
        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![
                        PluginGraphicsOp::SetAsset {
                            asset_id: 1,
                            source: rgba(10, 20),
                        },
                        PluginGraphicsOp::PlaceImage {
                            placement_id: 2,
                            asset_id: 1,
                            destination: PluginCellRect {
                                x: 1,
                                y: 2,
                                columns: 3,
                                rows: 4,
                            },
                            source: Some(PluginPixelRect {
                                x: 2,
                                y: 3,
                                width: 5,
                                height: 6,
                            }),
                            z_index: -1,
                        },
                    ],
                },
                Path::new("."),
            )
            .unwrap();

        let bundle = scene.visible_kitty_render_bundle(10, 20, 80, 24);

        assert_eq!(bundle.explicit_chunks.len(), 1);
        let chunk = &bundle.explicit_chunks[0];
        assert_eq!(chunk.cell_x, 11);
        assert_eq!(chunk.cell_y, 22);
        assert_eq!(chunk.columns, 3);
        assert_eq!(chunk.rows, 4);
        assert_eq!(chunk.source_x, 2);
        assert_eq!(chunk.source_y, 3);
        assert_eq!(chunk.source_width, 5);
        assert_eq!(chunk.source_height, 6);
        assert_eq!(chunk.z_index, -1);
        assert!(matches!(chunk.placement_id, Some(PlacementId::Protocol(_))));
    }

    #[test]
    fn plugin_graphics_placement_rejects_unknown_asset_and_invalid_rects() {
        let (mut scene, _) = scene();

        assert!(scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::PlaceImage {
                        placement_id: 1,
                        asset_id: 99,
                        destination: destination(1, 1),
                        source: None,
                        z_index: 0,
                    }],
                },
                Path::new("."),
            )
            .is_err());

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::SetAsset {
                        asset_id: 1,
                        source: rgba(2, 2),
                    }],
                },
                Path::new("."),
            )
            .unwrap();

        assert!(scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::PlaceImage {
                        placement_id: 1,
                        asset_id: 1,
                        destination: destination(0, 1),
                        source: None,
                        z_index: 0,
                    }],
                },
                Path::new("."),
            )
            .is_err());
        assert!(scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::PlaceImage {
                        placement_id: 1,
                        asset_id: 1,
                        destination: destination(1, 1),
                        source: Some(source(0, 1)),
                        z_index: 0,
                    }],
                },
                Path::new("."),
            )
            .is_err());
        assert!(scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::PlaceImage {
                        placement_id: 1,
                        asset_id: 1,
                        destination: destination(1, 1),
                        source: Some(PluginPixelRect {
                            x: 1,
                            y: 0,
                            width: 2,
                            height: 1,
                        }),
                        z_index: 0,
                    }],
                },
                Path::new("."),
            )
            .is_err());
    }

    #[test]
    fn plugin_graphics_deletes_and_clears_scene_parts() {
        let (mut scene, store) = scene();
        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![
                        PluginGraphicsOp::SetAsset {
                            asset_id: 1,
                            source: rgba(1, 1),
                        },
                        PluginGraphicsOp::PlaceImage {
                            placement_id: 1,
                            asset_id: 1,
                            destination: destination(1, 1),
                            source: None,
                            z_index: 0,
                        },
                    ],
                },
                Path::new("."),
            )
            .unwrap();
        let host_image_id = scene.assets.get(&1).unwrap().host_image_id;

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::DeletePlacement { placement_id: 1 }],
                },
                Path::new("."),
            )
            .unwrap();
        assert!(scene
            .visible_kitty_render_bundle(0, 0, 10, 10)
            .explicit_chunks
            .is_empty());
        assert!(store.borrow().asset(host_image_id).is_some());

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![
                        PluginGraphicsOp::PlaceImage {
                            placement_id: 1,
                            asset_id: 1,
                            destination: destination(1, 1),
                            source: None,
                            z_index: 0,
                        },
                        PluginGraphicsOp::DeleteAsset { asset_id: 1 },
                    ],
                },
                Path::new("."),
            )
            .unwrap();
        assert!(scene.is_empty());
        assert!(store.borrow().asset(host_image_id).is_none());
    }

    #[test]
    fn plugin_graphics_clear_placements_keeps_assets_but_clear_assets_removes_all() {
        let (mut scene, store) = scene();
        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![
                        PluginGraphicsOp::SetAsset {
                            asset_id: 1,
                            source: rgba(1, 1),
                        },
                        PluginGraphicsOp::PlaceImage {
                            placement_id: 1,
                            asset_id: 1,
                            destination: destination(1, 1),
                            source: None,
                            z_index: 0,
                        },
                    ],
                },
                Path::new("."),
            )
            .unwrap();
        let host_image_id = scene.assets.get(&1).unwrap().host_image_id;

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::ClearPlacements],
                },
                Path::new("."),
            )
            .unwrap();
        assert!(scene.placements.is_empty());
        assert!(scene.assets.contains_key(&1));
        assert!(store.borrow().asset(host_image_id).is_some());

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::ClearAssets],
                },
                Path::new("."),
            )
            .unwrap();
        assert!(scene.is_empty());
        assert!(store.borrow().asset(host_image_id).is_none());
    }

    #[test]
    fn plugin_graphics_placement_upsert_preserves_host_placement_id() {
        let (mut scene, _) = scene();
        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![
                        PluginGraphicsOp::SetAsset {
                            asset_id: 1,
                            source: rgba(4, 4),
                        },
                        PluginGraphicsOp::PlaceImage {
                            placement_id: 1,
                            asset_id: 1,
                            destination: destination(1, 1),
                            source: None,
                            z_index: 0,
                        },
                    ],
                },
                Path::new("."),
            )
            .unwrap();
        let first = scene.placements.get(&1).unwrap().host_placement_id;

        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![PluginGraphicsOp::PlaceImage {
                        placement_id: 1,
                        asset_id: 1,
                        destination: destination(2, 2),
                        source: None,
                        z_index: 1,
                    }],
                },
                Path::new("."),
            )
            .unwrap();

        assert_eq!(scene.placements.get(&1).unwrap().host_placement_id, first);
    }

    #[test]
    fn plugin_graphics_placement_clips_to_viewport_without_deleting() {
        let (mut scene, _) = scene();
        scene
            .apply_update(
                PluginGraphicsUpdate {
                    ops: vec![
                        PluginGraphicsOp::SetAsset {
                            asset_id: 1,
                            source: rgba(100, 100),
                        },
                        PluginGraphicsOp::PlaceImage {
                            placement_id: 1,
                            asset_id: 1,
                            destination: PluginCellRect {
                                x: 2,
                                y: 1,
                                columns: 4,
                                rows: 4,
                            },
                            source: None,
                            z_index: 0,
                        },
                    ],
                },
                Path::new("."),
            )
            .unwrap();

        let small = scene.visible_kitty_render_bundle(0, 0, 4, 3);
        let chunk = &small.explicit_chunks[0];
        assert_eq!(chunk.cell_x, 2);
        assert_eq!(chunk.cell_y, 1);
        assert_eq!(chunk.columns, 2);
        assert_eq!(chunk.rows, 2);
        assert_eq!(chunk.source_width, 50);
        assert_eq!(chunk.source_height, 50);

        assert!(scene
            .visible_kitty_render_bundle(0, 0, 1, 1)
            .explicit_chunks
            .is_empty());
        assert!(!scene.placements.is_empty());
        assert_eq!(
            scene
                .visible_kitty_render_bundle(0, 0, 10, 10)
                .explicit_chunks
                .len(),
            1
        );
    }
}
