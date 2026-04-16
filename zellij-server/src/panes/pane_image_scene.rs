use std::collections::HashMap;

use zellij_utils::pane_size::SizeInPixels;

use crate::output::{KittyImageChunk, KittyPlaceholderCellRender, KittyPlaceholderRender};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KittyRenderBundle {
    pub explicit_chunks: Vec<KittyImageChunk>,
    pub placeholder_renders: Vec<KittyPlaceholderRender>,
}
use crate::panes::kitty::{KittyImageInsertion, KittyImageState};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageAssetId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LogicalPlacementId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KittyProtocolPlacementKey {
    pub image_id: u32,
    pub placement_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtocolPlacementIdentity {
    Kitty {
        image_id: Option<u32>,
        placement_id: Option<u32>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlowAnchor {
    LogicalRow {
        logical_row: usize,
        column: usize,
    },
    CanonicalLine {
        canonical_line_index: usize,
        offset_in_line: usize,
    },
}

impl FlowAnchor {
    pub fn logical_row(&self) -> Option<usize> {
        match self {
            FlowAnchor::LogicalRow { logical_row, .. } => Some(*logical_row),
            FlowAnchor::CanonicalLine { .. } => None,
        }
    }

    pub fn column(&self) -> Option<usize> {
        match self {
            FlowAnchor::LogicalRow { column, .. } => Some(*column),
            FlowAnchor::CanonicalLine { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacementOccupancy {
    pub columns: usize,
    pub rows: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageContentFlow {
    NoCursorMovement,
    MoveCursorByCells { rows: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagePlacementGeometry {
    pub anchor_x: usize,
    pub logical_row: usize,
    pub columns: usize,
    pub rows: usize,
    pub columns_specified: bool,
    pub rows_specified: bool,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub z_index: i32,
    pub x_offset: u32,
    pub y_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewportProjection {
    pub cell_x: usize,
    pub cell_y: usize,
    pub columns: usize,
    pub rows: usize,
    pub clipped_left_cols: usize,
    pub clipped_top_rows: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyExplicitPlacementFlavor {
    pub occupancy: PlacementOccupancy,
    pub columns_specified: bool,
    pub rows_specified: bool,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub z_index: i32,
    pub x_offset: u32,
    pub y_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyVirtualPlacementFlavor {
    pub occupancy: PlacementOccupancy,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlacementFlavor {
    KittyExplicit(KittyExplicitPlacementFlavor),
    KittyPlaceholder(KittyVirtualPlacementFlavor),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagePlacement {
    pub logical_placement_id: LogicalPlacementId,
    pub asset_id: ImageAssetId,
    pub protocol_identity: Option<ProtocolPlacementIdentity>,
    pub anchor: FlowAnchor,
    pub flavor: PlacementFlavor,
    pub content_flow: ImageContentFlow,
}

impl ImagePlacement {
    pub fn anchor_logical_row(&self) -> Option<usize> {
        self.anchor.logical_row()
    }

    pub fn anchor_column(&self) -> Option<usize> {
        self.anchor.column()
    }

    pub fn occupancy(&self) -> &PlacementOccupancy {
        match &self.flavor {
            PlacementFlavor::KittyExplicit(flavor) => &flavor.occupancy,
            PlacementFlavor::KittyPlaceholder(flavor) => &flavor.occupancy,
        }
    }

    pub fn kitty_explicit_flavor(&self) -> Option<&KittyExplicitPlacementFlavor> {
        match &self.flavor {
            PlacementFlavor::KittyExplicit(flavor) => Some(flavor),
            _ => None,
        }
    }

    pub fn kitty_placeholder_flavor(&self) -> Option<&KittyVirtualPlacementFlavor> {
        match &self.flavor {
            PlacementFlavor::KittyPlaceholder(flavor) => Some(flavor),
            _ => None,
        }
    }

    pub fn kitty_protocol_identity(&self) -> Option<(Option<u32>, Option<u32>)> {
        match &self.protocol_identity {
            Some(ProtocolPlacementIdentity::Kitty {
                image_id,
                placement_id,
            }) => Some((*image_id, *placement_id)),
            _ => None,
        }
    }

    pub fn kitty_protocol_placement_key(&self) -> Option<KittyProtocolPlacementKey> {
        match &self.protocol_identity {
            Some(ProtocolPlacementIdentity::Kitty {
                image_id: Some(image_id),
                placement_id,
            }) => Some(KittyProtocolPlacementKey {
                image_id: *image_id,
                placement_id: *placement_id,
            }),
            _ => None,
        }
    }

    pub fn kitty_internal_image_id(&self) -> u32 {
        self.asset_id.0 as u32
    }

    pub fn rows(&self) -> usize {
        self.occupancy().rows
    }

    pub fn columns(&self) -> usize {
        self.occupancy().columns
    }
}

fn scale_u32(total: u32, kept: usize, original: usize) -> u32 {
    if original == 0 {
        0
    } else {
        ((total as u64 * kept as u64) / original as u64) as u32
    }
}

pub fn project_placement_to_viewport(
    logical_row: usize,
    column: usize,
    occupancy: &PlacementOccupancy,
    content_x: usize,
    content_y: usize,
    scrollback_size_in_lines: usize,
    viewport_width: usize,
    viewport_height: usize,
) -> Option<ViewportProjection> {
    let placement_left = column as isize;
    let placement_right = placement_left + occupancy.columns as isize;
    let viewport_left = 0isize;
    let viewport_right = viewport_width as isize;
    let visible_left = std::cmp::max(placement_left, viewport_left);
    let visible_right = std::cmp::min(placement_right, viewport_right);
    if visible_left >= visible_right {
        return None;
    }

    let placement_top = logical_row as isize;
    let placement_bottom = placement_top + occupancy.rows as isize;
    let viewport_top = scrollback_size_in_lines as isize;
    let viewport_bottom = viewport_top + viewport_height as isize;
    let visible_top = std::cmp::max(placement_top, viewport_top);
    let visible_bottom = std::cmp::min(placement_bottom, viewport_bottom);
    if visible_top >= visible_bottom {
        return None;
    }

    let clipped_left_cols = (visible_left - placement_left) as usize;
    let clipped_right_cols = (placement_right - visible_right) as usize;
    let columns = occupancy
        .columns
        .saturating_sub(clipped_left_cols)
        .saturating_sub(clipped_right_cols);
    if columns == 0 {
        return None;
    }

    let clipped_top_rows = (visible_top - placement_top) as usize;
    let clipped_bottom_rows = (placement_bottom - visible_bottom) as usize;
    let rows = occupancy
        .rows
        .saturating_sub(clipped_top_rows)
        .saturating_sub(clipped_bottom_rows);
    if rows == 0 {
        return None;
    }

    Some(ViewportProjection {
        cell_x: content_x + (visible_left - viewport_left) as usize,
        cell_y: content_y + (visible_top - viewport_top) as usize,
        columns,
        rows,
        clipped_left_cols,
        clipped_top_rows,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageInsertionEffect {
    pub placement: ImagePlacement,
}

static NEXT_IMAGE_ASSET_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_LOGICAL_PLACEMENT_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_image_asset_id() -> ImageAssetId {
    ImageAssetId(NEXT_IMAGE_ASSET_ID.fetch_add(1, Ordering::Relaxed))
}

pub fn next_logical_placement_id() -> LogicalPlacementId {
    LogicalPlacementId(NEXT_LOGICAL_PLACEMENT_ID.fetch_add(1, Ordering::Relaxed))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyPlaceholderCell {
    pub logical_placement_id: LogicalPlacementId,
    pub placeholder_row: u16,
    pub placeholder_col: u16,
    pub anchor: FlowAnchor,
}

#[derive(Clone, Debug, Default)]
pub struct PaneImageScene {
    kitty: KittyImageState,
    placements: HashMap<LogicalPlacementId, ImagePlacement>,
    kitty_placeholder_cells: Vec<KittyPlaceholderCell>,
    kitty_logical_placement_ids: HashMap<KittyProtocolPlacementKey, LogicalPlacementId>,
}

impl PaneImageScene {
    pub fn rows_for_pixel_height(
        pixel_height: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> usize {
        let Some(character_cell_size) = character_cell_size else {
            return 0;
        };
        (pixel_height as f64 / character_cell_size.height as f64).ceil() as usize
    }

    pub fn handle_kitty_apc(
        &mut self,
        apc_bytes: &[u8],
        anchor: FlowAnchor,
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> Option<ImageInsertionEffect> {
        self.kitty
            .handle_apc(apc_bytes, anchor.clone(), cursor_x, scrollback_row, character_cell_size)
            .map(|insertion: KittyImageInsertion| {
                let geometry = insertion.geometry;
                let content_flow = if geometry.rows > 0 {
                    ImageContentFlow::MoveCursorByCells {
                        rows: geometry.rows,
                    }
                } else {
                    ImageContentFlow::NoCursorMovement
                };
                let protocol_identity = Some(ProtocolPlacementIdentity::Kitty {
                    image_id: insertion.protocol_image_id,
                    placement_id: insertion.protocol_placement_id,
                });
                let logical_placement_id = next_logical_placement_id();
                let flavor = match insertion.placement_mode {
                    crate::output::KittyImagePlacementMode::Explicit => {
                        PlacementFlavor::KittyExplicit(KittyExplicitPlacementFlavor {
                            occupancy: PlacementOccupancy {
                                columns: geometry.columns,
                                rows: geometry.rows,
                            },
                            columns_specified: geometry.columns_specified,
                            rows_specified: geometry.rows_specified,
                            source_x: geometry.source_x,
                            source_y: geometry.source_y,
                            source_width: geometry.source_width,
                            source_height: geometry.source_height,
                            z_index: geometry.z_index,
                            x_offset: geometry.x_offset,
                            y_offset: geometry.y_offset,
                        })
                    },
                    crate::output::KittyImagePlacementMode::Placeholder => {
                        PlacementFlavor::KittyPlaceholder(KittyVirtualPlacementFlavor {
                            occupancy: PlacementOccupancy {
                                columns: geometry.columns,
                                rows: geometry.rows,
                            },
                            source_x: geometry.source_x,
                            source_y: geometry.source_y,
                            source_width: geometry.source_width,
                            source_height: geometry.source_height,
                            x_offset: geometry.x_offset,
                            y_offset: geometry.y_offset,
                        })
                    },
                };
                if let Some(kitty_protocol_placement_key) = protocol_identity
                    .as_ref()
                    .and_then(|protocol_identity| match protocol_identity {
                        ProtocolPlacementIdentity::Kitty {
                            image_id: Some(image_id),
                            placement_id,
                        } => Some(KittyProtocolPlacementKey {
                            image_id: *image_id,
                            placement_id: *placement_id,
                        }),
                        _ => None,
                    })
                {
                    if let Some(previous_logical_placement_id) = self
                        .kitty_logical_placement_ids
                        .insert(kitty_protocol_placement_key, logical_placement_id)
                    {
                    self.placements.remove(&previous_logical_placement_id);
                    self.kitty_placeholder_cells.retain(|placeholder_cell| {
                        placeholder_cell.logical_placement_id != previous_logical_placement_id
                    });
                    }
                }
                let placement = ImagePlacement {
                    logical_placement_id,
                    asset_id: insertion.asset_id,
                    protocol_identity,
                    anchor,
                    flavor,
                    content_flow,
                };
                self.placements
                    .insert(logical_placement_id, placement.clone());
                ImageInsertionEffect { placement }
            })
    }

    pub fn kitty_logical_placement_id(
        &self,
        image_id: u32,
        placement_id: Option<u32>,
    ) -> Option<LogicalPlacementId> {
        self.kitty_logical_placement_ids
            .get(&KittyProtocolPlacementKey {
                image_id,
                placement_id,
            })
            .copied()
    }

    pub fn placement(&self, logical_placement_id: LogicalPlacementId) -> Option<&ImagePlacement> {
        self.placements.get(&logical_placement_id)
    }

    pub fn kitty_image_chunk_data(&self, image_id: u32) -> Option<crate::output::KittyImageData> {
        self.kitty.image_chunk_data(image_id)
    }

    pub fn kitty_image_dimensions(&self, image_id: u32) -> Option<(u32, u32)> {
        self.kitty.image_dimensions(image_id)
    }

    pub fn add_kitty_placeholder_cell(&mut self, placeholder_cell: KittyPlaceholderCell) {
        self.kitty_placeholder_cells.push(placeholder_cell);
    }

    pub fn remove_kitty_placeholder_cell_at_anchor(&mut self, anchor: &FlowAnchor) {
        self.kitty_placeholder_cells
            .retain(|placeholder_cell| &placeholder_cell.anchor != anchor);
    }

    pub fn visible_kitty_render_bundle<F>(
        &self,
        content_x: usize,
        content_y: usize,
        scrollback_size_in_lines: usize,
        viewport_width: usize,
        viewport_height: usize,
        _character_cell_size: Option<SizeInPixels>,
        resolve_anchor: F,
    ) -> KittyRenderBundle
    where
        F: Fn(&FlowAnchor) -> Option<(usize, usize)>,
    {
        let mut explicit_chunks = vec![];
        for placement in self.placements.values() {
            let Some(flavor) = placement.kitty_explicit_flavor() else {
                continue;
            };
            let Some((_protocol_image_id, placement_id)) = placement.kitty_protocol_identity() else {
                continue;
            };
            let image_id = placement.kitty_internal_image_id();
            let Some(image_data) = self.kitty_image_chunk_data(image_id) else {
                continue;
            };
            let Some((logical_row, column)) = resolve_anchor(&placement.anchor) else {
                continue;
            };
            let mut source_x = flavor.source_x;
            let mut source_y = flavor.source_y;
            let mut source_width = flavor.source_width;
            let mut source_height = flavor.source_height;
            let mut columns = flavor.occupancy.columns;
            let mut rows = flavor.occupancy.rows;
            if columns == 0 || rows == 0 {
                continue;
            }
            let Some(projection) = project_placement_to_viewport(
                logical_row,
                column,
                &flavor.occupancy,
                content_x,
                content_y,
                scrollback_size_in_lines,
                viewport_width,
                viewport_height,
            ) else {
                continue;
            };
            if projection.clipped_left_cols > 0 {
                source_x += scale_u32(source_width, projection.clipped_left_cols, columns);
            }
            source_width = scale_u32(source_width, projection.columns, columns);
            columns = projection.columns;
            if projection.clipped_top_rows > 0 {
                source_y += scale_u32(source_height, projection.clipped_top_rows, rows);
            }
            source_height = scale_u32(source_height, projection.rows, rows);
            rows = projection.rows;
            let serialized_columns = if flavor.columns_specified { columns } else { 0 };
            let serialized_rows = if flavor.rows_specified { rows } else { 0 };
            explicit_chunks.push(KittyImageChunk {
                image_id,
                placement_id,
                placement_mode: crate::output::KittyImagePlacementMode::Explicit,
                cell_x: projection.cell_x,
                cell_y: projection.cell_y,
                columns: serialized_columns,
                rows: serialized_rows,
                source_x,
                source_y,
                source_width,
                source_height,
                z_index: flavor.z_index,
                x_offset: flavor.x_offset,
                y_offset: flavor.y_offset,
                image_data,
            });
        }
        let mut placeholder_renders = vec![];
        let mut grouped_placeholder_cells: HashMap<LogicalPlacementId, Vec<&KittyPlaceholderCell>> =
            HashMap::new();
        for placeholder_cell in &self.kitty_placeholder_cells {
            grouped_placeholder_cells
                .entry(placeholder_cell.logical_placement_id)
                .or_default()
                .push(placeholder_cell);
        }

        log::debug!(
            "kitty placeholder render groups: {}",
            grouped_placeholder_cells.len()
        );
        for (logical_placement_id, placeholder_cells) in grouped_placeholder_cells {
            let Some(logical_placement) = self.placement(logical_placement_id) else {
                continue;
            };
            let Some((_protocol_image_id, placement_id)) = logical_placement.kitty_protocol_identity() else {
                continue;
            };
            let image_id = logical_placement.kitty_internal_image_id();
            let Some((image_width, image_height)) = self.kitty_image_dimensions(image_id) else {
                continue;
            };
            let Some(image_data) = self.kitty_image_chunk_data(image_id) else {
                continue;
            };
            let Some(flavor) = logical_placement.kitty_placeholder_flavor() else {
                continue;
            };

            let mut resolved_cells = vec![];
            for cell in placeholder_cells {
                let Some((logical_row, column)) = resolve_anchor(&cell.anchor) else {
                    continue;
                };
                if column >= viewport_width {
                    continue;
                }
                if logical_row < scrollback_size_in_lines
                    || logical_row >= scrollback_size_in_lines + viewport_height
                {
                    continue;
                }
                resolved_cells.push((
                    logical_row,
                    column,
                    cell.placeholder_row as usize,
                    cell.placeholder_col as usize,
                ));
            }
            if resolved_cells.is_empty() {
                continue;
            }

            let total_columns = flavor.occupancy.columns;
            let total_rows = flavor.occupancy.rows;
            if total_columns == 0 || total_rows == 0 {
                continue;
            }
            let min_placeholder_col = resolved_cells
                .iter()
                .map(|(_, _, _, placeholder_col)| *placeholder_col)
                .min()
                .unwrap_or(0);
            let min_placeholder_row = resolved_cells
                .iter()
                .map(|(_, _, placeholder_row, _)| *placeholder_row)
                .min()
                .unwrap_or(0);
            let max_placeholder_col = resolved_cells
                .iter()
                .map(|(_, _, _, placeholder_col)| *placeholder_col)
                .max()
                .unwrap_or(0);
            let max_placeholder_row = resolved_cells
                .iter()
                .map(|(_, _, placeholder_row, _)| *placeholder_row)
                .max()
                .unwrap_or(0);
            let visible_columns = max_placeholder_col.saturating_sub(min_placeholder_col) + 1;
            let visible_rows = max_placeholder_row.saturating_sub(min_placeholder_row) + 1;

            let base_source_x = flavor.source_x;
            let base_source_y = flavor.source_y;
            let base_source_width = if flavor.source_width == 0 {
                image_width.saturating_sub(base_source_x)
            } else {
                flavor.source_width
            };
            let base_source_height = if flavor.source_height == 0 {
                image_height.saturating_sub(base_source_y)
            } else {
                flavor.source_height
            };
            let source_x = base_source_x
                + ((base_source_width as u64 * min_placeholder_col as u64) / total_columns as u64)
                    as u32;
            let source_y = base_source_y
                + ((base_source_height as u64 * min_placeholder_row as u64) / total_rows as u64)
                    as u32;
            let source_width =
                ((base_source_width as u64 * visible_columns as u64) / total_columns as u64)
                    as u32;
            let source_height =
                ((base_source_height as u64 * visible_rows as u64) / total_rows as u64) as u32;
            let cells = resolved_cells
                .into_iter()
                .map(|(logical_row, column, placeholder_row, placeholder_col)| {
                    KittyPlaceholderCellRender {
                        cell_x: content_x + column,
                        cell_y: content_y + (logical_row - scrollback_size_in_lines),
                        placeholder_row,
                        placeholder_col,
                    }
                })
                .collect();
            placeholder_renders.push(KittyPlaceholderRender {
                image_id,
                placement_id,
                columns: total_columns,
                rows: total_rows,
                source_x,
                source_y,
                source_width,
                source_height,
                x_offset: flavor.x_offset,
                y_offset: flavor.y_offset,
                image_data,
                cells,
            });
        }

        KittyRenderBundle {
            explicit_chunks,
            placeholder_renders,
        }
    }

    pub fn visible_kitty_image_chunks<F>(
        &self,
        content_x: usize,
        content_y: usize,
        scrollback_size_in_lines: usize,
        viewport_width: usize,
        viewport_height: usize,
        character_cell_size: Option<SizeInPixels>,
        resolve_anchor: F,
    ) -> Vec<KittyImageChunk>
    where
        F: Fn(&FlowAnchor) -> Option<(usize, usize)>,
    {
        self.visible_kitty_render_bundle(
            content_x,
            content_y,
            scrollback_size_in_lines,
            viewport_width,
            viewport_height,
            character_cell_size,
            resolve_anchor,
        )
        .explicit_chunks
    }

    pub fn visible_kitty_placeholder_renders<F>(
        &self,
        content_x: usize,
        content_y: usize,
        scrollback_size_in_lines: usize,
        viewport_width: usize,
        viewport_height: usize,
        character_cell_size: Option<SizeInPixels>,
        resolve_anchor: F,
    ) -> Vec<KittyPlaceholderRender>
    where
        F: Fn(&FlowAnchor) -> Option<(usize, usize)>,
    {
        self.visible_kitty_render_bundle(
            content_x,
            content_y,
            scrollback_size_in_lines,
            viewport_width,
            viewport_height,
            character_cell_size,
            resolve_anchor,
        )
        .placeholder_renders
    }

    pub fn clear(&mut self) {
        self.kitty.clear();
        self.placements.clear();
        self.kitty_placeholder_cells.clear();
        self.kitty_logical_placement_ids.clear();
    }
}
