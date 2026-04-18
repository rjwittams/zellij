use std::collections::HashMap;

use zellij_utils::pane_size::SizeInPixels;

use crate::output::{
    KittyImageChunk, KittyImagePlacementMode, KittyPlaceholderCellRender, KittyPlaceholderRender,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KittyRenderBundle {
    pub explicit_chunks: Vec<KittyImageChunk>,
    pub placeholder_renders: Vec<KittyPlaceholderRender>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageRowSpan {
    pub start_row: usize,
    pub line_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KittyDamageRedraw {
    row_spans: Vec<DamageRowSpan>,
}

impl KittyDamageRedraw {
    pub fn from_changed_rects(changed_rects: HashMap<usize, usize>) -> Self {
        let mut row_spans: Vec<DamageRowSpan> = changed_rects
            .into_iter()
            .filter_map(|(start_row, line_count)| {
                if line_count == 0 {
                    None
                } else {
                    Some(DamageRowSpan {
                        start_row,
                        line_count,
                    })
                }
            })
            .collect();
        row_spans.sort_by_key(|span| span.start_row);
        KittyDamageRedraw { row_spans }
    }

    pub fn is_empty(&self) -> bool {
        self.row_spans.is_empty()
    }

    pub fn row_spans(&self) -> &[DamageRowSpan] {
        &self.row_spans
    }

    pub fn intersects_absolute_rows(
        &self,
        content_y: usize,
        absolute_y: usize,
        row_count: usize,
    ) -> bool {
        let item_start = absolute_y;
        let item_end = absolute_y + row_count.max(1);
        self.row_spans.iter().any(|span| {
            let changed_start = content_y + span.start_row;
            let changed_end = changed_start + span.line_count;
            item_start < changed_end && changed_start < item_end
        })
    }
}

use crate::panes::kitty::{KittyApcEffect, KittyImageInsertion, KittyImageState};
use crate::panes::kitty_asset_store::KittyAssetStore;
use std::cell::RefCell;
use std::rc::Rc;
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
    pub cleared_placeholder_rows: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageSceneEffect {
    Placement(ImageInsertionEffect),
    AssetReplaced {
        cleared_placeholder_rows: Vec<usize>,
    },
    AssetStored,
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

#[derive(Clone, Debug)]
pub struct PaneImageScene {
    kitty: KittyImageState,
    placements: HashMap<LogicalPlacementId, ImagePlacement>,
    kitty_placeholder_cells: Vec<KittyPlaceholderCell>,
    kitty_logical_placement_ids: HashMap<KittyProtocolPlacementKey, LogicalPlacementId>,
}

impl PaneImageScene {
    pub fn empty_clone(&self) -> Self {
        Self::new(self.kitty.kitty_asset_store())
    }

    fn remove_kitty_asset_placements<F>(
        &mut self,
        asset_id: ImageAssetId,
        scrollback_size_in_lines: usize,
        viewport_height: usize,
        resolve_anchor: &F,
    ) -> Vec<usize>
    where
        F: Fn(&FlowAnchor) -> Option<(usize, usize)>,
    {
        let logical_placement_ids_to_remove: Vec<_> = self
            .placements
            .iter()
            .filter_map(|(logical_placement_id, placement)| {
                (placement.asset_id == asset_id).then_some(*logical_placement_id)
            })
            .collect();
        let mut cleared_placeholder_rows: Vec<_> = self
            .kitty_placeholder_cells
            .iter()
            .filter_map(|placeholder_cell| {
                if !logical_placement_ids_to_remove
                    .contains(&placeholder_cell.logical_placement_id)
                {
                    return None;
                }
                let (logical_row, _column) = resolve_anchor(&placeholder_cell.anchor)?;
                let viewport_row = logical_row.checked_sub(scrollback_size_in_lines)?;
                (viewport_row < viewport_height).then_some(viewport_row)
            })
            .collect();
        cleared_placeholder_rows.sort_unstable();
        cleared_placeholder_rows.dedup();
        self.placements.retain(|logical_placement_id, _| {
            !logical_placement_ids_to_remove.contains(logical_placement_id)
        });
        self.kitty_placeholder_cells.retain(|placeholder_cell| {
            !logical_placement_ids_to_remove.contains(&placeholder_cell.logical_placement_id)
        });
        self.kitty_logical_placement_ids
            .retain(|_, logical_placement_id| {
                !logical_placement_ids_to_remove.contains(logical_placement_id)
            });
        cleared_placeholder_rows
    }

    pub fn new(kitty_asset_store: Rc<RefCell<KittyAssetStore>>) -> Self {
        Self {
            kitty: KittyImageState::new(kitty_asset_store),
            placements: HashMap::new(),
            kitty_placeholder_cells: vec![],
            kitty_logical_placement_ids: HashMap::new(),
        }
    }

    pub fn rows_for_pixel_height(
        pixel_height: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> usize {
        let Some(character_cell_size) = character_cell_size else {
            return 0;
        };
        (pixel_height as f64 / character_cell_size.height as f64).ceil() as usize
    }

    pub fn handle_kitty_apc<F>(
        &mut self,
        apc_bytes: &[u8],
        anchor: FlowAnchor,
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
        scrollback_size_in_lines: usize,
        viewport_height: usize,
        resolve_anchor: F,
    ) -> Option<ImageSceneEffect>
    where
        F: Fn(&FlowAnchor) -> Option<(usize, usize)>,
    {
        self.kitty
            .handle_apc(
                apc_bytes,
                anchor.clone(),
                cursor_x,
                scrollback_row,
                character_cell_size,
            )
            .map(|effect| match effect {
                KittyApcEffect::AssetReplaced { asset_id } => {
                    let cleared_placeholder_rows = self.remove_kitty_asset_placements(
                        asset_id,
                        scrollback_size_in_lines,
                        viewport_height,
                        &resolve_anchor,
                    );
                    ImageSceneEffect::AssetReplaced {
                        cleared_placeholder_rows,
                    }
                },
                KittyApcEffect::AssetStored => ImageSceneEffect::AssetStored,
                KittyApcEffect::Placement(insertion) => {
                    let insertion: KittyImageInsertion = insertion;
                    let cleared_placeholder_rows = if insertion.replaced_existing_asset {
                        self.remove_kitty_asset_placements(
                            insertion.asset_id,
                            scrollback_size_in_lines,
                            viewport_height,
                            &resolve_anchor,
                        )
                    } else {
                        vec![]
                    };
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
                        KittyImagePlacementMode::Explicit => {
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
                        KittyImagePlacementMode::Placeholder => {
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
                    if let Some(kitty_protocol_placement_key) =
                        protocol_identity
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
                                placeholder_cell.logical_placement_id
                                    != previous_logical_placement_id
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
                    ImageSceneEffect::Placement(ImageInsertionEffect {
                        placement,
                        cleared_placeholder_rows,
                    })
                },
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

    pub fn kitty_placeholder_anchors_in_range<F>(
        &self,
        logical_row: usize,
        start_column: usize,
        end_column_exclusive: usize,
        resolve_anchor: F,
    ) -> Vec<FlowAnchor>
    where
        F: Fn(&FlowAnchor) -> Option<(usize, usize)>,
    {
        self.kitty_placeholder_cells
            .iter()
            .filter_map(|placeholder_cell| {
                let (cell_logical_row, cell_column) = resolve_anchor(&placeholder_cell.anchor)?;
                if cell_logical_row == logical_row
                    && cell_column >= start_column
                    && cell_column < end_column_exclusive
                {
                    Some(placeholder_cell.anchor.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn delete_kitty_protocol_placement(
        &mut self,
        protocol_image_id: u32,
        placement_id: Option<u32>,
    ) {
        let logical_placement_ids_to_remove: Vec<_> = self
            .placements
            .iter()
            .filter_map(
                |(logical_placement_id, placement)| match &placement.protocol_identity {
                    Some(ProtocolPlacementIdentity::Kitty {
                        image_id: Some(image_id),
                        placement_id: existing_placement_id,
                    }) if *image_id == protocol_image_id
                        && (placement_id.is_none() || *existing_placement_id == placement_id) =>
                    {
                        Some(*logical_placement_id)
                    },
                    _ => None,
                },
            )
            .collect();
        self.placements.retain(|logical_placement_id, _| {
            !logical_placement_ids_to_remove.contains(logical_placement_id)
        });
        self.kitty_placeholder_cells.retain(|placeholder_cell| {
            !logical_placement_ids_to_remove.contains(&placeholder_cell.logical_placement_id)
        });
        self.kitty_logical_placement_ids
            .retain(|key, logical_placement_id| {
                !(key.image_id == protocol_image_id
                    && (placement_id.is_none() || key.placement_id == placement_id)
                    || logical_placement_ids_to_remove.contains(logical_placement_id))
            });
        self.kitty
            .delete_protocol_placement(protocol_image_id, placement_id);
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
            let Some((_protocol_image_id, placement_id)) = placement.kitty_protocol_identity()
            else {
                continue;
            };
            let image_id = placement.kitty_internal_image_id();
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
                placement_mode: KittyImagePlacementMode::Explicit,
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
            let Some((_protocol_image_id, placement_id)) =
                logical_placement.kitty_protocol_identity()
            else {
                continue;
            };
            let image_id = logical_placement.kitty_internal_image_id();
            let Some((image_width, image_height)) = self.kitty_image_dimensions(image_id) else {
                continue;
            };
            let Some(flavor) = logical_placement.kitty_placeholder_flavor() else {
                continue;
            };

            let mut resolved_cells = vec![];
            let mut unresolved_anchor_count = 0;
            let mut clipped_by_width_count = 0;
            let mut clipped_by_height_count = 0;
            for cell in &placeholder_cells {
                let Some((logical_row, column)) = resolve_anchor(&cell.anchor) else {
                    unresolved_anchor_count += 1;
                    continue;
                };
                if column >= viewport_width {
                    clipped_by_width_count += 1;
                    continue;
                }
                if logical_row < scrollback_size_in_lines
                    || logical_row >= scrollback_size_in_lines + viewport_height
                {
                    clipped_by_height_count += 1;
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
                log::debug!(
                    "kitty placeholder render empty: placement={:?} total_cells={} unresolved={} clipped_width={} clipped_height={} viewport={}x{} scrollback={}",
                    logical_placement.kitty_protocol_identity(),
                    placeholder_cells.len(),
                    unresolved_anchor_count,
                    clipped_by_width_count,
                    clipped_by_height_count,
                    viewport_width,
                    viewport_height,
                    scrollback_size_in_lines,
                );
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
                ((base_source_width as u64 * visible_columns as u64) / total_columns as u64) as u32;
            let source_height =
                ((base_source_height as u64 * visible_rows as u64) / total_rows as u64) as u32;
            let cells: Vec<KittyPlaceholderCellRender> = resolved_cells
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
            log::debug!(
                "kitty placeholder render: placement={:?} total_cells={} resolved_cells={} min_row={} max_row={} min_col={} max_col={} source=({},{} {}x{}) viewport={}x{} scrollback={}",
                logical_placement.kitty_protocol_identity(),
                placeholder_cells.len(),
                cells.len(),
                min_placeholder_row,
                max_placeholder_row,
                min_placeholder_col,
                max_placeholder_col,
                source_x,
                source_y,
                source_width,
                source_height,
                viewport_width,
                viewport_height,
                scrollback_size_in_lines,
            );
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
                cells,
            });
        }

        let bundle = KittyRenderBundle {
            explicit_chunks,
            placeholder_renders,
        };
        bundle
    }

    pub fn visible_kitty_render_bundle_for_damage_redraw<F>(
        &self,
        damage_redraw: &KittyDamageRedraw,
        content_x: usize,
        content_y: usize,
        scrollback_size_in_lines: usize,
        viewport_width: usize,
        viewport_height: usize,
        character_cell_size: Option<SizeInPixels>,
        resolve_anchor: F,
    ) -> KittyRenderBundle
    where
        F: Fn(&FlowAnchor) -> Option<(usize, usize)>,
    {
        if damage_redraw.is_empty() {
            return KittyRenderBundle::default();
        }
        let visible_bundle = self.visible_kitty_render_bundle(
            content_x,
            content_y,
            scrollback_size_in_lines,
            viewport_width,
            viewport_height,
            character_cell_size,
            resolve_anchor,
        );
        let explicit_chunks = visible_bundle
            .explicit_chunks
            .into_iter()
            .filter(|chunk| {
                damage_redraw.intersects_absolute_rows(content_y, chunk.cell_y, chunk.rows)
            })
            .collect();
        let placeholder_renders = visible_bundle
            .placeholder_renders
            .into_iter()
            .filter(|render| {
                render
                    .cells
                    .iter()
                    .any(|cell| damage_redraw.intersects_absolute_rows(content_y, cell.cell_y, 1))
            })
            .collect();
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
