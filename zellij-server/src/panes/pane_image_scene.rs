use std::collections::HashMap;

use zellij_utils::pane_size::SizeInPixels;

use crate::output::KittyImageChunk;
use crate::panes::kitty::{KittyImageInsertion, KittyImageState};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageAssetId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LogicalPlacementId(pub u64);

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
pub struct ImagePlacement {
    pub logical_placement_id: LogicalPlacementId,
    pub asset_id: ImageAssetId,
    pub protocol_identity: Option<ProtocolPlacementIdentity>,
    pub anchor: FlowAnchor,
    pub occupancy: PlacementOccupancy,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub z_index: i32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub content_flow: ImageContentFlow,
}

impl ImagePlacement {
    pub fn anchor_logical_row(&self) -> Option<usize> {
        self.anchor.logical_row()
    }

    pub fn anchor_column(&self) -> Option<usize> {
        self.anchor.column()
    }

    pub fn rows(&self) -> usize {
        self.occupancy.rows
    }

    pub fn columns(&self) -> usize {
        self.occupancy.columns
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
    pub image_id: u32,
    pub placement_id: Option<u32>,
    pub placeholder_row: u16,
    pub placeholder_col: u16,
    pub anchor: FlowAnchor,
}

#[derive(Clone, Debug, Default)]
pub struct PaneImageScene {
    kitty: KittyImageState,
    kitty_placeholder_cells: Vec<KittyPlaceholderCell>,
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
                ImageInsertionEffect {
                    placement: ImagePlacement {
                        logical_placement_id: next_logical_placement_id(),
                        asset_id: insertion.asset_id,
                        protocol_identity,
                        anchor,
                        occupancy: PlacementOccupancy {
                            columns: geometry.columns,
                            rows: geometry.rows,
                        },
                        source_x: geometry.source_x,
                        source_y: geometry.source_y,
                        source_width: geometry.source_width,
                        source_height: geometry.source_height,
                        z_index: geometry.z_index,
                        x_offset: geometry.x_offset,
                        y_offset: geometry.y_offset,
                        content_flow,
                    },
                }
            })
    }

    pub fn add_kitty_placeholder_cell(&mut self, placeholder_cell: KittyPlaceholderCell) {
        self.kitty_placeholder_cells.push(placeholder_cell);
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
        let mut chunks = self.kitty.visible_chunks(
            content_x,
            content_y,
            scrollback_size_in_lines,
            viewport_width,
            viewport_height,
            character_cell_size,
            &resolve_anchor,
        );

        let mut grouped_placeholder_cells: HashMap<(u32, Option<u32>), Vec<&KittyPlaceholderCell>> =
            HashMap::new();
        for placeholder_cell in &self.kitty_placeholder_cells {
            grouped_placeholder_cells
                .entry((placeholder_cell.image_id, placeholder_cell.placement_id))
                .or_default()
                .push(placeholder_cell);
        }

        for ((image_id, placement_id), placeholder_cells) in grouped_placeholder_cells {
            let Some((image_width, image_height)) = self.kitty.image_dimensions(image_id) else {
                continue;
            };
            let Some(image_data) = self.kitty.image_chunk_data(image_id) else {
                continue;
            };

            let mut resolved_cells = vec![];
            for cell in placeholder_cells {
                let Some((logical_row, column)) = resolve_anchor(&cell.anchor) else {
                    continue;
                };
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

            let top_left_logical_row = resolved_cells
                .iter()
                .map(|(logical_row, _, placeholder_row, _)| logical_row.saturating_sub(*placeholder_row))
                .min()
                .unwrap_or(0);
            let top_left_column = resolved_cells
                .iter()
                .map(|(_, column, _, placeholder_col)| column.saturating_sub(*placeholder_col))
                .min()
                .unwrap_or(0);
            let columns = resolved_cells
                .iter()
                .map(|(_, _, _, placeholder_col)| *placeholder_col)
                .max()
                .unwrap_or(0)
                + 1;
            let rows = resolved_cells
                .iter()
                .map(|(_, _, placeholder_row, _)| *placeholder_row)
                .max()
                .unwrap_or(0)
                + 1;

            let Some(projection) = project_placement_to_viewport(
                top_left_logical_row,
                top_left_column,
                &PlacementOccupancy { columns, rows },
                content_x,
                content_y,
                scrollback_size_in_lines,
                viewport_width,
                viewport_height,
            ) else {
                continue;
            };

            let source_x = if projection.clipped_left_cols > 0 {
                ((image_width as u64 * projection.clipped_left_cols as u64) / columns as u64) as u32
            } else {
                0
            };
            let source_y = if projection.clipped_top_rows > 0 {
                ((image_height as u64 * projection.clipped_top_rows as u64) / rows as u64) as u32
            } else {
                0
            };
            let source_width =
                ((image_width as u64 * projection.columns as u64) / columns as u64) as u32;
            let source_height =
                ((image_height as u64 * projection.rows as u64) / rows as u64) as u32;

            chunks.push(KittyImageChunk {
                image_id,
                placement_id,
                cell_x: projection.cell_x,
                cell_y: projection.cell_y,
                columns: projection.columns,
                rows: projection.rows,
                source_x,
                source_y,
                source_width,
                source_height,
                z_index: 0,
                x_offset: 0,
                y_offset: 0,
                image_data,
            });
        }

        chunks
    }

    pub fn clear(&mut self) {
        self.kitty.clear();
        self.kitty_placeholder_cells.clear();
    }
}
