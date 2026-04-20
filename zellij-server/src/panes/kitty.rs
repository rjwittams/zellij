use base64;
use miniz_oxide::inflate::decompress_to_vec_zlib;
use zellij_utils::pane_size::SizeInPixels;

use crate::output::{
    KittyImageChunk, KittyImageData, KittyImagePlacementMode, KittyPlaceholderRender, PlacementId,
};
use crate::panes::kitty_asset_store::KittyAssetStore;
use crate::panes::kitty_placeholder::{
    kitty_diacritic_to_index, KITTY_ROWCOL_DIACRITICS, KITTY_UNICODE_PLACEHOLDER_CHAR,
};
use crate::panes::terminal_character::{AnsiCode, RcCharacterStyles};

use crate::panes::pane_image_scene::{
    project_placement_to_viewport, FlowAnchor, ImageAssetId, ImagePlacementGeometry,
    PlacementOccupancy,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub struct PendingKittyPlaceholder {
    image_id_low_bits: Option<u32>,
    placement_id: Option<PlacementId>,
    row_diacritic: Option<char>,
    column_diacritic: Option<char>,
    image_id_high_byte_diacritic: Option<char>,
    anchor: Option<FlowAnchor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedKittyPlaceholder {
    pub image_id: u32,
    pub placement_id: Option<PlacementId>,
    pub placeholder_row: u16,
    pub placeholder_col: u16,
    pub anchor: FlowAnchor,
}

impl PendingKittyPlaceholder {
    pub fn new(styles: &RcCharacterStyles, anchor: FlowAnchor) -> Self {
        Self {
            image_id_low_bits: kitty_placeholder_image_id_from_styles(styles),
            placement_id: kitty_placeholder_placement_id_from_styles(styles),
            anchor: Some(anchor),
            ..Default::default()
        }
    }

    pub fn absorb_diacritic(&mut self, c: char) -> bool {
        if self.row_diacritic.is_none() {
            self.row_diacritic = Some(c);
            true
        } else if self.column_diacritic.is_none() {
            self.column_diacritic = Some(c);
            true
        } else if self.image_id_high_byte_diacritic.is_none() {
            self.image_id_high_byte_diacritic = Some(c);
            true
        } else {
            false
        }
    }

    fn can_inherit_from(&self, previous: &ResolvedKittyPlaceholder) -> bool {
        let Some(anchor) = self.anchor.as_ref() else {
            return false;
        };
        if previous.placement_id != self.placement_id {
            return false;
        }
        if previous.image_id & 0x00FF_FFFF != self.image_id_low_bits.unwrap_or_default() {
            return false;
        }
        match (anchor, &previous.anchor) {
            (
                FlowAnchor::LogicalRow {
                    logical_row: current_row,
                    column: current_column,
                },
                FlowAnchor::LogicalRow {
                    logical_row: previous_row,
                    column: previous_column,
                },
            ) => {
                current_row == previous_row && *current_column == previous_column.saturating_add(1)
            },
            (
                FlowAnchor::CanonicalLine {
                    canonical_line_index: current_line,
                    offset_in_line: current_offset,
                },
                FlowAnchor::CanonicalLine {
                    canonical_line_index: previous_line,
                    offset_in_line: previous_offset,
                },
            ) => {
                current_line == previous_line
                    && *current_offset == previous_offset.saturating_add(1)
            },
            _ => false,
        }
    }

    pub fn resolve_with_previous(
        self,
        previous: Option<&ResolvedKittyPlaceholder>,
    ) -> Option<ResolvedKittyPlaceholder> {
        let inherited = previous.filter(|previous| self.can_inherit_from(previous));
        let anchor = self.anchor?;
        let image_id_low_bits = self.image_id_low_bits?;
        let placeholder_row = match self.row_diacritic {
            Some(row_diacritic) => kitty_diacritic_to_index(row_diacritic)? as u16,
            None => inherited.map(|previous| previous.placeholder_row)?,
        };
        let placeholder_col = match self.column_diacritic {
            Some(column_diacritic) => kitty_diacritic_to_index(column_diacritic)? as u16,
            None => inherited
                .map(|previous| previous.placeholder_col.saturating_add(1))
                .unwrap_or(0),
        };
        let image_id = if let Some(high_byte_diacritic) = self.image_id_high_byte_diacritic {
            let high_byte = kitty_diacritic_to_index(high_byte_diacritic)?;
            image_id_low_bits | ((high_byte as u32) << 24)
        } else {
            let inherited_high_byte = inherited.map(|previous| previous.image_id & 0xFF00_0000);
            image_id_low_bits | inherited_high_byte.unwrap_or(0)
        };
        Some(ResolvedKittyPlaceholder {
            image_id,
            placement_id: self.placement_id,
            placeholder_row,
            placeholder_col,
            anchor,
        })
    }

    pub fn resolve(self) -> Option<ResolvedKittyPlaceholder> {
        self.resolve_with_previous(None)
    }
}

fn kitty_placeholder_image_id_from_styles(styles: &RcCharacterStyles) -> Option<u32> {
    match styles.foreground {
        Some(AnsiCode::ColorIndex(index)) => Some(index as u32),
        Some(AnsiCode::RgbCode((r, g, b))) => {
            Some(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
        },
        _ => None,
    }
}

fn kitty_placeholder_placement_id_from_styles(styles: &RcCharacterStyles) -> Option<PlacementId> {
    match styles.underline_color {
        Some(AnsiCode::ColorIndex(index)) => Some(PlacementId::Protocol(index as u32)),
        Some(AnsiCode::RgbCode((r, g, b))) => {
            Some(PlacementId::Protocol(
                ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
            ))
        },
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct KittyPlacement {
    pub image_id: u32,
    pub placement_id: Option<PlacementId>,
    pub placement_mode: KittyImagePlacementMode,
    pub cursor_movement_policy: KittyCursorMovementPolicy,
    pub anchor: FlowAnchor,
    pub source_x: Option<u32>,
    pub source_y: Option<u32>,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
    pub columns: Option<u32>,
    pub rows: Option<u32>,
    pub columns_specified: bool,
    pub rows_specified: bool,
    pub x_offset: Option<u32>,
    pub y_offset: Option<u32>,
    pub z_index: Option<i32>,
}

impl Default for KittyPlacement {
    fn default() -> Self {
        Self {
            image_id: 0,
            placement_id: None,
            placement_mode: KittyImagePlacementMode::Explicit,
            cursor_movement_policy: KittyCursorMovementPolicy::AfterPlacement,
            anchor: FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            source_x: None,
            source_y: None,
            source_width: None,
            source_height: None,
            columns: None,
            rows: None,
            columns_specified: false,
            rows_specified: false,
            x_offset: None,
            y_offset: None,
            z_index: None,
        }
    }
}

#[derive(Clone, Debug)]
struct PendingKittyTransmit {
    protocol_image_id: Option<u32>,
    image_number: Option<u32>,
    image_id: u32,
    image_format: KittyImageFormat,
    compression: Option<KittyTransportCompression>,
    width: u32,
    height: u32,
    placement: Option<KittyPlacement>,
    reply_context: PendingKittyReplyContext,
    payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingKittyReplyKind {
    Transmit,
    Placement,
}

#[derive(Clone, Debug)]
struct PendingKittyReplyContext {
    kind: PendingKittyReplyKind,
    quiet: u8,
    parsed_image_id: Option<u32>,
    placement_id: Option<u32>,
    image_number: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KittyImageFormat {
    Png,
    Rgb,
    Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KittyTransportCompression {
    Zlib,
}

#[derive(Clone, Debug)]
pub struct KittyImageState {
    kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
    placements: Vec<KittyPlacement>,
    protocol_image_id_to_internal_id: HashMap<u32, u32>,
    image_number_to_protocol_image_ids: HashMap<u32, Vec<u32>>,
    protocol_image_id_to_image_number: HashMap<u32, u32>,
    next_generated_protocol_image_id: u32,
    pending_transmit: Option<PendingKittyTransmit>,
}

fn scale_u32(total: u32, kept: usize, original: usize) -> u32 {
    if original == 0 {
        0
    } else {
        ((total as u64 * kept as u64) / original as u64) as u32
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyImageInsertion {
    pub asset_id: ImageAssetId,
    pub anchor: FlowAnchor,
    pub geometry: ImagePlacementGeometry,
    pub protocol_image_id: Option<u32>,
    pub protocol_image_number: Option<u32>,
    pub protocol_placement_id: Option<PlacementId>,
    pub placement_mode: KittyImagePlacementMode,
    pub cursor_movement_policy: KittyCursorMovementPolicy,
    pub replaced_existing_asset: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KittyApcEffect {
    Placement(KittyImageInsertion),
    AssetReplaced {
        asset_id: ImageAssetId,
        protocol_image_id: Option<u32>,
        protocol_image_number: Option<u32>,
    },
    AssetStored {
        protocol_image_id: Option<u32>,
        protocol_image_number: Option<u32>,
    },
}

#[derive(Clone, Debug)]
pub struct KittyApcOutcome {
    pub effect: Option<KittyApcEffect>,
    pub reply: Option<KittyQueryResponse>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyCursorMovementPolicy {
    AfterPlacement,
    NoMovement,
}

impl KittyImageState {
    pub fn new(kitty_asset_store: Rc<RefCell<KittyAssetStore>>) -> Self {
        Self {
            kitty_asset_store,
            placements: vec![],
            protocol_image_id_to_internal_id: HashMap::new(),
            image_number_to_protocol_image_ids: HashMap::new(),
            protocol_image_id_to_image_number: HashMap::new(),
            next_generated_protocol_image_id: 0x8000_0001,
            pending_transmit: None,
        }
    }

    fn next_synthetic_protocol_image_id(&mut self) -> u32 {
        loop {
            let candidate = self.next_generated_protocol_image_id;
            self.next_generated_protocol_image_id = self
                .next_generated_protocol_image_id
                .wrapping_add(1)
                .max(0x8000_0001);
            if !self
                .protocol_image_id_to_internal_id
                .contains_key(&candidate)
            {
                return candidate;
            }
        }
    }

    fn protocol_image_id_for_create(
        &mut self,
        protocol_image_id: Option<u32>,
        image_number: Option<u32>,
    ) -> Option<u32> {
        if let Some(protocol_image_id) = protocol_image_id {
            Some(protocol_image_id)
        } else if let Some(image_number) = image_number {
            let synthetic_id = self.next_synthetic_protocol_image_id();
            self.image_number_to_protocol_image_ids
                .entry(image_number)
                .or_default()
                .push(synthetic_id);
            self.protocol_image_id_to_image_number
                .insert(synthetic_id, image_number);
            Some(synthetic_id)
        } else {
            None
        }
    }

    fn resolve_protocol_image_id(
        &self,
        protocol_image_id: Option<u32>,
        image_number: Option<u32>,
    ) -> Option<u32> {
        if let Some(protocol_image_id) = protocol_image_id {
            Some(protocol_image_id)
        } else {
            image_number.and_then(|image_number| {
                self.image_number_to_protocol_image_ids
                    .get(&image_number)
                    .and_then(|protocol_image_ids| protocol_image_ids.last().copied())
            })
        }
    }

    pub fn kitty_asset_store(&self) -> Rc<RefCell<KittyAssetStore>> {
        self.kitty_asset_store.clone()
    }

    pub fn image_dimensions(&self, image_id: u32) -> Option<(u32, u32)> {
        self.kitty_asset_store.borrow().image_dimensions(image_id)
    }

    pub fn placement(
        &self,
        image_id: u32,
        placement_id: Option<PlacementId>,
    ) -> Option<&KittyPlacement> {
        self.placements.iter().find(|placement| {
            placement.image_id == image_id && placement.placement_id == placement_id
        })
    }

    fn finalize_pending_transmit(
        &mut self,
        anchor: FlowAnchor,
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> KittyApcOutcome {
        let Some(pending) = self.pending_transmit.take() else {
            return KittyApcOutcome {
                effect: None,
                reply: None,
            };
        };
        let protocol_image_id = pending.protocol_image_id;
        let image_number = pending.image_number;
        let mut placement = pending.placement.clone();
        let image_id = pending.image_id;
        let reply_context = pending.reply_context.clone();
        let replaced_existing_asset = self
            .kitty_asset_store
            .borrow()
            .image_data(image_id)
            .is_some();
        let image_data = match pending.into_image_data() {
            Ok(image_data) => image_data,
            Err(message) => {
                return KittyApcOutcome {
                    effect: None,
                    reply: build_non_query_reply(
                        &reply_context,
                        protocol_image_id,
                        image_number,
                        Some(message),
                    ),
                };
            },
        };
        let image_dimensions = kitty_image_dimensions(&image_data);
        if let Some(placement) = placement.as_mut() {
            placement.anchor = anchor.clone();
        }
        self.kitty_asset_store
            .borrow_mut()
            .insert_asset(image_id, image_data);
        let asset_id = ImageAssetId(image_id as u64);
        if replaced_existing_asset {
            self.placements.retain(|p| p.image_id != image_id);
        }
        let Some(placement) = placement else {
            let effect = if replaced_existing_asset {
                KittyApcEffect::AssetReplaced {
                    asset_id,
                    protocol_image_id,
                    protocol_image_number: image_number,
                }
            } else {
                KittyApcEffect::AssetStored {
                    protocol_image_id,
                    protocol_image_number: image_number,
                }
            };
            return KittyApcOutcome {
                effect: Some(effect),
                reply: build_non_query_reply(&reply_context, protocol_image_id, image_number, None),
            };
        };
        self.placements.retain(|p| {
            if let Some(new_placement_id) = placement.placement_id {
                !(p.image_id == placement.image_id && p.placement_id == Some(new_placement_id))
            } else {
                true
            }
        });
        let protocol_placement_id = placement.placement_id;
        let placement_mode = placement.placement_mode;
        let cursor_movement_policy = placement.cursor_movement_policy;
        let placement_anchor = placement.anchor.clone();
        let geometry = placement.geometry_for_image(
            image_dimensions,
            cursor_x,
            scrollback_row,
            character_cell_size,
        );
        self.placements.push(placement);
        KittyApcOutcome {
            effect: Some(KittyApcEffect::Placement(KittyImageInsertion {
                asset_id,
                anchor: placement_anchor,
                geometry,
                protocol_image_id,
                protocol_image_number: image_number,
                protocol_placement_id,
                placement_mode,
                cursor_movement_policy,
                replaced_existing_asset,
            })),
            reply: build_non_query_reply(&reply_context, protocol_image_id, image_number, None),
        }
    }

    pub fn handle_apc(
        &mut self,
        apc_bytes: &[u8],
        anchor: FlowAnchor,
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> KittyApcOutcome {
        let reply_context = parse_non_query_reply_context(apc_bytes);
        let Some(command) = ParsedKittyCommand::parse(apc_bytes) else {
            return KittyApcOutcome {
                effect: None,
                reply: reply_context.and_then(|reply_context| {
                    build_non_query_reply(
                        &reply_context,
                        reply_context.parsed_image_id,
                        reply_context.image_number,
                        Some(non_query_failure_message(reply_context.kind).to_string()),
                    )
                }),
            };
        };
        match command {
            ParsedKittyCommand::ImmediateTransmit {
                protocol_image_id,
                image_number,
                image_format,
                compression,
                width,
                height,
                mut placement,
                more,
                payload,
            } => {
                let resolved_protocol_image_id =
                    self.protocol_image_id_for_create(protocol_image_id, image_number);
                let image_id = if let Some(protocol_image_id) = resolved_protocol_image_id {
                    if let Some(existing_image_id) = self
                        .protocol_image_id_to_internal_id
                        .get(&protocol_image_id)
                    {
                        *existing_image_id
                    } else {
                        let image_id = self.kitty_asset_store.borrow_mut().next_asset_id();
                        self.protocol_image_id_to_internal_id
                            .insert(protocol_image_id, image_id);
                        image_id
                    }
                } else {
                    self.kitty_asset_store.borrow_mut().next_asset_id()
                };
                if let Some(placement) = placement.as_mut() {
                    placement.image_id = image_id;
                }
                let Some(reply_context) = reply_context else {
                    return KittyApcOutcome {
                        effect: None,
                        reply: None,
                    };
                };
                self.pending_transmit = Some(PendingKittyTransmit {
                    protocol_image_id: resolved_protocol_image_id,
                    image_number,
                    image_id,
                    image_format,
                    compression,
                    width,
                    height,
                    placement,
                    reply_context,
                    payload,
                });
                if more {
                    KittyApcOutcome {
                        effect: None,
                        reply: None,
                    }
                } else {
                    self.finalize_pending_transmit(
                        anchor,
                        cursor_x,
                        scrollback_row,
                        character_cell_size,
                    )
                }
            },
            ParsedKittyCommand::DisplayPlacement {
                protocol_image_id,
                image_number,
                mut placement,
            } => {
                let Some(reply_context) = reply_context else {
                    return KittyApcOutcome {
                        effect: None,
                        reply: None,
                    };
                };
                let resolved_protocol_image_id =
                    match self.resolve_protocol_image_id(protocol_image_id, image_number) {
                        Some(resolved_protocol_image_id) => resolved_protocol_image_id,
                        None => {
                            return KittyApcOutcome {
                                effect: None,
                                reply: build_non_query_reply(
                                    &reply_context,
                                    protocol_image_id,
                                    image_number,
                                    Some(non_query_failure_message(reply_context.kind).to_string()),
                                ),
                            };
                        },
                    };
                let image_id = match self
                    .protocol_image_id_to_internal_id
                    .get(&resolved_protocol_image_id)
                {
                    Some(image_id) => *image_id,
                    None => {
                        return KittyApcOutcome {
                            effect: None,
                            reply: build_non_query_reply(
                                &reply_context,
                                Some(resolved_protocol_image_id),
                                image_number,
                                Some(non_query_failure_message(reply_context.kind).to_string()),
                            ),
                        };
                    },
                };
                let image_dimensions =
                    match self.kitty_asset_store.borrow().image_dimensions(image_id) {
                        Some(image_dimensions) => image_dimensions,
                        None => {
                            return KittyApcOutcome {
                                effect: None,
                                reply: build_non_query_reply(
                                    &reply_context,
                                    Some(resolved_protocol_image_id),
                                    image_number,
                                    Some(non_query_failure_message(reply_context.kind).to_string()),
                                ),
                            };
                        },
                    };
                placement.image_id = image_id;
                placement.anchor = anchor;
                self.placements.retain(|p| {
                    if let Some(new_placement_id) = placement.placement_id {
                        !(p.image_id == placement.image_id
                            && p.placement_id == Some(new_placement_id))
                    } else {
                        true
                    }
                });
                let geometry = placement.geometry_for_image(
                    image_dimensions,
                    cursor_x,
                    scrollback_row,
                    character_cell_size,
                );
                let protocol_placement_id = placement.placement_id;
                let placement_mode = placement.placement_mode;
                let cursor_movement_policy = placement.cursor_movement_policy;
                let placement_anchor = placement.anchor.clone();
                self.placements.push(placement);
                KittyApcOutcome {
                    effect: Some(KittyApcEffect::Placement(KittyImageInsertion {
                        asset_id: ImageAssetId(image_id as u64),
                        anchor: placement_anchor,
                        geometry,
                        protocol_image_id: Some(resolved_protocol_image_id),
                        protocol_image_number: image_number,
                        protocol_placement_id,
                        placement_mode,
                        cursor_movement_policy,
                        replaced_existing_asset: false,
                    })),
                    reply: build_non_query_reply(
                        &reply_context,
                        Some(resolved_protocol_image_id),
                        image_number,
                        None,
                    ),
                }
            },
            ParsedKittyCommand::TransmitChunk { more, payload } => {
                let Some(pending) = self.pending_transmit.as_mut() else {
                    return KittyApcOutcome {
                        effect: None,
                        reply: None,
                    };
                };
                if let Some(quiet) = parse_chunk_quiet(apc_bytes) {
                    pending.reply_context.quiet = quiet;
                }
                pending.payload.extend(payload);
                if more {
                    KittyApcOutcome {
                        effect: None,
                        reply: None,
                    }
                } else {
                    self.finalize_pending_transmit(
                        anchor,
                        cursor_x,
                        scrollback_row,
                        character_cell_size,
                    )
                }
            },
        }
    }

    pub fn visible_chunks<F>(
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
        let Some(cell_size) = character_cell_size else {
            return vec![];
        };
        let mut chunks = vec![];
        let kitty_asset_store = self.kitty_asset_store.borrow();
        for placement in &self.placements {
            let Some((image_width, image_height)) =
                kitty_asset_store.image_dimensions(placement.image_id)
            else {
                continue;
            };
            let mut source_x = placement.source_x.unwrap_or(0);
            let mut source_y = placement.source_y.unwrap_or(0);
            let mut source_width = placement
                .source_width
                .unwrap_or_else(|| image_width.saturating_sub(source_x));
            let mut source_height = placement
                .source_height
                .unwrap_or_else(|| image_height.saturating_sub(source_y));
            let mut columns = placement.columns.unwrap_or_else(|| {
                ((source_width as usize + cell_size.width.saturating_sub(1)) / cell_size.width)
                    .max(1) as u32
            }) as usize;
            let mut rows = placement.rows.unwrap_or_else(|| {
                ((source_height as usize + cell_size.height.saturating_sub(1)) / cell_size.height)
                    .max(1) as u32
            }) as usize;

            if columns == 0 || rows == 0 {
                continue;
            }

            let Some((logical_row, column)) = resolve_anchor(&placement.anchor) else {
                continue;
            };
            let Some(projection) = project_placement_to_viewport(
                logical_row,
                column,
                &PlacementOccupancy { columns, rows },
                content_x,
                content_y,
                scrollback_size_in_lines,
                viewport_width,
                viewport_height,
            ) else {
                continue;
            };
            let clipped_by_projection = projection.clipped_left_cols > 0
                || projection.clipped_top_rows > 0
                || projection.columns != columns
                || projection.rows != rows;

            if projection.clipped_left_cols > 0 {
                source_x =
                    source_x + scale_u32(source_width, projection.clipped_left_cols, columns);
            }
            source_width = scale_u32(source_width, projection.columns, columns);
            columns = projection.columns;

            if projection.clipped_top_rows > 0 {
                source_y = source_y + scale_u32(source_height, projection.clipped_top_rows, rows);
            }
            source_height = scale_u32(source_height, projection.rows, rows);
            rows = projection.rows;

            let cell_x = projection.cell_x;
            let cell_y = projection.cell_y;
            chunks.push(KittyImageChunk {
                image_id: placement.image_id,
                placement_id: placement.placement_id,
                placement_mode: placement.placement_mode,
                cell_x,
                cell_y,
                columns,
                rows,
                columns_specified: placement.columns_specified || clipped_by_projection,
                rows_specified: placement.rows_specified || clipped_by_projection,
                source_x,
                source_y,
                source_width,
                source_height,
                z_index: placement.z_index.unwrap_or(0),
                x_offset: placement.x_offset.unwrap_or(0),
                y_offset: placement.y_offset.unwrap_or(0),
            });
        }
        chunks
    }

    pub fn clear(&mut self) {
        self.placements.clear();
        self.protocol_image_id_to_internal_id.clear();
        self.image_number_to_protocol_image_ids.clear();
        self.protocol_image_id_to_image_number.clear();
        self.pending_transmit = None;
    }

    pub fn abort_pending_transmit(&mut self) {
        self.pending_transmit = None;
    }

    fn remove_protocol_image_references(&mut self, protocol_image_id: u32) {
        self.protocol_image_id_to_internal_id
            .remove(&protocol_image_id);
        if let Some(image_number) = self
            .protocol_image_id_to_image_number
            .remove(&protocol_image_id)
        {
            if let Some(protocol_image_ids) = self
                .image_number_to_protocol_image_ids
                .get_mut(&image_number)
            {
                protocol_image_ids.retain(|existing_protocol_image_id| {
                    *existing_protocol_image_id != protocol_image_id
                });
                if protocol_image_ids.is_empty() {
                    self.image_number_to_protocol_image_ids
                        .remove(&image_number);
                }
            }
        }
    }

    pub fn delete_protocol_placement(
        &mut self,
        protocol_image_id: u32,
        placement_id: Option<PlacementId>,
        free_image_data: bool,
    ) {
        let Some(internal_image_id) = self
            .protocol_image_id_to_internal_id
            .get(&protocol_image_id)
            .copied()
        else {
            return;
        };
        self.placements.retain(|placement| {
            if placement.image_id != internal_image_id {
                return true;
            }
            match placement_id {
                Some(placement_id) => placement.placement_id != Some(placement_id),
                None => false,
            }
        });
        let has_remaining_references = self
            .placements
            .iter()
            .any(|placement| placement.image_id == internal_image_id);
        if free_image_data && !has_remaining_references {
            self.kitty_asset_store
                .borrow_mut()
                .remove_asset(internal_image_id);
            self.remove_protocol_image_references(protocol_image_id);
        }
    }

    pub fn protocol_image_id_for_image_number(&self, image_number: u32) -> Option<u32> {
        self.image_number_to_protocol_image_ids
            .get(&image_number)
            .and_then(|protocol_image_ids| protocol_image_ids.last().copied())
    }

    pub fn serialize_chunks_with_asset_store(
        chunks: &[KittyImageChunk],
        kitty_asset_store: &KittyAssetStore,
    ) -> String {
        if chunks.is_empty() {
            return String::new();
        }
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str("\u{1b}[s");

        let mut transmitted_image_ids = std::collections::HashSet::new();
        for chunk in chunks {
            if transmitted_image_ids.insert(chunk.image_id) {
                let Some(image_data) = kitty_asset_store.image_data(chunk.image_id) else {
                    continue;
                };
                for transmit_command in serialize_transmit(chunk.image_id, &image_data) {
                    raw_vte_output.push_str("\u{1b}_G");
                    raw_vte_output.push_str(&transmit_command);
                    raw_vte_output.push_str("\u{1b}\\");
                }
            }
        }

        for (placement_index, chunk) in chunks.iter().enumerate() {
            let placement_id = chunk
                .placement_id
                .unwrap_or(PlacementId::Synthetic(placement_index as u32 + 1))
                .wire_value();
            raw_vte_output.push_str(&Self::serialize_explicit_placement(chunk, placement_id));
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }

    pub fn serialize_placeholder_renders_with_asset_store(
        renders: &[KittyPlaceholderRender],
        kitty_asset_store: &KittyAssetStore,
    ) -> String {
        if renders.is_empty() {
            return String::new();
        }
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str("\u{1b}[s");

        let mut transmitted_image_ids = std::collections::HashSet::new();
        for render in renders {
            if transmitted_image_ids.insert(render.image_id) {
                let Some(image_data) = kitty_asset_store.image_data(render.image_id) else {
                    continue;
                };
                for transmit_command in serialize_transmit(render.image_id, &image_data) {
                    raw_vte_output.push_str("\u{1b}_G");
                    raw_vte_output.push_str(&transmit_command);
                    raw_vte_output.push_str("\u{1b}\\");
                }
            }
        }

        for (placement_index, render) in renders.iter().enumerate() {
            let placement_id = render
                .placement_id
                .unwrap_or(PlacementId::Synthetic(placement_index as u32 + 1))
                .wire_value();
            raw_vte_output.push_str(&Self::serialize_placeholder_render(render, placement_id));
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }

    pub fn serialize_full_scene_with_asset_store(
        chunks: &[KittyImageChunk],
        renders: &[KittyPlaceholderRender],
        kitty_asset_store: &KittyAssetStore,
    ) -> String {
        if chunks.is_empty() && renders.is_empty() {
            return String::new();
        }
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str("\u{1b}[s");

        let mut transmitted_image_ids = std::collections::HashSet::new();
        for image_id in chunks
            .iter()
            .map(|chunk| chunk.image_id)
            .chain(renders.iter().map(|render| render.image_id))
        {
            if transmitted_image_ids.insert(image_id) {
                let Some(image_data) = kitty_asset_store.image_data(image_id) else {
                    continue;
                };
                for transmit_command in serialize_transmit(image_id, &image_data) {
                    raw_vte_output.push_str("\u{1b}_G");
                    raw_vte_output.push_str(&transmit_command);
                    raw_vte_output.push_str("\u{1b}\\");
                }
            }
        }

        let mut next_synthesized_placement_id = 1u32;
        for chunk in chunks {
            let placement_id = chunk.placement_id.unwrap_or_else(|| {
                let placement_id = next_synthesized_placement_id;
                next_synthesized_placement_id += 1;
                PlacementId::Synthetic(placement_id)
            });
            raw_vte_output.push_str(&Self::serialize_explicit_placement(
                chunk,
                placement_id.wire_value(),
            ));
        }
        for render in renders {
            let placement_id = render.placement_id.unwrap_or_else(|| {
                let placement_id = next_synthesized_placement_id;
                next_synthesized_placement_id += 1;
                PlacementId::Synthetic(placement_id)
            });
            raw_vte_output.push_str(&Self::serialize_placeholder_render(
                render,
                placement_id.wire_value(),
            ));
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }

    pub fn serialize_image_data(image_id: u32, image_data: &KittyImageData) -> String {
        let mut raw_vte_output = String::new();
        for transmit_command in serialize_transmit(image_id, image_data) {
            raw_vte_output.push_str("\u{1b}_G");
            raw_vte_output.push_str(&transmit_command);
            raw_vte_output.push_str("\u{1b}\\");
        }
        raw_vte_output
    }

    pub fn serialize_delete_placement(image_id: u32, placement_id: u32) -> String {
        format!("\u{1b}_Ga=d,d=i,i={},p={}\u{1b}\\", image_id, placement_id)
    }

    pub fn serialize_explicit_placement(chunk: &KittyImageChunk, placement_id: u32) -> String {
        let cursor_x = chunk.cell_x + 1;
        let cursor_y = chunk.cell_y + 1;
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str(&format!("\u{1b}[{};{}H", cursor_y, cursor_x));
        raw_vte_output.push_str("\u{1b}_G");
        raw_vte_output.push_str(&serialize_display(chunk, placement_id));
        raw_vte_output.push_str("\u{1b}\\");
        raw_vte_output
    }

    pub fn serialize_placeholder_render(
        render: &KittyPlaceholderRender,
        placement_id: u32,
    ) -> String {
        serialize_placeholder_render(render, placement_id)
    }
}

#[derive(Clone, Debug)]
pub enum KittyQueryResponse {
    Ok {
        image_id: Option<u32>,
        placement_id: Option<u32>,
        image_number: Option<u32>,
    },
    Error {
        image_id: Option<u32>,
        placement_id: Option<u32>,
        image_number: Option<u32>,
        message: String,
    },
}

impl KittyQueryResponse {
    pub fn to_apc_response(&self) -> String {
        match self {
            KittyQueryResponse::Ok {
                image_id,
                placement_id,
                image_number,
            } => {
                let mut control_data = String::new();
                if let Some(image_id) = image_id {
                    control_data.push_str(&format!("i={}", image_id));
                    if let Some(placement_id) = placement_id {
                        control_data.push_str(&format!(",p={}", placement_id));
                    }
                    if let Some(image_number) = image_number {
                        control_data.push_str(&format!(",I={}", image_number));
                    }
                    control_data.push(';');
                } else if let Some(image_number) = image_number {
                    control_data.push_str(&format!("I={};", image_number));
                } else {
                    control_data.push(';');
                }
                format!("\u{1b}_G{}OK\u{1b}\\", control_data)
            },
            KittyQueryResponse::Error {
                image_id,
                placement_id,
                image_number,
                message,
            } => {
                let mut control_data = String::new();
                if let Some(image_id) = image_id {
                    control_data.push_str(&format!("i={}", image_id));
                    if let Some(placement_id) = placement_id {
                        control_data.push_str(&format!(",p={}", placement_id));
                    }
                    if let Some(image_number) = image_number {
                        control_data.push_str(&format!(",I={}", image_number));
                    }
                    control_data.push(';');
                } else if let Some(image_number) = image_number {
                    control_data.push_str(&format!("I={};", image_number));
                } else {
                    control_data.push(';');
                }
                format!("\u{1b}_G{}{}\u{1b}\\", control_data, message)
            },
        }
    }
}

#[derive(Clone, Debug)]
enum ParsedKittyCommand {
    ImmediateTransmit {
        protocol_image_id: Option<u32>,
        image_number: Option<u32>,
        image_format: KittyImageFormat,
        compression: Option<KittyTransportCompression>,
        width: u32,
        height: u32,
        placement: Option<KittyPlacement>,
        more: bool,
        payload: Vec<u8>,
    },
    DisplayPlacement {
        protocol_image_id: Option<u32>,
        image_number: Option<u32>,
        placement: KittyPlacement,
    },
    TransmitChunk {
        more: bool,
        payload: Vec<u8>,
    },
}

impl KittyPlacement {
    fn geometry_for_image(
        &self,
        image_dimensions: (u32, u32),
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> ImagePlacementGeometry {
        let (image_width, image_height) = image_dimensions;
        let source_x = self.source_x.unwrap_or(0);
        let source_y = self.source_y.unwrap_or(0);
        let source_width = self
            .source_width
            .unwrap_or_else(|| image_width.saturating_sub(source_x));
        let source_height = self
            .source_height
            .unwrap_or_else(|| image_height.saturating_sub(source_y));
        let (columns, rows) = if let Some(cell_size) = character_cell_size {
            let default_columns = || {
                ((source_width as usize + cell_size.width.saturating_sub(1)) / cell_size.width)
                    .max(1) as u32
            };
            let default_rows = || {
                ((source_height as usize + cell_size.height.saturating_sub(1)) / cell_size.height)
                    .max(1) as u32
            };
            match (self.columns, self.rows) {
                (Some(columns), Some(rows)) => (columns as usize, rows as usize),
                (Some(columns), None) => {
                    let scaled_width_pixels = (columns as u64) * (cell_size.width as u64);
                    let scaled_height_pixels = if source_width > 0 {
                        ((scaled_width_pixels * (source_height as u64)) + (source_width as u64) - 1)
                            / (source_width as u64)
                    } else {
                        0
                    };
                    let rows = ((scaled_height_pixels + (cell_size.height as u64) - 1)
                        / (cell_size.height as u64))
                        .max(1) as usize;
                    (columns as usize, rows)
                },
                (None, Some(rows)) => {
                    let scaled_height_pixels = (rows as u64) * (cell_size.height as u64);
                    let scaled_width_pixels = if source_height > 0 {
                        ((scaled_height_pixels * (source_width as u64)) + (source_height as u64)
                            - 1)
                            / (source_height as u64)
                    } else {
                        0
                    };
                    let columns = ((scaled_width_pixels + (cell_size.width as u64) - 1)
                        / (cell_size.width as u64))
                        .max(1) as usize;
                    (columns, rows as usize)
                },
                (None, None) => (default_columns() as usize, default_rows() as usize),
            }
        } else {
            (
                self.columns.unwrap_or(0) as usize,
                self.rows.unwrap_or(0) as usize,
            )
        };
        ImagePlacementGeometry {
            anchor_x: cursor_x,
            logical_row: scrollback_row,
            columns,
            rows,
            columns_specified: self.columns_specified,
            rows_specified: self.rows_specified,
            source_x,
            source_y,
            source_width,
            source_height,
            z_index: self.z_index.unwrap_or(0),
            x_offset: self.x_offset.unwrap_or(0),
            y_offset: self.y_offset.unwrap_or(0),
        }
    }
}

impl PendingKittyTransmit {
    fn into_image_data(self) -> Result<KittyImageData, String> {
        let payload = match self.compression {
            Some(KittyTransportCompression::Zlib) => decompress_to_vec_zlib(&self.payload)
                .map_err(|_| "EINVAL:Invalid image payload encoding".to_string())?,
            None => self.payload,
        };
        Ok(match self.image_format {
            KittyImageFormat::Png => {
                let (width, height) = parse_png_dimensions(&payload).ok_or_else(|| {
                    "EINVAL:Invalid image payload for requested format".to_string()
                })?;
                KittyImageData::Png {
                    data: payload,
                    width,
                    height,
                }
            },
            KittyImageFormat::Rgb => {
                validate_raw_payload_size(payload.len(), self.width, self.height, 3)?;
                KittyImageData::Rgb {
                    data: payload,
                    width: self.width,
                    height: self.height,
                }
            },
            KittyImageFormat::Rgba => {
                validate_raw_payload_size(payload.len(), self.width, self.height, 4)?;
                KittyImageData::Rgba {
                    data: payload,
                    width: self.width,
                    height: self.height,
                }
            },
        })
    }
}

fn validate_raw_payload_size(
    payload_len: usize,
    width: u32,
    height: u32,
    bytes_per_pixel: usize,
) -> Result<(), String> {
    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixel_count| pixel_count.checked_mul(bytes_per_pixel))
        .ok_or_else(|| "EINVAL:Invalid image payload for requested format".to_string())?;
    if payload_len < expected_len {
        Err(format!(
            "ENODATA:Insufficient image data: {payload_len} < {expected_len}"
        ))
    } else if payload_len > expected_len {
        Err("EINVAL:Invalid image payload for requested format".to_string())
    } else {
        Ok(())
    }
}

fn kitty_image_dimensions(image_data: &KittyImageData) -> (u32, u32) {
    match image_data {
        KittyImageData::Png { width, height, .. }
        | KittyImageData::Rgb { width, height, .. }
        | KittyImageData::Rgba { width, height, .. } => (*width, *height),
    }
}

fn kitty_delete_header(apc_bytes: &[u8]) -> Option<HashMap<&str, &str>> {
    let rest = apc_bytes.strip_prefix(b"G")?;
    let mut parts = rest.splitn(2, |b| *b == b';');
    let header = std::str::from_utf8(parts.next()?).ok()?;
    let mut kv = HashMap::new();
    for part in header.split(',') {
        if part.is_empty() {
            continue;
        }
        let mut split = part.splitn(2, '=');
        let key = split.next()?;
        let value = split.next().unwrap_or("");
        kv.insert(key, value);
    }
    Some(kv)
}

fn decode_kitty_payload(payload_b64: &[u8], compression: Option<&str>) -> Option<Vec<u8>> {
    let payload = decode_kitty_transport_payload(payload_b64)?;
    match compression {
        Some("z") => decompress_to_vec_zlib(&payload).ok(),
        Some(_) => None,
        None => Some(payload),
    }
}

fn decode_kitty_transport_payload(payload_b64: &[u8]) -> Option<Vec<u8>> {
    base64::decode(payload_b64).ok()
}

fn parse_kitty_transport_compression(
    compression: Option<&str>,
) -> Option<Option<KittyTransportCompression>> {
    match compression {
        Some("z") => Some(Some(KittyTransportCompression::Zlib)),
        Some(_) => None,
        None => Some(None),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyDeleteMode {
    PlacementsOnly,
    PlacementsAndBackingData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyGeometrySelector {
    Cursor,
    Cell { x: u32, y: u32, z: Option<i32> },
    Column { x: u32 },
    Row { y: u32 },
    Z { z: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyDeleteSelector {
    AllVisible,
    ImageId {
        image_id: u32,
        placement_id: Option<PlacementId>,
    },
    ImageNumber {
        image_number: u32,
        placement_id: Option<PlacementId>,
    },
    Geometry(KittyGeometrySelector),
    Range {
        first_image_id: u32,
        last_image_id: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KittyDeleteRequest {
    pub selector: KittyDeleteSelector,
    pub mode: KittyDeleteMode,
}

impl KittyDeleteRequest {
    pub fn free_image_data(&self) -> bool {
        self.mode == KittyDeleteMode::PlacementsAndBackingData
    }
}

pub fn kitty_delete_request(apc_bytes: &[u8]) -> Option<KittyDeleteRequest> {
    let kv = kitty_delete_header(apc_bytes)?;
    if kv.get("a").copied() != Some("d") {
        return None;
    }
    let delete_selector = kv.get("d").copied().unwrap_or("a");
    let mode = match delete_selector {
        "A" | "I" | "N" | "C" | "P" | "Q" | "R" | "X" | "Y" | "Z" => {
            KittyDeleteMode::PlacementsAndBackingData
        },
        "a" | "i" | "n" | "c" | "p" | "q" | "r" | "x" | "y" | "z" => {
            KittyDeleteMode::PlacementsOnly
        },
        _ => return None,
    };
    let placement_id = kv
        .get("p")
        .and_then(|p| p.parse::<u32>().ok())
        .map(PlacementId::Protocol);
    let selector = match delete_selector {
        "a" | "A" => KittyDeleteSelector::AllVisible,
        "i" | "I" => KittyDeleteSelector::ImageId {
            image_id: kv.get("i")?.parse::<u32>().ok()?,
            placement_id,
        },
        "n" | "N" => KittyDeleteSelector::ImageNumber {
            image_number: kv.get("I")?.parse::<u32>().ok()?,
            placement_id,
        },
        "c" | "C" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Cursor),
        "p" | "P" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
            x: kv.get("x")?.parse::<u32>().ok()?,
            y: kv.get("y")?.parse::<u32>().ok()?,
            z: None,
        }),
        "q" | "Q" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
            x: kv.get("x")?.parse::<u32>().ok()?,
            y: kv.get("y")?.parse::<u32>().ok()?,
            z: Some(kv.get("z")?.parse::<i32>().ok()?),
        }),
        "x" | "X" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Column {
            x: kv.get("x")?.parse::<u32>().ok()?,
        }),
        "y" | "Y" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Row {
            y: kv.get("y")?.parse::<u32>().ok()?,
        }),
        "z" | "Z" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Z {
            z: kv.get("z")?.parse::<i32>().ok()?,
        }),
        "r" | "R" => KittyDeleteSelector::Range {
            first_image_id: kv.get("x")?.parse::<u32>().ok()?,
            last_image_id: kv.get("y")?.parse::<u32>().ok()?,
        },
        _ => return None,
    };
    Some(KittyDeleteRequest { selector, mode })
}

pub fn kitty_delete_all_visible(apc_bytes: &[u8]) -> bool {
    matches!(
        kitty_delete_request(apc_bytes),
        Some(KittyDeleteRequest {
            selector: KittyDeleteSelector::AllVisible,
            ..
        })
    )
}

pub fn kitty_delete_by_image_id(apc_bytes: &[u8]) -> Option<(u32, Option<PlacementId>, bool)> {
    match kitty_delete_request(apc_bytes)? {
        KittyDeleteRequest {
            selector:
                KittyDeleteSelector::ImageId {
                    image_id,
                    placement_id,
                },
            mode,
        } => Some((
            image_id,
            placement_id,
            mode == KittyDeleteMode::PlacementsAndBackingData,
        )),
        _ => None,
    }
}

pub fn kitty_delete_by_image_number(
    apc_bytes: &[u8],
) -> Option<(u32, Option<PlacementId>, bool)> {
    match kitty_delete_request(apc_bytes)? {
        KittyDeleteRequest {
            selector:
                KittyDeleteSelector::ImageNumber {
                    image_number,
                    placement_id,
                },
            mode,
        } => Some((
            image_number,
            placement_id,
            mode == KittyDeleteMode::PlacementsAndBackingData,
        )),
        _ => None,
    }
}

pub fn kitty_query_response(apc_bytes: &[u8]) -> Option<KittyQueryResponse> {
    let rest = apc_bytes.strip_prefix(b"G")?;
    let mut parts = rest.splitn(2, |b| *b == b';');
    let header = std::str::from_utf8(parts.next()?).ok()?;
    let payload_b64 = parts.next().unwrap_or_default();

    let mut kv = HashMap::new();
    for part in header.split(',') {
        if part.is_empty() {
            continue;
        }
        let mut split = part.splitn(2, '=');
        let key = split.next()?;
        let value = split.next().unwrap_or("");
        kv.insert(key, value);
    }

    if kv.get("a").copied() != Some("q") {
        return None;
    }

    let quiet = kv.get("q").and_then(|q| q.parse::<u8>().ok()).unwrap_or(0);
    let image_id = kv.get("i").and_then(|i| i.parse::<u32>().ok());
    let placement_id = kv.get("p").and_then(|p| p.parse::<u32>().ok());
    let image_number = kv.get("I").and_then(|i| i.parse::<u32>().ok());
    let transport = kv.get("t").copied().unwrap_or("d");

    let reply = if image_id.is_some() && image_number.is_some() {
        KittyQueryResponse::Error {
            image_id,
            placement_id,
            image_number,
            message: "EINVAL:Must not specify both i and I".to_string(),
        }
    } else if transport != "d" {
        KittyQueryResponse::Error {
            image_id,
            placement_id,
            image_number,
            message: "EINVAL:Unsupported transmission medium".to_string(),
        }
    } else {
        let payload = match decode_kitty_payload(payload_b64, kv.get("o").copied()) {
            Some(payload) => payload,
            None => {
                let response = KittyQueryResponse::Error {
                    image_id,
                    placement_id,
                    image_number,
                    message: "EINVAL:Invalid image payload encoding".to_string(),
                };
                return if quiet == 2 { None } else { Some(response) };
            },
        };
        let format = kv.get("f").copied().unwrap_or("32");
        let valid = match format {
            "24" => {
                let width = match kv.get("s").and_then(|v| v.parse::<usize>().ok()) {
                    Some(width) => width,
                    None => {
                        let response = KittyQueryResponse::Error {
                            image_id,
                            placement_id,
                            image_number,
                            message: "EINVAL:Missing or invalid s for RGB payload".to_string(),
                        };
                        return if quiet == 2 { None } else { Some(response) };
                    },
                };
                let height = match kv.get("v").and_then(|v| v.parse::<usize>().ok()) {
                    Some(height) => height,
                    None => {
                        let response = KittyQueryResponse::Error {
                            image_id,
                            placement_id,
                            image_number,
                            message: "EINVAL:Missing or invalid v for RGB payload".to_string(),
                        };
                        return if quiet == 2 { None } else { Some(response) };
                    },
                };
                payload.len() == width * height * 3
            },
            "32" => {
                let width = match kv.get("s").and_then(|v| v.parse::<usize>().ok()) {
                    Some(width) => width,
                    None => {
                        let response = KittyQueryResponse::Error {
                            image_id,
                            placement_id,
                            image_number,
                            message: "EINVAL:Missing or invalid s for RGBA payload".to_string(),
                        };
                        return if quiet == 2 { None } else { Some(response) };
                    },
                };
                let height = match kv.get("v").and_then(|v| v.parse::<usize>().ok()) {
                    Some(height) => height,
                    None => {
                        let response = KittyQueryResponse::Error {
                            image_id,
                            placement_id,
                            image_number,
                            message: "EINVAL:Missing or invalid v for RGBA payload".to_string(),
                        };
                        return if quiet == 2 { None } else { Some(response) };
                    },
                };
                payload.len() == width * height * 4
            },
            "100" => parse_png_dimensions(&payload).is_some(),
            _ => false,
        };
        if valid {
            KittyQueryResponse::Ok {
                image_id,
                placement_id,
                image_number,
            }
        } else {
            KittyQueryResponse::Error {
                image_id,
                placement_id,
                image_number,
                message: "EINVAL:Invalid image payload for requested format".to_string(),
            }
        }
    };

    match (&reply, quiet) {
        (KittyQueryResponse::Ok { .. }, 1 | 2) => None,
        (KittyQueryResponse::Error { .. }, 2) => None,
        _ => Some(reply),
    }
}

pub fn kitty_non_query_response(
    apc_bytes: &[u8],
    command_succeeded: bool,
    resolved_image_id: Option<u32>,
    resolved_image_number: Option<u32>,
) -> Option<KittyQueryResponse> {
    let reply_context = parse_non_query_reply_context(apc_bytes)?;
    let error =
        (!command_succeeded).then(|| non_query_failure_message(reply_context.kind).to_string());
    build_non_query_reply(
        &reply_context,
        resolved_image_id.or(reply_context.parsed_image_id),
        resolved_image_number.or(reply_context.image_number),
        error,
    )
}

fn parse_non_query_reply_context(apc_bytes: &[u8]) -> Option<PendingKittyReplyContext> {
    let rest = apc_bytes.strip_prefix(b"G")?;
    let mut parts = rest.splitn(2, |b| *b == b';');
    let header = std::str::from_utf8(parts.next()?).ok()?;

    let mut kv = HashMap::new();
    for part in header.split(',') {
        if part.is_empty() {
            continue;
        }
        let mut split = part.splitn(2, '=');
        let key = split.next()?;
        let value = split.next().unwrap_or("");
        kv.insert(key, value);
    }

    let kind = match kv.get("a").copied()? {
        "t" | "T" => PendingKittyReplyKind::Transmit,
        "p" => PendingKittyReplyKind::Placement,
        _ => return None,
    };
    Some(PendingKittyReplyContext {
        kind,
        quiet: kv.get("q").and_then(|q| q.parse::<u8>().ok()).unwrap_or(0),
        parsed_image_id: kv.get("i").and_then(|i| i.parse::<u32>().ok()),
        placement_id: kv.get("p").and_then(|p| p.parse::<u32>().ok()),
        image_number: kv.get("I").and_then(|i| i.parse::<u32>().ok()),
    })
}

fn parse_chunk_quiet(apc_bytes: &[u8]) -> Option<u8> {
    let rest = apc_bytes.strip_prefix(b"G")?;
    let mut parts = rest.splitn(2, |b| *b == b';');
    let header = std::str::from_utf8(parts.next()?).ok()?;
    for part in header.split(',') {
        let mut split = part.splitn(2, '=');
        let key = split.next()?;
        let value = split.next().unwrap_or("");
        if key == "q" {
            return value.parse::<u8>().ok();
        }
    }
    None
}

fn non_query_failure_message(kind: PendingKittyReplyKind) -> &'static str {
    match kind {
        PendingKittyReplyKind::Transmit => "EINVAL:Invalid or unsupported kitty command",
        PendingKittyReplyKind::Placement => "ENOENT:Image or placement not found",
    }
}

fn build_non_query_reply(
    reply_context: &PendingKittyReplyContext,
    image_id: Option<u32>,
    image_number: Option<u32>,
    error_message: Option<String>,
) -> Option<KittyQueryResponse> {
    let reply = if reply_context.parsed_image_id.is_some() && reply_context.image_number.is_some() {
        KittyQueryResponse::Error {
            image_id,
            placement_id: reply_context.placement_id,
            image_number,
            message: "EINVAL:Must not specify both i and I".to_string(),
        }
    } else if let Some(error_message) = error_message {
        KittyQueryResponse::Error {
            image_id,
            placement_id: reply_context.placement_id,
            image_number,
            message: error_message,
        }
    } else {
        KittyQueryResponse::Ok {
            image_id,
            placement_id: reply_context.placement_id,
            image_number,
        }
    };

    match (&reply, reply_context.quiet) {
        (KittyQueryResponse::Ok { .. }, 1 | 2) => None,
        (KittyQueryResponse::Error { .. }, 2) => None,
        _ => Some(reply),
    }
}

impl ParsedKittyCommand {
    fn parse(apc_bytes: &[u8]) -> Option<Self> {
        let rest = apc_bytes.strip_prefix(b"G")?;
        let mut parts = rest.splitn(2, |b| *b == b';');
        let header = std::str::from_utf8(parts.next()?).ok()?;
        let payload = parts.next().unwrap_or_default();

        let mut kv = HashMap::new();
        for part in header.split(',') {
            if part.is_empty() {
                continue;
            }
            let mut split = part.splitn(2, '=');
            let key = split.next()?;
            let value = split.next().unwrap_or("");
            kv.insert(key, value);
        }

        let more = kv.get("m").and_then(|m| m.parse::<u8>().ok()).unwrap_or(0) != 0;
        let payload = decode_kitty_transport_payload(payload)?;

        if let Some(action) = kv.get("a") {
            let placement = KittyPlacement {
                image_id: 0,
                placement_id: kv
                    .get("p")
                    .and_then(|p| p.parse::<u32>().ok())
                    .map(PlacementId::Protocol),
                placement_mode: if kv.get("U").copied() == Some("1") {
                    KittyImagePlacementMode::Placeholder
                } else {
                    KittyImagePlacementMode::Explicit
                },
                source_x: kv.get("x").and_then(|v| v.parse::<u32>().ok()),
                source_y: kv.get("y").and_then(|v| v.parse::<u32>().ok()),
                source_width: kv.get("w").and_then(|v| v.parse::<u32>().ok()),
                source_height: kv.get("h").and_then(|v| v.parse::<u32>().ok()),
                columns: kv.get("c").and_then(|v| v.parse::<u32>().ok()),
                rows: kv.get("r").and_then(|v| v.parse::<u32>().ok()),
                columns_specified: kv.contains_key("c"),
                rows_specified: kv.contains_key("r"),
                x_offset: kv.get("X").and_then(|v| v.parse::<u32>().ok()),
                y_offset: kv.get("Y").and_then(|v| v.parse::<u32>().ok()),
                z_index: kv.get("z").and_then(|v| v.parse::<i32>().ok()),
                cursor_movement_policy: match kv.get("C").copied() {
                    Some("1") => KittyCursorMovementPolicy::NoMovement,
                    _ => KittyCursorMovementPolicy::AfterPlacement,
                },
                ..Default::default()
            };
            match *action {
                "T" | "t" => {
                    let image_format = match kv.get("f").copied().unwrap_or("32") {
                        "100" => KittyImageFormat::Png,
                        "24" => KittyImageFormat::Rgb,
                        "32" => KittyImageFormat::Rgba,
                        _ => return None,
                    };
                    let compression = parse_kitty_transport_compression(kv.get("o").copied())?;
                    let protocol_image_id = kv.get("i").and_then(|i| i.parse::<u32>().ok());
                    let image_number = kv.get("I").and_then(|i| i.parse::<u32>().ok());
                    let width = kv.get("s").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                    let height = kv.get("v").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                    let should_create_placement = *action == "T"
                        || kv.contains_key("p")
                        || kv.contains_key("c")
                        || kv.contains_key("r")
                        || kv.contains_key("x")
                        || kv.contains_key("y")
                        || kv.contains_key("w")
                        || kv.contains_key("h")
                        || kv.contains_key("X")
                        || kv.contains_key("Y")
                        || kv.contains_key("z")
                        || kv.get("U").copied() == Some("1");
                    Some(ParsedKittyCommand::ImmediateTransmit {
                        protocol_image_id,
                        image_number,
                        image_format,
                        compression,
                        width,
                        height,
                        placement: if should_create_placement {
                            Some(placement)
                        } else {
                            None
                        },
                        more,
                        payload,
                    })
                },
                "p" => {
                    let protocol_image_id = kv.get("i").and_then(|i| i.parse::<u32>().ok());
                    let image_number = kv.get("I").and_then(|i| i.parse::<u32>().ok());
                    if protocol_image_id.is_none() && image_number.is_none() {
                        return None;
                    }
                    Some(ParsedKittyCommand::DisplayPlacement {
                        protocol_image_id,
                        image_number,
                        placement,
                    })
                },
                _ => None,
            }
        } else {
            Some(ParsedKittyCommand::TransmitChunk { more, payload })
        }
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

fn serialize_transmit(image_id: u32, image_data: &KittyImageData) -> Vec<String> {
    const KITTY_TRANSMIT_CHUNK_SIZE: usize = 3072;

    let mut parts = vec![
        "a=t".to_string(),
        format!("i={}", image_id),
        "q=2".to_string(),
    ];
    let payload = match image_data {
        KittyImageData::Png { data, .. } => {
            parts.push("f=100".to_string());
            data
        },
        KittyImageData::Rgb {
            data,
            width,
            height,
        } => {
            parts.push("f=24".to_string());
            parts.push(format!("s={}", width));
            parts.push(format!("v={}", height));
            data
        },
        KittyImageData::Rgba {
            data,
            width,
            height,
        } => {
            parts.push("f=32".to_string());
            parts.push(format!("s={}", width));
            parts.push(format!("v={}", height));
            data
        },
    };
    let payload = base64::encode(payload);
    let payload_parts: Vec<&str> = if payload.is_empty() {
        vec![""]
    } else {
        payload
            .as_bytes()
            .chunks(KITTY_TRANSMIT_CHUNK_SIZE)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect()
    };
    let last_index = payload_parts.len().saturating_sub(1);
    payload_parts
        .into_iter()
        .enumerate()
        .map(|(index, payload_part)| {
            let more = if index < last_index { 1 } else { 0 };
            if index == 0 {
                format!("{},m={};{}", parts.join(","), more, payload_part)
            } else {
                format!("m={};{}", more, payload_part)
            }
        })
        .collect()
}

fn serialize_display(chunk: &KittyImageChunk, placement_id: u32) -> String {
    let mut parts = vec![
        "a=p".to_string(),
        format!("i={}", chunk.image_id),
        format!("p={}", placement_id),
        "q=2".to_string(),
        "C=1".to_string(),
        format!("x={}", chunk.source_x),
        format!("y={}", chunk.source_y),
        format!("w={}", chunk.source_width),
        format!("h={}", chunk.source_height),
    ];
    if chunk.columns_specified {
        parts.push(format!("c={}", chunk.columns));
    }
    if chunk.rows_specified {
        parts.push(format!("r={}", chunk.rows));
    }
    if chunk.x_offset > 0 {
        parts.push(format!("X={}", chunk.x_offset));
    }
    if chunk.y_offset > 0 {
        parts.push(format!("Y={}", chunk.y_offset));
    }
    if chunk.z_index != 0 {
        parts.push(format!("z={}", chunk.z_index));
    }
    parts.join(",")
}

fn serialize_placeholder_render(render: &KittyPlaceholderRender, placement_id: u32) -> String {
    let mut output = String::new();
    output.push_str("\u{1b}_G");
    output.push_str(&serialize_virtual_placeholder_placement(
        render,
        placement_id,
    ));
    output.push_str("\u{1b}\\");

    let image_id_low_24 = render.image_id & 0x00FF_FFFF;
    let image_id_r = ((image_id_low_24 >> 16) & 0xFF) as u8;
    let image_id_g = ((image_id_low_24 >> 8) & 0xFF) as u8;
    let image_id_b = (image_id_low_24 & 0xFF) as u8;
    let placement_id_low_24 = placement_id & 0x00FF_FFFF;
    let placement_id_r = ((placement_id_low_24 >> 16) & 0xFF) as u8;
    let placement_id_g = ((placement_id_low_24 >> 8) & 0xFF) as u8;
    let placement_id_b = (placement_id_low_24 & 0xFF) as u8;
    let image_id_high_byte = ((render.image_id >> 24) & 0xFF) as usize;
    let image_id_high_diacritic = KITTY_ROWCOL_DIACRITICS.get(image_id_high_byte).copied();

    for cell in &render.cells {
        output.push_str(&format!("\u{1b}[{};{}H", cell.cell_y + 1, cell.cell_x + 1));
        output.push_str(&format!(
            "\u{1b}[38;2;{};{};{}m\u{1b}[58;2;{};{};{}m",
            image_id_r, image_id_g, image_id_b, placement_id_r, placement_id_g, placement_id_b,
        ));
        let Some(row_diacritic) = KITTY_ROWCOL_DIACRITICS.get(cell.placeholder_row).copied() else {
            continue;
        };
        let Some(col_diacritic) = KITTY_ROWCOL_DIACRITICS.get(cell.placeholder_col).copied() else {
            continue;
        };
        output.push(KITTY_UNICODE_PLACEHOLDER_CHAR);
        output.push(row_diacritic);
        output.push(col_diacritic);
        if let Some(image_id_high_diacritic) = image_id_high_diacritic {
            output.push(image_id_high_diacritic);
        }
        output.push_str("\u{1b}[0m");
    }
    output
}

fn serialize_virtual_placeholder_placement(
    render: &KittyPlaceholderRender,
    placement_id: u32,
) -> String {
    let mut parts = vec![
        "a=p".to_string(),
        "U=1".to_string(),
        format!("i={}", render.image_id),
        format!("p={}", placement_id),
        "q=2".to_string(),
        "C=1".to_string(),
        format!("x={}", render.source_x),
        format!("y={}", render.source_y),
        format!("w={}", render.source_width),
        format!("h={}", render.source_height),
        format!("c={}", render.columns),
        format!("r={}", render.rows),
    ];
    if render.x_offset > 0 {
        parts.push(format!("X={}", render.x_offset));
    }
    if render.y_offset > 0 {
        parts.push(format!("Y={}", render.y_offset));
    }
    parts.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pid(value: u32) -> PlacementId {
        PlacementId::Protocol(value)
    }

    fn assert_delete_request(apc_bytes: &[u8], expected: KittyDeleteRequest) {
        assert_eq!(kitty_delete_request(apc_bytes), Some(expected));
    }

    fn test_image_dimensions(width: u32, height: u32) -> (u32, u32) {
        (width, height)
    }

    #[test]
    fn kitty_delete_request_parses_geometry_selectors() {
        assert_delete_request(
            b"Ga=d,d=c",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cursor),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=C",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cursor),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=p,x=24,y=11",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 24,
                    y: 11,
                    z: None,
                }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=P,x=24,y=11",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 24,
                    y: 11,
                    z: None,
                }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=q,x=24,y=11,z=-1",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 24,
                    y: 11,
                    z: Some(-1),
                }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=Q,x=24,y=11,z=-1",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 24,
                    y: 11,
                    z: Some(-1),
                }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=x,x=8",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Column { x: 8 }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=X,x=8",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Column { x: 8 }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=y,y=11",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Row { y: 11 }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=Y,y=11",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Row { y: 11 }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=z,z=-1",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Z { z: -1 }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=Z,z=-1",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Z { z: -1 }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
    }

    #[test]
    fn kitty_delete_request_parses_range_selectors() {
        assert_delete_request(
            b"Ga=d,d=r,x=200,y=204",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Range {
                    first_image_id: 200,
                    last_image_id: 204,
                },
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=R,x=200,y=204",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Range {
                    first_image_id: 200,
                    last_image_id: 204,
                },
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
    }

    #[test]
    fn kitty_query_response_honors_errors_and_quiet_modes() {
        let success = kitty_query_response(b"Gq=0,a=q,t=d,f=24,s=1,v=1,i=41;EjRW").unwrap();
        assert_eq!(success.to_apc_response(), "\u{1b}_Gi=41;OK\u{1b}\\");

        let invalid_both_ids =
            kitty_query_response(b"Gq=0,a=q,t=d,f=24,s=1,v=1,i=42,I=1;EjRW").unwrap();
        let invalid_response = invalid_both_ids.to_apc_response();
        assert!(invalid_response.contains("i=42,I=1;EINVAL:"));

        let quiet_success = kitty_query_response(b"Gq=1,a=q,t=d,f=24,s=1,v=1,i=43;EjRW");
        assert!(quiet_success.is_none());

        let quiet_failure = kitty_query_response(b"Gq=2,a=q,t=d,f=24,s=1,v=1,i=44,I=1;EjRW");
        assert!(quiet_failure.is_none());
    }

    #[test]
    fn kitty_query_response_rejects_unsupported_transmission_media() {
        let cases = [
            (
                b"Gq=0,a=q,t=f,f=24,s=1,v=1,i=45;L3RtcC9raXR0eS1xdWVyeS1maWxl" as &[u8],
                45u32,
            ),
            (
                b"Gq=0,a=q,t=t,f=24,s=1,v=1,i=46;L3RtcC9raXR0eS1xdWVyeS10ZW1w" as &[u8],
                46u32,
            ),
            (
                b"Gq=0,a=q,t=s,f=24,s=1,v=1,i=47;a2l0dHktcXVlcnktc2ht" as &[u8],
                47u32,
            ),
        ];

        for (query, image_id) in cases {
            let reply = kitty_query_response(query).unwrap();
            let response = reply.to_apc_response();
            assert!(
                response.contains(&format!(
                    "i={image_id};EINVAL:Unsupported transmission medium"
                )),
                "expected unsupported-medium query reply for i={image_id}, got {response:?}",
            );
        }
    }

    #[test]
    fn kitty_query_response_suppresses_unsupported_media_failures_for_q2() {
        let quiet_failure =
            kitty_query_response(b"Gq=2,a=q,t=f,f=24,s=1,v=1,i=48;L3RtcC9raXR0eS1xdWVyeS1maWxl");
        assert!(quiet_failure.is_none());
    }

    #[test]
    fn kitty_non_query_response_suppresses_success_for_q2() {
        let reply =
            kitty_non_query_response(b"Gq=2,a=T,f=24,s=1,v=1,i=52,c=1,r=1;EjRW", true, None, None);
        assert!(reply.is_none());
    }

    #[test]
    fn kitty_rgb24_payloads_roundtrip_natively() {
        let payload = vec![0x12, 0x34, 0x56];
        let parsed = ParsedKittyCommand::parse(b"Ga=t,f=24,s=1,v=1,i=7;EjRW").unwrap();
        let ParsedKittyCommand::ImmediateTransmit {
            protocol_image_id,
            image_format,
            width,
            height,
            payload: parsed_payload,
            ..
        } = parsed
        else {
            panic!("expected immediate transmit");
        };
        assert_eq!(protocol_image_id, Some(7));
        assert_eq!(image_format, KittyImageFormat::Rgb);
        assert_eq!(width, 1);
        assert_eq!(height, 1);
        assert_eq!(parsed_payload, payload);

        let image_data = PendingKittyTransmit {
            protocol_image_id: Some(7),
            image_number: None,
            image_id: 99,
            image_format,
            compression: None,
            width,
            height,
            placement: None,
            reply_context: PendingKittyReplyContext {
                kind: PendingKittyReplyKind::Transmit,
                quiet: 0,
                parsed_image_id: Some(7),
                placement_id: None,
                image_number: None,
            },
            payload: payload.clone(),
        }
        .into_image_data()
        .unwrap();
        match image_data {
            KittyImageData::Rgb {
                data,
                width,
                height,
            } => {
                assert_eq!(data, payload);
                assert_eq!(width, 1);
                assert_eq!(height, 1);
            },
            other => panic!("expected rgb image data, got {:?}", other),
        }

        let serialized = serialize_transmit(
            99,
            &KittyImageData::Rgb {
                data: vec![0x12, 0x34, 0x56],
                width: 1,
                height: 1,
            },
        );
        assert_eq!(serialized.len(), 1);
        assert!(serialized[0].contains("a=t"));
        assert!(serialized[0].contains("f=24"));
        assert!(serialized[0].contains("s=1"));
        assert!(serialized[0].contains("v=1"));
        assert!(serialized[0].ends_with(";EjRW"));
    }

    #[test]
    fn one_dimensional_kitty_sizing_preserves_prediction_and_wire_intent() {
        let image_dimensions = test_image_dimensions(40, 20);
        let cell_size = Some(SizeInPixels {
            width: 10,
            height: 10,
        });
        let cases = vec![
            (Some(3), Some(5), 3usize, 5usize, true, true),
            (Some(3), None, 3usize, 2usize, true, false),
            (None, Some(3), 6usize, 3usize, false, true),
        ];

        for (columns, rows, expected_columns, expected_rows, expect_c, expect_r) in cases {
            let placement = KittyPlacement {
                image_id: 1,
                placement_id: Some(pid(7)),
                placement_mode: KittyImagePlacementMode::Explicit,
                cursor_movement_policy: KittyCursorMovementPolicy::AfterPlacement,
                anchor: FlowAnchor::LogicalRow {
                    logical_row: 0,
                    column: 0,
                },
                source_x: None,
                source_y: None,
                source_width: None,
                source_height: None,
                columns,
                rows,
                columns_specified: columns.is_some(),
                rows_specified: rows.is_some(),
                x_offset: None,
                y_offset: None,
                z_index: None,
            };
            let geometry = placement.geometry_for_image(image_dimensions, 0, 0, cell_size);
            assert_eq!(geometry.columns, expected_columns);
            assert_eq!(geometry.rows, expected_rows);
            assert_eq!(geometry.columns_specified, expect_c);
            assert_eq!(geometry.rows_specified, expect_r);

            let chunk = KittyImageChunk {
                image_id: 1,
                placement_id: Some(pid(7)),
                placement_mode: KittyImagePlacementMode::Explicit,
                cell_x: 0,
                cell_y: 0,
                columns: geometry.columns,
                rows: geometry.rows,
                columns_specified: geometry.columns_specified,
                rows_specified: geometry.rows_specified,
                source_x: geometry.source_x,
                source_y: geometry.source_y,
                source_width: geometry.source_width,
                source_height: geometry.source_height,
                z_index: geometry.z_index,
                x_offset: geometry.x_offset,
                y_offset: geometry.y_offset,
            };
            let serialized = serialize_display(&chunk, 7);
            assert_eq!(serialized.contains("c="), expect_c);
            assert_eq!(serialized.contains("r="), expect_r);
        }
    }

    #[test]
    fn naive_bounded_conversion_for_one_dimensional_geometry_does_not_preserve_rendered_pixel_size()
    {
        let image_dimensions = test_image_dimensions(16, 9);
        let cell_size = Some(SizeInPixels {
            width: 10,
            height: 20,
        });

        let columns_only = KittyPlacement {
            image_id: 1,
            placement_id: Some(pid(7)),
            placement_mode: KittyImagePlacementMode::Explicit,
            cursor_movement_policy: KittyCursorMovementPolicy::AfterPlacement,
            anchor: FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            source_x: None,
            source_y: None,
            source_width: None,
            source_height: None,
            columns: Some(10),
            rows: None,
            columns_specified: true,
            rows_specified: false,
            x_offset: None,
            y_offset: None,
            z_index: None,
        };

        let geometry = columns_only.geometry_for_image(image_dimensions, 0, 0, cell_size);
        assert_eq!(geometry.columns, 10);
        assert_eq!(geometry.rows, 3);

        // Kitty/Ghostty calculate the one-dimensional rendered size in pixels first.
        let one_dimensional_rendered_width = 10 * 10;
        let one_dimensional_rendered_height = 56;

        // A naive conversion to bounded c+r uses the derived row count as a fit box.
        let bounded_box_width = geometry.columns * 10;
        let bounded_box_height = geometry.rows * 20;

        assert_eq!(bounded_box_width, one_dimensional_rendered_width);
        assert_eq!(
            bounded_box_height, one_dimensional_rendered_height,
            "naively turning a one-dimensional placement into bounded c+r changes the rendered pixel size"
        );
    }

    #[test]
    fn bounded_conversion_with_offsets_can_preserve_one_dimensional_rendered_pixel_size() {
        let cell_size = SizeInPixels {
            width: 10,
            height: 20,
        };
        let one_dimensional_rendered_width = 100usize;
        let one_dimensional_rendered_height = 56usize;

        // One plausible bounded equivalent is a 10x3 box with the rendered image
        // starting at the top-left and leaving trailing slack in the final row.
        let bounded_chunk = KittyImageChunk {
            image_id: 1,
            placement_id: Some(pid(7)),
            placement_mode: KittyImagePlacementMode::Explicit,
            cell_x: 0,
            cell_y: 0,
            columns: 10,
            rows: 3,
            columns_specified: true,
            rows_specified: true,
            source_x: 0,
            source_y: 0,
            source_width: 16,
            source_height: 9,
            z_index: 0,
            x_offset: 0,
            y_offset: 0,
        };

        let rendered_width = bounded_chunk.columns * cell_size.width as usize;
        let rendered_height =
            bounded_chunk.rows * cell_size.height as usize - bounded_chunk.y_offset as usize;

        assert_eq!(rendered_width, one_dimensional_rendered_width);
        assert_eq!(
            rendered_height, one_dimensional_rendered_height,
            "a bounded c+r conversion with offsets should be able to preserve the one-dimensional rendered pixel size"
        );
    }
}
