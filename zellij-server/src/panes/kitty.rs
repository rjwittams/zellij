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
use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::rc::Rc;

const KITTY_TEMP_FILE_MARKER: &str = "tty-graphics-protocol";

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
        Some(AnsiCode::RgbCode((r, g, b))) => Some(PlacementId::Protocol(
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        )),
        _ => None,
    }
}

const KITTY_RELATIVE_PARENT_DEPTH_LIMIT: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyPlacement {
    pub image_id: u32,
    pub protocol_image_id: Option<u32>,
    pub placement_id: Option<PlacementId>,
    pub relative_to: Option<KittyRelativePlacement>,
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
            protocol_image_id: None,
            placement_id: None,
            relative_to: None,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KittyRelativePlacement {
    pub parent_image_id: u32,
    pub parent_placement_id: Option<PlacementId>,
    pub offset_x: i32,
    pub offset_y: i32,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KittyTransmissionMedium {
    Direct,
    RegularFile,
    TemporaryFile,
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
    pub placement: KittyPlacement,
    pub image_dimensions: (u32, u32),
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

    fn relative_parent_placement(
        &self,
        relative_to: &KittyRelativePlacement,
    ) -> Option<&KittyPlacement> {
        self.placements.iter().find(|placement| {
            placement.protocol_image_id == Some(relative_to.parent_image_id)
                && placement.placement_id == relative_to.parent_placement_id
        })
    }

    fn has_relative_parent(&self, relative_to: &KittyRelativePlacement) -> bool {
        self.relative_parent_placement(relative_to).is_some()
    }

    fn would_create_relative_cycle(
        &self,
        protocol_image_id: Option<u32>,
        placement_id: Option<PlacementId>,
        relative_to: &KittyRelativePlacement,
    ) -> bool {
        let Some(protocol_image_id) = protocol_image_id else {
            return false;
        };
        let mut current_parent = self.relative_parent_placement(relative_to);
        let target = (protocol_image_id, placement_id);
        let mut traversed = 0usize;
        while let Some(parent) = current_parent {
            if (parent.protocol_image_id, parent.placement_id) == (Some(target.0), target.1) {
                return true;
            }
            current_parent = parent
                .relative_to
                .as_ref()
                .and_then(|relative_to| self.relative_parent_placement(relative_to));
            traversed += 1;
            if traversed > self.placements.len() {
                return true;
            }
        }
        false
    }

    fn exceeds_relative_depth_limit(&self, relative_to: &KittyRelativePlacement) -> bool {
        let mut current_parent = self.relative_parent_placement(relative_to);
        let mut depth = 0usize;
        while let Some(parent) = current_parent {
            if depth >= KITTY_RELATIVE_PARENT_DEPTH_LIMIT {
                return true;
            }
            depth += 1;
            current_parent = parent
                .relative_to
                .as_ref()
                .and_then(|relative_to| self.relative_parent_placement(relative_to));
        }
        false
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

    fn referenced_image_ids(&self) -> HashSet<u32> {
        self.placements
            .iter()
            .map(|placement| placement.image_id)
            .collect()
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
            placement.protocol_image_id = protocol_image_id;
        }
        let protected_image_ids = self.referenced_image_ids();
        let evicted_image_ids = self.kitty_asset_store.borrow_mut().insert_asset_protecting(
            image_id,
            image_data,
            &protected_image_ids,
        );
        for evicted_image_id in evicted_image_ids {
            self.remove_protocol_references_for_internal_image_id(evicted_image_id);
        }
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
        let cursor_movement_policy = if placement.relative_to.is_some() {
            KittyCursorMovementPolicy::NoMovement
        } else {
            placement.cursor_movement_policy
        };
        let placement_anchor = placement.anchor.clone();
        let geometry = placement.geometry_for_image(
            image_dimensions,
            cursor_x,
            scrollback_row,
            character_cell_size,
        );
        self.placements.push(placement.clone());
        KittyApcOutcome {
            effect: Some(KittyApcEffect::Placement(KittyImageInsertion {
                asset_id,
                placement,
                image_dimensions,
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
                    placement.protocol_image_id = resolved_protocol_image_id;
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
                placement.protocol_image_id = Some(resolved_protocol_image_id);
                if placement.placement_mode == KittyImagePlacementMode::Placeholder
                    && placement.relative_to.is_some()
                {
                    return KittyApcOutcome {
                        effect: None,
                        reply: build_non_query_reply(
                            &reply_context,
                            Some(resolved_protocol_image_id),
                            image_number,
                            Some("EINVAL:Virtual placements cannot be relative".to_string()),
                        ),
                    };
                }
                if let Some(relative_to) = placement.relative_to {
                    if !self.has_relative_parent(&relative_to) {
                        return KittyApcOutcome {
                            effect: None,
                            reply: build_non_query_reply(
                                &reply_context,
                                Some(resolved_protocol_image_id),
                                image_number,
                                Some(format!(
                                    "ENOPARENT:Parent placement not found for parent image id: {} and placement id: {:?}",
                                    relative_to.parent_image_id, relative_to.parent_placement_id
                                )),
                            ),
                        };
                    }
                    if self.would_create_relative_cycle(
                        Some(resolved_protocol_image_id),
                        placement.placement_id,
                        &relative_to,
                    ) {
                        return KittyApcOutcome {
                            effect: None,
                            reply: build_non_query_reply(
                                &reply_context,
                                Some(resolved_protocol_image_id),
                                image_number,
                                Some("ECYCLE:Relative placement cycle detected".to_string()),
                            ),
                        };
                    }
                    if self.exceeds_relative_depth_limit(&relative_to) {
                        return KittyApcOutcome {
                            effect: None,
                            reply: build_non_query_reply(
                                &reply_context,
                                Some(resolved_protocol_image_id),
                                image_number,
                                Some("ETOODEEP:Too many levels of parent references".to_string()),
                            ),
                        };
                    }
                }
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
                let cursor_movement_policy = if placement.relative_to.is_some() {
                    KittyCursorMovementPolicy::NoMovement
                } else {
                    placement.cursor_movement_policy
                };
                let placement_anchor = placement.anchor.clone();
                self.placements.push(placement.clone());
                KittyApcOutcome {
                    effect: Some(KittyApcEffect::Placement(KittyImageInsertion {
                        asset_id: ImageAssetId(image_id as u64),
                        placement,
                        image_dimensions,
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
                stable_render_id: placement.image_id as u64,
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

    fn remove_protocol_references_for_internal_image_id(&mut self, internal_image_id: u32) {
        let protocol_image_ids = self
            .protocol_image_id_to_internal_id
            .iter()
            .filter_map(|(protocol_image_id, mapped_internal_image_id)| {
                (*mapped_internal_image_id == internal_image_id).then_some(*protocol_image_id)
            })
            .collect::<Vec<_>>();
        for protocol_image_id in protocol_image_ids {
            self.remove_protocol_image_references(protocol_image_id);
        }
    }

    pub fn delete_protocol_placement(
        &mut self,
        protocol_image_id: u32,
        placement_id: Option<PlacementId>,
        free_image_data: bool,
    ) {
        let mut removed_indices = std::collections::HashSet::new();
        let mut removed_parent_keys = Vec::new();
        for (index, placement) in self.placements.iter().enumerate() {
            let matches_target = placement.protocol_image_id == Some(protocol_image_id)
                && match placement_id {
                    Some(placement_id) => placement.placement_id == Some(placement_id),
                    None => true,
                };
            if matches_target {
                removed_indices.insert(index);
                if let Some(protocol_image_id) = placement.protocol_image_id {
                    removed_parent_keys.push((protocol_image_id, placement.placement_id));
                }
            }
        }
        if removed_indices.is_empty() {
            return;
        }
        loop {
            let mut added_descendant = false;
            for (index, placement) in self.placements.iter().enumerate() {
                if removed_indices.contains(&index) {
                    continue;
                }
                let Some(relative_to) = placement.relative_to.as_ref() else {
                    continue;
                };
                if removed_parent_keys
                    .contains(&(relative_to.parent_image_id, relative_to.parent_placement_id))
                {
                    removed_indices.insert(index);
                    if let Some(protocol_image_id) = placement.protocol_image_id {
                        removed_parent_keys.push((protocol_image_id, placement.placement_id));
                    }
                    added_descendant = true;
                }
            }
            if !added_descendant {
                break;
            }
        }
        let removed_internal_image_ids: std::collections::HashSet<u32> = removed_indices
            .iter()
            .filter_map(|index| {
                self.placements
                    .get(*index)
                    .map(|placement| placement.image_id)
            })
            .collect();
        self.placements = self
            .placements
            .iter()
            .enumerate()
            .filter(|(index, _)| !removed_indices.contains(index))
            .map(|(_, placement)| placement.clone())
            .collect();
        if free_image_data {
            for internal_image_id in removed_internal_image_ids {
                let has_remaining_references = self
                    .placements
                    .iter()
                    .any(|placement| placement.image_id == internal_image_id);
                if has_remaining_references {
                    continue;
                }
                self.kitty_asset_store
                    .borrow_mut()
                    .remove_asset(internal_image_id);
                self.remove_protocol_references_for_internal_image_id(internal_image_id);
            }
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
    pub(crate) fn geometry_for_image(
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

fn apply_kitty_transport_compression(
    payload: Vec<u8>,
    compression: Option<&str>,
) -> Option<Vec<u8>> {
    match compression {
        Some("z") => decompress_to_vec_zlib(&payload).ok(),
        Some(_) => None,
        None => Some(payload),
    }
}

fn decode_kitty_transport_payload(payload_b64: &[u8]) -> Option<Vec<u8>> {
    base64::decode(payload_b64).ok()
}

fn parse_kitty_transmission_medium(medium: Option<&str>) -> Option<KittyTransmissionMedium> {
    match medium {
        Some("f") => Some(KittyTransmissionMedium::RegularFile),
        Some("t") => Some(KittyTransmissionMedium::TemporaryFile),
        Some("d") | None => Some(KittyTransmissionMedium::Direct),
        Some(_) => None,
    }
}

fn parse_kitty_payload_byte_range(
    size: Option<&str>,
    offset: Option<&str>,
) -> Option<(Option<usize>, u64)> {
    let size = match size {
        Some(size) => Some(size.parse::<usize>().ok()?),
        None => None,
    };
    let offset = match offset {
        Some(offset) => offset.parse::<u64>().ok()?,
        None => 0,
    };
    Some((size, offset))
}

fn kitty_path_payload(path_payload: &[u8]) -> Option<&Path> {
    Some(Path::new(std::str::from_utf8(path_payload).ok()?))
}

fn read_kitty_regular_file_payload(
    path_payload: &[u8],
    size: Option<usize>,
    offset: u64,
) -> Option<Vec<u8>> {
    read_kitty_file_payload(kitty_path_payload(path_payload)?, size, offset)
}

fn read_kitty_file_payload(path: &Path, size: Option<usize>, offset: u64) -> Option<Vec<u8>> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let mut file = std::fs::File::open(path).ok()?;
    if offset > 0 {
        file.seek(SeekFrom::Start(offset)).ok()?;
    }
    let mut payload = Vec::new();
    match size {
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

fn read_kitty_temporary_file_payload(
    path_payload: &[u8],
    size: Option<usize>,
    offset: u64,
) -> Option<Vec<u8>> {
    let path = kitty_path_payload(path_payload)?;
    let payload = read_kitty_file_payload(path, size, offset)?;
    if is_safe_kitty_temporary_file_path(path) {
        std::fs::remove_file(path).ok();
    }
    Some(payload)
}

fn read_kitty_transmission_payload(
    payload_b64: &[u8],
    medium: Option<&str>,
    size: Option<&str>,
    offset: Option<&str>,
) -> Option<Vec<u8>> {
    let payload = decode_kitty_transport_payload(payload_b64)?;
    match parse_kitty_transmission_medium(medium)? {
        KittyTransmissionMedium::Direct => Some(payload),
        KittyTransmissionMedium::RegularFile => {
            let (size, offset) = parse_kitty_payload_byte_range(size, offset)?;
            read_kitty_regular_file_payload(&payload, size, offset)
        },
        KittyTransmissionMedium::TemporaryFile => {
            let (size, offset) = parse_kitty_payload_byte_range(size, offset)?;
            read_kitty_temporary_file_payload(&payload, size, offset)
        },
    }
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
        .filter(|placement_id| *placement_id != 0)
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

pub fn kitty_delete_by_image_number(apc_bytes: &[u8]) -> Option<(u32, Option<PlacementId>, bool)> {
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
    let placement_id = kv
        .get("p")
        .and_then(|p| p.parse::<u32>().ok())
        .filter(|placement_id| *placement_id != 0);
    let image_number = kv.get("I").and_then(|i| i.parse::<u32>().ok());
    let transport = parse_kitty_transmission_medium(kv.get("t").copied());

    let reply = if image_id.is_some() && image_number.is_some() {
        KittyQueryResponse::Error {
            image_id,
            placement_id,
            image_number,
            message: "EINVAL:Must not specify both i and I".to_string(),
        }
    } else if transport.is_none() {
        KittyQueryResponse::Error {
            image_id,
            placement_id,
            image_number,
            message: "EINVAL:Unsupported transmission medium".to_string(),
        }
    } else {
        let payload = match read_kitty_transmission_payload(
            payload_b64,
            kv.get("t").copied(),
            kv.get("S").copied(),
            kv.get("O").copied(),
        )
        .and_then(|payload| apply_kitty_transport_compression(payload, kv.get("o").copied()))
        {
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
        placement_id: kv
            .get("p")
            .and_then(|p| p.parse::<u32>().ok())
            .filter(|placement_id| *placement_id != 0),
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
        let payload = read_kitty_transmission_payload(
            payload,
            kv.get("t").copied(),
            kv.get("S").copied(),
            kv.get("O").copied(),
        )?;

        if let Some(action) = kv.get("a") {
            let placement = KittyPlacement {
                image_id: 0,
                placement_id: kv
                    .get("p")
                    .and_then(|p| p.parse::<u32>().ok())
                    .filter(|placement_id| *placement_id != 0)
                    .map(PlacementId::Protocol),
                relative_to: kv
                    .get("P")
                    .and_then(|parent_image_id| parent_image_id.parse::<u32>().ok())
                    .map(|parent_image_id| KittyRelativePlacement {
                        parent_image_id,
                        parent_placement_id: kv
                            .get("Q")
                            .and_then(|placement_id| placement_id.parse::<u32>().ok())
                            .filter(|placement_id| *placement_id != 0)
                            .map(PlacementId::Protocol),
                        offset_x: kv
                            .get("H")
                            .and_then(|offset| offset.parse::<i32>().ok())
                            .unwrap_or(0),
                        offset_y: kv
                            .get("V")
                            .and_then(|offset| offset.parse::<i32>().ok())
                            .unwrap_or(0),
                    }),
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
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pid(value: u32) -> PlacementId {
        PlacementId::Protocol(value)
    }

    fn assert_delete_request(apc_bytes: &[u8], expected: KittyDeleteRequest) {
        assert_eq!(kitty_delete_request(apc_bytes), Some(expected));
    }

    fn test_image_dimensions(width: u32, height: u32) -> (u32, u32) {
        (width, height)
    }

    fn write_kitty_file_media_fixture(name: &str, bytes: &[u8]) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zellij-kitty-file-media-{name}-{}-{unique}.bin",
            std::process::id()
        ));
        std::fs::write(&path, bytes).expect("fixture file should be writable");
        path
    }

    fn write_kitty_safe_temp_media_fixture(name: &str, bytes: &[u8]) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "{KITTY_TEMP_FILE_MARKER}-zellij-{name}-{}-{unique}.bin",
            std::process::id()
        ));
        std::fs::write(&path, bytes).expect("fixture file should be writable");
        path
    }

    fn kitty_file_media_transmit_apc(
        medium: &str,
        image_id: u32,
        image_format: u32,
        path: &Path,
        size: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<u8> {
        let mut control = format!("Gq=0,a=t,t={medium},f={image_format},i={image_id}");
        if image_format != 100 {
            control.push_str(",s=2,v=2");
        }
        if let Some(size) = size {
            control.push_str(&format!(",S={size}"));
        }
        if let Some(offset) = offset {
            control.push_str(&format!(",O={offset}"));
        }
        control.push(';');
        control.push_str(&base64::encode(path.to_string_lossy().as_bytes()));
        control.into_bytes()
    }

    fn kitty_file_media_query_apc(
        medium: &str,
        image_id: u32,
        image_format: u32,
        path: &Path,
        size: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<u8> {
        let mut control = format!("Gq=0,a=q,t={medium},f={image_format},i={image_id}");
        if image_format != 100 {
            control.push_str(",s=2,v=2");
        }
        if let Some(size) = size {
            control.push_str(&format!(",S={size}"));
        }
        if let Some(offset) = offset {
            control.push_str(&format!(",O={offset}"));
        }
        control.push(';');
        control.push_str(&base64::encode(path.to_string_lossy().as_bytes()));
        control.into_bytes()
    }

    fn kitty_regular_file_transmit_apc(
        image_id: u32,
        path: &Path,
        size: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<u8> {
        kitty_file_media_transmit_apc("f", image_id, 32, path, size, offset)
    }

    fn kitty_regular_file_query_apc(
        image_id: u32,
        path: &Path,
        size: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<u8> {
        kitty_file_media_query_apc("f", image_id, 32, path, size, offset)
    }

    fn assert_stored_rgba_payload(
        kitty_state: &KittyImageState,
        kitty_asset_store: &Rc<RefCell<KittyAssetStore>>,
        protocol_image_id: u32,
        expected_payload: &[u8],
    ) {
        let internal_image_id = *kitty_state
            .protocol_image_id_to_internal_id
            .get(&protocol_image_id)
            .expect("protocol id should be mapped to an internal asset");
        match kitty_asset_store
            .borrow()
            .image_data(internal_image_id)
            .expect("image should be stored")
        {
            KittyImageData::Rgba {
                data,
                width,
                height,
            } => {
                assert_eq!(data, expected_payload);
                assert_eq!(width, 2);
                assert_eq!(height, 2);
            },
            other => panic!("expected rgba image data, got {:?}", other),
        }
    }

    #[test]
    fn kitty_regular_file_rgba_upload_reads_file_bytes() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = write_kitty_file_media_fixture("whole", &payload);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_regular_file_transmit_apc(601, &path, None, None),
            FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            0,
            0,
            None,
        );

        std::fs::remove_file(path).ok();
        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("OK"),
            "regular file upload should succeed, got {reply:?}"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 601, &payload);
    }

    #[test]
    fn kitty_regular_file_rgba_upload_honors_unaligned_offset_and_size() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let mut file_bytes = vec![0xAA, 0xBB, 0xCC];
        file_bytes.extend_from_slice(&payload);
        file_bytes.extend_from_slice(&[0xDD, 0xEE]);
        let path = write_kitty_file_media_fixture("offset", &file_bytes);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_regular_file_transmit_apc(602, &path, Some(payload.len()), Some(3)),
            FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            0,
            0,
            None,
        );

        std::fs::remove_file(path).ok();
        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("OK"),
            "regular file upload with unaligned O should succeed, got {reply:?}"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 602, &payload);
    }

    #[test]
    fn kitty_temporary_file_rgba_upload_deletes_safe_temp_file() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = write_kitty_safe_temp_media_fixture("whole", &payload);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_file_media_transmit_apc("t", 605, 32, &path, None, None),
            FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            0,
            0,
            None,
        );

        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("OK"),
            "temporary file upload should succeed, got {reply:?}"
        );
        assert!(
            !path.exists(),
            "safe temporary file should be deleted after read"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 605, &payload);
    }

    #[test]
    fn kitty_temporary_file_rgba_upload_keeps_temp_file_without_magic_name() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = write_kitty_file_media_fixture("temp-without-magic", &payload);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_file_media_transmit_apc("t", 606, 32, &path, None, None),
            FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            0,
            0,
            None,
        );

        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("OK"),
            "temporary file upload without magic name should still read, got {reply:?}"
        );
        assert!(path.exists(), "unsafe temporary file name should be kept");
        std::fs::remove_file(path).ok();
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 606, &payload);
    }

    #[test]
    fn kitty_temporary_file_rgba_upload_honors_offset_and_deletes_safe_temp_file() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let mut file_bytes = vec![0xAA; 4096];
        file_bytes.extend_from_slice(&payload);
        file_bytes.extend_from_slice(&[0xDD, 0xEE]);
        let path = write_kitty_safe_temp_media_fixture("offset", &file_bytes);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_file_media_transmit_apc("t", 607, 32, &path, Some(payload.len()), Some(4096)),
            FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            0,
            0,
            None,
        );

        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("OK"),
            "temporary file upload with S/O should succeed, got {reply:?}"
        );
        assert!(
            !path.exists(),
            "safe temporary file with S/O should be deleted after read"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 607, &payload);
    }

    #[test]
    fn kitty_temporary_file_invalid_png_upload_deletes_after_successful_read() {
        let path = write_kitty_safe_temp_media_fixture("invalid-png", b"not a png");
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_file_media_transmit_apc("t", 608, 100, &path, None, None),
            FlowAnchor::LogicalRow {
                logical_row: 0,
                column: 0,
            },
            0,
            0,
            None,
        );

        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("EINVAL:Invalid image payload for requested format"),
            "invalid PNG should fail after file read, got {reply:?}"
        );
        assert!(
            !path.exists(),
            "safe temporary file should be deleted even if decoded image is invalid"
        );
    }

    #[test]
    fn local_quota_eviction_removes_oldest_unplaced_asset_but_keeps_visible_assets() {
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::with_decoded_byte_quota(8)));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());
        let anchor = FlowAnchor::LogicalRow {
            logical_row: 0,
            column: 0,
        };
        let cell_size = Some(SizeInPixels {
            width: 1,
            height: 1,
        });

        let anchor_reply = kitty_state.handle_apc(
            b"Gq=0,a=T,C=1,f=24,s=1,v=1,i=131,p=1,c=1,r=1;EjRW",
            anchor.clone(),
            0,
            0,
            cell_size,
        );
        assert!(
            anchor_reply.reply.unwrap().to_apc_response().contains("OK"),
            "anchor placement should succeed"
        );
        let anchor_internal_image_id = *kitty_state
            .protocol_image_id_to_internal_id
            .get(&131)
            .expect("anchor should have an internal image id");

        let stored_first_reply = kitty_state.handle_apc(
            b"Gq=0,a=t,f=24,s=1,v=1,i=132;EjRW",
            anchor.clone(),
            0,
            0,
            cell_size,
        );
        assert!(
            stored_first_reply
                .reply
                .unwrap()
                .to_apc_response()
                .contains("OK"),
            "first stored-only image should fit the quota"
        );
        let first_stored_internal_image_id = *kitty_state
            .protocol_image_id_to_internal_id
            .get(&132)
            .expect("first stored image should have an internal image id");

        let stored_second_reply = kitty_state.handle_apc(
            b"Gq=0,a=t,f=24,s=1,v=1,i=133;EjRW",
            anchor.clone(),
            0,
            0,
            cell_size,
        );
        assert!(
            stored_second_reply
                .reply
                .unwrap()
                .to_apc_response()
                .contains("OK"),
            "new stored-only image should be accepted even when it triggers eviction"
        );

        assert!(
            kitty_asset_store
                .borrow()
                .image_data(anchor_internal_image_id)
                .is_some(),
            "visible anchor image should be protected from quota eviction"
        );
        assert!(
            kitty_asset_store
                .borrow()
                .image_data(first_stored_internal_image_id)
                .is_none(),
            "oldest unplaced image should be evicted under quota pressure"
        );
        let second_stored_internal_image_id = *kitty_state
            .protocol_image_id_to_internal_id
            .get(&133)
            .expect("second stored image should have an internal image id");
        assert!(
            kitty_asset_store
                .borrow()
                .image_data(second_stored_internal_image_id)
                .is_some(),
            "newly uploaded stored-only image should remain available"
        );

        let evicted_place_reply =
            kitty_state.handle_apc(b"Gq=0,a=p,i=132,p=1,c=1,r=1", anchor, 0, 0, cell_size);
        let evicted_place_reply = evicted_place_reply.reply.unwrap().to_apc_response();
        assert!(
            evicted_place_reply.contains("ENOENT"),
            "placing evicted image should fail, got {evicted_place_reply:?}"
        );
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
        let cases = [(
            b"Gq=0,a=q,t=s,f=24,s=1,v=1,i=47;a2l0dHktcXVlcnktc2ht" as &[u8],
            47u32,
        )];

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
            kitty_query_response(b"Gq=2,a=q,t=s,f=24,s=1,v=1,i=48;a2l0dHktcXVlcnktc2ht");
        assert!(quiet_failure.is_none());
    }

    #[test]
    fn kitty_query_response_accepts_regular_file_rgba_payload() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = write_kitty_file_media_fixture("query-whole", &payload);
        let query = kitty_regular_file_query_apc(603, &path, None, None);

        let reply = kitty_query_response(&query).unwrap().to_apc_response();

        std::fs::remove_file(path).ok();
        assert_eq!(reply, "\u{1b}_Gi=603;OK\u{1b}\\");
    }

    #[test]
    fn kitty_query_response_honors_regular_file_unaligned_offset_and_size() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let mut file_bytes = vec![0xAA, 0xBB, 0xCC];
        file_bytes.extend_from_slice(&payload);
        file_bytes.extend_from_slice(&[0xDD, 0xEE]);
        let path = write_kitty_file_media_fixture("query-offset", &file_bytes);
        let query = kitty_regular_file_query_apc(604, &path, Some(payload.len()), Some(3));

        let reply = kitty_query_response(&query).unwrap().to_apc_response();

        std::fs::remove_file(path).ok();
        assert_eq!(reply, "\u{1b}_Gi=604;OK\u{1b}\\");
    }

    #[test]
    fn kitty_query_response_accepts_temporary_file_rgba_payload_and_deletes_file() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = write_kitty_safe_temp_media_fixture("query-whole", &payload);
        let query = kitty_file_media_query_apc("t", 609, 32, &path, None, None);

        let reply = kitty_query_response(&query).unwrap().to_apc_response();

        assert_eq!(reply, "\u{1b}_Gi=609;OK\u{1b}\\");
        assert!(
            !path.exists(),
            "query action should delete safe temporary file after read"
        );
    }

    #[test]
    fn kitty_query_response_invalid_temporary_file_png_deletes_after_successful_read() {
        let path = write_kitty_safe_temp_media_fixture("query-invalid-png", b"not a png");
        let query = kitty_file_media_query_apc("t", 610, 100, &path, None, None);

        let reply = kitty_query_response(&query).unwrap().to_apc_response();

        assert!(
            reply.contains("i=610;EINVAL:Invalid image payload for requested format"),
            "invalid temporary PNG query should fail after file read, got {reply:?}"
        );
        assert!(
            !path.exists(),
            "query action should delete safe temporary file even if image data is invalid"
        );
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
                protocol_image_id: Some(1),
                placement_id: Some(pid(7)),
                relative_to: None,
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
                stable_render_id: 7,
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
            protocol_image_id: Some(1),
            placement_id: Some(pid(7)),
            relative_to: None,
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
        assert_ne!(
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
            stable_render_id: 7,
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
            y_offset: 4,
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
