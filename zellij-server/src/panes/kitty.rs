use base64;
use miniz_oxide::inflate::decompress_to_vec_zlib;
use zellij_utils::pane_size::SizeInPixels;

use crate::output::{
    KittyImageChunk, KittyImageData, KittyImagePlacementMode, KittyPlaceholderRender, PlacementId,
};
use crate::panes::kitty_asset_store::{
    KittyAssetData, KittyAssetFormat, KittyAssetStore, KittyByteRange, KittyExternalMedia,
    KittyExternalMediaLocation,
};
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
use std::fmt::Write as _;
use std::path::Path;
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
            None => inherited
                .map(|previous| previous.placeholder_row)
                .unwrap_or(0),
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
    image_id: Option<u32>,
    image_format: KittyImageFormat,
    compression: Option<KittyTransportCompression>,
    width: u32,
    height: u32,
    placement: Option<KittyPlacement>,
    reply_context: PendingKittyReplyContext,
    media_source: KittyMediaSource,
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
    SharedMemory,
}

const KITTY_MAX_PENDING_DIRECT_TRANSMIT_BYTES: usize = 80 * 1024 * 1024;
const KITTY_COMPRESSED_RAW_DIRECT_TRANSMIT_PADDING_BYTES: usize = 1024;

#[derive(Clone, Debug)]
enum KittyMediaSource {
    Direct(Vec<u8>),
    External(KittyExternalMedia),
}

impl KittyMediaSource {
    fn from_apc(apc: &KittyApc<'_>, medium: KittyTransmissionMedium) -> Option<Self> {
        match medium {
            KittyTransmissionMedium::Direct => Some(KittyMediaSource::Direct(
                decode_kitty_transport_payload(apc.payload)?,
            )),
            KittyTransmissionMedium::RegularFile => {
                let path_payload = decode_kitty_transport_payload(apc.payload)?;
                let (size, offset) = parse_kitty_payload_byte_range(apc.size, apc.offset)?;
                Some(KittyMediaSource::External(KittyExternalMedia::new(
                    KittyExternalMediaLocation::RegularFile(
                        kitty_path_payload(&path_payload)?.to_path_buf(),
                    ),
                    KittyByteRange { offset, size },
                )))
            },
            KittyTransmissionMedium::TemporaryFile => {
                let path_payload = decode_kitty_transport_payload(apc.payload)?;
                let (size, offset) = parse_kitty_payload_byte_range(apc.size, apc.offset)?;
                Some(KittyMediaSource::External(KittyExternalMedia::new(
                    KittyExternalMediaLocation::TemporaryFile(
                        kitty_path_payload(&path_payload)?.to_path_buf(),
                    ),
                    KittyByteRange { offset, size },
                )))
            },
            KittyTransmissionMedium::SharedMemory => {
                let name_payload = decode_kitty_transport_payload(apc.payload)?;
                let (size, offset) = parse_kitty_payload_byte_range(apc.size, apc.offset)?;
                Some(KittyMediaSource::External(KittyExternalMedia::new(
                    KittyExternalMediaLocation::SharedMemory(
                        std::str::from_utf8(&name_payload).ok()?.to_string(),
                    ),
                    KittyByteRange {
                        offset,
                        size: size.or(apc.raw_payload_size()),
                    },
                )))
            },
        }
    }

    fn into_payload(self) -> Result<Vec<u8>, String> {
        match self {
            KittyMediaSource::Direct(payload) => Ok(payload),
            KittyMediaSource::External(media) => media
                .into_payload()
                .ok_or_else(|| "EINVAL:Invalid or unsupported kitty command".to_string()),
        }
    }

    fn external_media(&self) -> Option<KittyExternalMedia> {
        match self {
            KittyMediaSource::External(media) => Some(media.clone()),
            KittyMediaSource::Direct(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct KittyImageState {
    kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
    placements: Vec<KittyPlacement>,
    protocol_image_id_to_internal_id: HashMap<u32, u32>,
    internal_image_id_to_protocol_image_ids: HashMap<u32, HashSet<u32>>,
    image_number_to_protocol_image_ids: HashMap<u32, Vec<u32>>,
    protocol_image_id_to_image_number: HashMap<u32, u32>,
    next_generated_protocol_image_id: u32,
    pending_transmit: Option<PendingKittyTransmit>,
}

impl Clone for KittyImageState {
    fn clone(&self) -> Self {
        for placement in &self.placements {
            self.kitty_asset_store
                .borrow_mut()
                .add_placement_reference(placement.image_id);
        }
        Self {
            kitty_asset_store: self.kitty_asset_store.clone(),
            placements: self.placements.clone(),
            protocol_image_id_to_internal_id: self.protocol_image_id_to_internal_id.clone(),
            internal_image_id_to_protocol_image_ids: self
                .internal_image_id_to_protocol_image_ids
                .clone(),
            image_number_to_protocol_image_ids: self.image_number_to_protocol_image_ids.clone(),
            protocol_image_id_to_image_number: self.protocol_image_id_to_image_number.clone(),
            next_generated_protocol_image_id: self.next_generated_protocol_image_id,
            pending_transmit: self.pending_transmit.clone(),
        }
    }
}

impl Drop for KittyImageState {
    fn drop(&mut self) {
        for placement in &self.placements {
            self.kitty_asset_store
                .borrow_mut()
                .remove_placement_reference(placement.image_id);
        }
    }
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
            internal_image_id_to_protocol_image_ids: HashMap::new(),
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
        } else if image_number.is_some() {
            let synthetic_id = self.next_synthetic_protocol_image_id();
            Some(synthetic_id)
        } else {
            None
        }
    }

    fn register_protocol_image_reference(
        &mut self,
        protocol_image_id: u32,
        internal_image_id: u32,
    ) {
        self.protocol_image_id_to_internal_id
            .insert(protocol_image_id, internal_image_id);
        self.internal_image_id_to_protocol_image_ids
            .entry(internal_image_id)
            .or_default()
            .insert(protocol_image_id);
    }

    fn register_image_number_reference(&mut self, image_number: u32, protocol_image_id: u32) {
        let protocol_image_ids = self
            .image_number_to_protocol_image_ids
            .entry(image_number)
            .or_default();
        if !protocol_image_ids.contains(&protocol_image_id) {
            protocol_image_ids.push(protocol_image_id);
        }
        self.protocol_image_id_to_image_number
            .insert(protocol_image_id, image_number);
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

    fn cleanup_abandoned_pending_transmit(&mut self) {
        let Some(pending) = self.pending_transmit.take() else {
            return;
        };
        if let Some(image_id) = pending.image_id {
            if self.kitty_asset_store.borrow().asset(image_id).is_none() {
                self.remove_protocol_references_for_internal_image_id(image_id);
            }
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

    fn push_placement(&mut self, placement: KittyPlacement) {
        self.kitty_asset_store
            .borrow_mut()
            .add_placement_reference(placement.image_id);
        self.placements.push(placement);
    }

    fn retain_placements<F>(&mut self, mut keep: F)
    where
        F: FnMut(&KittyPlacement) -> bool,
    {
        let mut removed_image_ids = Vec::new();
        self.placements.retain(|placement| {
            let should_keep = keep(placement);
            if !should_keep {
                removed_image_ids.push(placement.image_id);
            }
            should_keep
        });
        for image_id in removed_image_ids {
            self.kitty_asset_store
                .borrow_mut()
                .remove_placement_reference(image_id);
        }
    }

    fn clear_placements(&mut self) {
        for placement in &self.placements {
            self.kitty_asset_store
                .borrow_mut()
                .remove_placement_reference(placement.image_id);
        }
        self.placements.clear();
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
        let pending_image_id = pending.image_id;
        let mut placement = pending.placement.clone();
        let reply_context = pending.reply_context.clone();
        let replaced_existing_asset = pending_image_id
            .map(|image_id| self.kitty_asset_store.borrow().asset(image_id).is_some())
            .unwrap_or(false);
        let asset_data = match pending.into_asset_data() {
            Ok(asset_data) => asset_data,
            Err(message) => {
                if let Some(image_id) = pending_image_id.filter(|_| !replaced_existing_asset) {
                    self.remove_protocol_references_for_internal_image_id(image_id);
                }
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
        let image_id = if let Some(image_id) = pending_image_id {
            image_id
        } else {
            let Some(image_id) = self.kitty_asset_store.borrow_mut().next_asset_id() else {
                return KittyApcOutcome {
                    effect: None,
                    reply: build_non_query_reply(
                        &reply_context,
                        protocol_image_id,
                        image_number,
                        Some(kitty_asset_id_space_exhausted_message()),
                    ),
                };
            };
            if let Some(protocol_image_id) = protocol_image_id {
                self.register_protocol_image_reference(protocol_image_id, image_id);
            }
            image_id
        };
        if let (Some(image_number), Some(protocol_image_id)) = (image_number, protocol_image_id) {
            self.register_image_number_reference(image_number, protocol_image_id);
        }
        let image_dimensions = asset_data.dimensions();
        if let Some(placement) = placement.as_mut() {
            placement.image_id = image_id;
            placement.anchor = anchor.clone();
            placement.protocol_image_id = protocol_image_id;
        }
        let protected_image_ids = self.referenced_image_ids();
        let evicted_image_ids = self
            .kitty_asset_store
            .borrow_mut()
            .insert_asset_data_protecting(image_id, asset_data, &protected_image_ids);
        for evicted_image_id in evicted_image_ids {
            self.remove_protocol_references_for_internal_image_id(evicted_image_id);
        }
        let asset_id = ImageAssetId(image_id as u64);
        if replaced_existing_asset {
            self.retain_placements(|p| p.image_id != image_id);
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
        self.retain_placements(|p| {
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
        self.push_placement(placement.clone());
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
        let Some(apc) = KittyApc::parse(apc_bytes) else {
            return KittyApcOutcome {
                effect: None,
                reply: None,
            };
        };
        self.handle_parsed_apc(&apc, anchor, cursor_x, scrollback_row, character_cell_size)
    }

    pub fn handle_parsed_apc(
        &mut self,
        apc: &KittyApc<'_>,
        anchor: FlowAnchor,
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> KittyApcOutcome {
        let reply_context = apc.reply_context();
        let Some(command) = ParsedKittyCommand::parse(apc) else {
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
                media_source,
            } => {
                self.cleanup_abandoned_pending_transmit();
                let resolved_protocol_image_id =
                    self.protocol_image_id_for_create(protocol_image_id, image_number);
                let image_id = if let Some(protocol_image_id) = resolved_protocol_image_id {
                    self.protocol_image_id_to_internal_id
                        .get(&protocol_image_id)
                        .copied()
                } else {
                    None
                };
                if let (Some(placement), Some(image_id)) = (placement.as_mut(), image_id) {
                    placement.image_id = image_id;
                    placement.protocol_image_id = resolved_protocol_image_id;
                }
                let Some(reply_context) = reply_context else {
                    return KittyApcOutcome {
                        effect: None,
                        reply: None,
                    };
                };
                let pending = PendingKittyTransmit {
                    protocol_image_id: resolved_protocol_image_id,
                    image_number,
                    image_id,
                    image_format,
                    compression,
                    width,
                    height,
                    placement,
                    reply_context,
                    media_source,
                };
                if let Err(message) = pending.validate_current_direct_payload_size() {
                    let replaced_existing_asset = image_id
                        .map(|image_id| self.kitty_asset_store.borrow().asset(image_id).is_some())
                        .unwrap_or(false);
                    if let Some(image_id) = image_id.filter(|_| !replaced_existing_asset) {
                        self.remove_protocol_references_for_internal_image_id(image_id);
                    }
                    return KittyApcOutcome {
                        effect: None,
                        reply: build_non_query_reply(
                            &pending.reply_context,
                            pending.protocol_image_id,
                            pending.image_number,
                            Some(message),
                        ),
                    };
                }
                self.pending_transmit = Some(pending);
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
                self.retain_placements(|p| {
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
                self.push_placement(placement.clone());
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
                if apc.quiet.is_some() {
                    pending.reply_context.quiet = apc.quiet();
                }
                let append_result = pending.append_direct_payload(payload);
                if let Err(message) = append_result {
                    let pending = self.pending_transmit.take().expect("pending checked above");
                    let replaced_existing_asset = pending
                        .image_id
                        .map(|image_id| self.kitty_asset_store.borrow().asset(image_id).is_some())
                        .unwrap_or(false);
                    if let Some(image_id) = pending.image_id.filter(|_| !replaced_existing_asset) {
                        self.remove_protocol_references_for_internal_image_id(image_id);
                    }
                    return KittyApcOutcome {
                        effect: None,
                        reply: build_non_query_reply(
                            &pending.reply_context,
                            pending.protocol_image_id,
                            pending.image_number,
                            Some(message),
                        ),
                    };
                }
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
                cell_x,
                cell_y,
                columns,
                rows,
                columns_specified: placement.columns.is_some() || clipped_by_projection,
                rows_specified: placement.rows.is_some() || clipped_by_projection,
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
        self.clear_placements();
        self.protocol_image_id_to_internal_id.clear();
        self.internal_image_id_to_protocol_image_ids.clear();
        self.image_number_to_protocol_image_ids.clear();
        self.protocol_image_id_to_image_number.clear();
        self.pending_transmit = None;
    }

    pub fn abort_pending_transmit(&mut self) {
        self.cleanup_abandoned_pending_transmit();
    }

    fn remove_protocol_image_references(&mut self, protocol_image_id: u32) {
        if let Some(internal_image_id) = self
            .protocol_image_id_to_internal_id
            .remove(&protocol_image_id)
        {
            if let Some(protocol_image_ids) = self
                .internal_image_id_to_protocol_image_ids
                .get_mut(&internal_image_id)
            {
                protocol_image_ids.remove(&protocol_image_id);
                if protocol_image_ids.is_empty() {
                    self.internal_image_id_to_protocol_image_ids
                        .remove(&internal_image_id);
                }
            }
        }
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
            .internal_image_id_to_protocol_image_ids
            .get(&internal_image_id)
            .cloned()
            .unwrap_or_default();
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
        let removed_placements: Vec<_> = removed_indices
            .iter()
            .filter_map(|index| self.placements.get(*index).cloned())
            .collect();
        let removed_internal_image_ids: std::collections::HashSet<u32> = removed_placements
            .iter()
            .map(|placement| placement.image_id)
            .collect();
        for placement in &removed_placements {
            self.kitty_asset_store
                .borrow_mut()
                .remove_placement_reference(placement.image_id);
        }
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
                if has_remaining_references
                    || self
                        .kitty_asset_store
                        .borrow()
                        .has_placement_references(internal_image_id)
                {
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
        kitty_asset_store: &mut KittyAssetStore,
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

        for chunk in chunks {
            let placement_id = match chunk.placement_id {
                Some(PlacementId::Synthetic(value)) => value,
                Some(PlacementId::Protocol(_)) | None => chunk.stable_render_id as u32,
            };
            raw_vte_output.push_str(&Self::serialize_explicit_placement(chunk, placement_id));
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }

    pub fn serialize_placeholder_renders_with_asset_store(
        renders: &[KittyPlaceholderRender],
        kitty_asset_store: &mut KittyAssetStore,
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

        for render in renders {
            let placement_id = render.stable_render_id as u32;
            raw_vte_output.push_str(&Self::serialize_placeholder_render(render, placement_id));
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }

    pub fn serialize_full_scene_with_asset_store(
        chunks: &[KittyImageChunk],
        renders: &[KittyPlaceholderRender],
        kitty_asset_store: &mut KittyAssetStore,
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

        for chunk in chunks {
            let placement_id = match chunk.placement_id {
                Some(PlacementId::Synthetic(value)) => value,
                Some(PlacementId::Protocol(_)) | None => chunk.stable_render_id as u32,
            };
            raw_vte_output.push_str(&Self::serialize_explicit_placement(chunk, placement_id));
        }
        for render in renders {
            let placement_id = render.stable_render_id as u32;
            raw_vte_output.push_str(&Self::serialize_placeholder_render(render, placement_id));
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

    pub fn serialize_image_data_from_file(
        image_id: u32,
        image_data: &KittyImageData,
        path: &Path,
        quiet: u8,
    ) -> String {
        let format = KittyAssetFormat::from(image_data);
        let (width, height) = kitty_image_dimensions(image_data);
        Self::serialize_image_file(image_id, format, width, height, path, quiet)
    }

    pub fn serialize_asset_data_from_file(
        image_id: u32,
        asset_data: &KittyAssetData,
        path: &Path,
        quiet: u8,
    ) -> String {
        let (width, height) = asset_data.dimensions();
        Self::serialize_image_file(image_id, asset_data.format(), width, height, path, quiet)
    }

    fn serialize_image_file(
        image_id: u32,
        format: KittyAssetFormat,
        width: u32,
        height: u32,
        path: &Path,
        quiet: u8,
    ) -> String {
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str("\u{1b}_G");
        raw_vte_output.push_str(&serialize_transmit_file(
            image_id, format, width, height, path, quiet,
        ));
        raw_vte_output.push_str("\u{1b}\\");
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KittyTerminalImageResponse {
    pub image_id: Option<u32>,
    pub placement_id: Option<u32>,
    pub image_number: Option<u32>,
    pub is_ok: bool,
    pub message: String,
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

#[derive(Clone, Copy, Debug)]
pub struct KittyApc<'a> {
    payload: &'a [u8],
    action: Option<&'a str>,
    quiet: Option<&'a str>,
    image_id: Option<&'a str>,
    image_number: Option<&'a str>,
    placement_id: Option<&'a str>,
    delete_selector: Option<&'a str>,
    format: Option<&'a str>,
    width: Option<&'a str>,
    height: Option<&'a str>,
    medium: Option<&'a str>,
    size: Option<&'a str>,
    offset: Option<&'a str>,
    compression: Option<&'a str>,
    more: Option<&'a str>,
    parent_image_id: Option<&'a str>,
    parent_placement_id: Option<&'a str>,
    parent_offset_x: Option<&'a str>,
    parent_offset_y: Option<&'a str>,
    placement_mode: Option<&'a str>,
    source_x: Option<&'a str>,
    source_y: Option<&'a str>,
    source_width: Option<&'a str>,
    source_height: Option<&'a str>,
    columns: Option<&'a str>,
    rows: Option<&'a str>,
    x_offset: Option<&'a str>,
    y_offset: Option<&'a str>,
    z_index: Option<&'a str>,
    cursor_policy: Option<&'a str>,
}

impl<'a> KittyApc<'a> {
    pub fn parse(apc_bytes: &'a [u8]) -> Option<Self> {
        let rest = apc_bytes.strip_prefix(b"G")?;
        let mut parts = rest.splitn(2, |b| *b == b';');
        let header = std::str::from_utf8(parts.next()?).ok()?;
        let payload = parts.next().unwrap_or_default();
        let mut apc = KittyApc {
            payload,
            action: None,
            quiet: None,
            image_id: None,
            image_number: None,
            placement_id: None,
            delete_selector: None,
            format: None,
            width: None,
            height: None,
            medium: None,
            size: None,
            offset: None,
            compression: None,
            more: None,
            parent_image_id: None,
            parent_placement_id: None,
            parent_offset_x: None,
            parent_offset_y: None,
            placement_mode: None,
            source_x: None,
            source_y: None,
            source_width: None,
            source_height: None,
            columns: None,
            rows: None,
            x_offset: None,
            y_offset: None,
            z_index: None,
            cursor_policy: None,
        };
        for part in header.split(',') {
            if part.is_empty() {
                continue;
            }
            let mut split = part.splitn(2, '=');
            let key = split.next()?;
            let value = split.next().unwrap_or("");
            match key {
                "a" => apc.action = Some(value),
                "q" => apc.quiet = Some(value),
                "i" => apc.image_id = Some(value),
                "I" => apc.image_number = Some(value),
                "p" => apc.placement_id = Some(value),
                "d" => apc.delete_selector = Some(value),
                "f" => apc.format = Some(value),
                "s" => apc.width = Some(value),
                "v" => apc.height = Some(value),
                "t" => apc.medium = Some(value),
                "S" => apc.size = Some(value),
                "O" => apc.offset = Some(value),
                "o" => apc.compression = Some(value),
                "m" => apc.more = Some(value),
                "P" => apc.parent_image_id = Some(value),
                "Q" => apc.parent_placement_id = Some(value),
                "H" => apc.parent_offset_x = Some(value),
                "V" => apc.parent_offset_y = Some(value),
                "U" => apc.placement_mode = Some(value),
                "x" => apc.source_x = Some(value),
                "y" => apc.source_y = Some(value),
                "w" => apc.source_width = Some(value),
                "h" => apc.source_height = Some(value),
                "c" => apc.columns = Some(value),
                "r" => apc.rows = Some(value),
                "X" => apc.x_offset = Some(value),
                "Y" => apc.y_offset = Some(value),
                "z" => apc.z_index = Some(value),
                "C" => apc.cursor_policy = Some(value),
                _ => {},
            }
        }
        Some(apc)
    }

    fn quiet(&self) -> u8 {
        self.quiet.and_then(|q| q.parse::<u8>().ok()).unwrap_or(0)
    }

    fn parsed_image_id(&self) -> Option<u32> {
        self.image_id
            .and_then(|image_id| image_id.parse::<u32>().ok())
    }

    fn placement_id(&self) -> Option<PlacementId> {
        self.placement_id
            .and_then(|placement_id| placement_id.parse::<u32>().ok())
            .filter(|placement_id| *placement_id != 0)
            .map(PlacementId::Protocol)
    }

    fn placement_id_u32(&self) -> Option<u32> {
        self.placement_id
            .and_then(|placement_id| placement_id.parse::<u32>().ok())
            .filter(|placement_id| *placement_id != 0)
    }

    fn image_number(&self) -> Option<u32> {
        self.image_number
            .and_then(|image_number| image_number.parse::<u32>().ok())
    }

    fn more(&self) -> bool {
        self.more
            .and_then(|more| more.parse::<u8>().ok())
            .unwrap_or(0)
            != 0
    }

    fn raw_payload_size(&self) -> Option<usize> {
        let bytes_per_pixel = match self.format.unwrap_or("32") {
            "24" => 3usize,
            "32" => 4usize,
            _ => return None,
        };
        let width = self.width?.parse::<usize>().ok()?;
        let height = self.height?.parse::<usize>().ok()?;
        width
            .checked_mul(height)
            .and_then(|pixel_count| pixel_count.checked_mul(bytes_per_pixel))
    }

    fn read_payload(&self) -> Option<Vec<u8>> {
        read_kitty_transmission_payload(
            self.payload,
            self.medium,
            self.size,
            self.offset,
            self.raw_payload_size(),
        )
    }

    fn reply_context(&self) -> Option<PendingKittyReplyContext> {
        let kind = match self.action? {
            "t" | "T" => PendingKittyReplyKind::Transmit,
            "p" => PendingKittyReplyKind::Placement,
            _ => return None,
        };
        Some(PendingKittyReplyContext {
            kind,
            quiet: self.quiet(),
            parsed_image_id: self.parsed_image_id(),
            placement_id: self.placement_id_u32(),
            image_number: self.image_number(),
        })
    }

    pub fn query_response(&self) -> Option<KittyQueryResponse> {
        if self.action != Some("q") {
            return None;
        }

        let quiet = self.quiet();
        let image_id = self.parsed_image_id();
        let placement_id = self.placement_id_u32();
        let image_number = self.image_number();
        let transport = parse_kitty_transmission_medium(self.medium);

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
            let payload = match self
                .read_payload()
                .and_then(|payload| apply_kitty_transport_compression(payload, self.compression))
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
            let format = self.format.unwrap_or("32");
            let valid = match format {
                "24" => {
                    let width = match self.width.and_then(|v| v.parse::<usize>().ok()) {
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
                    let height = match self.height.and_then(|v| v.parse::<usize>().ok()) {
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
                    let width = match self.width.and_then(|v| v.parse::<usize>().ok()) {
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
                    let height = match self.height.and_then(|v| v.parse::<usize>().ok()) {
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

    pub fn delete_request(&self) -> Option<KittyDeleteRequest> {
        if self.action != Some("d") {
            return None;
        }
        let delete_selector = self.delete_selector.unwrap_or("a");
        let mode = match delete_selector {
            "A" | "I" | "N" | "C" | "P" | "Q" | "R" | "X" | "Y" | "Z" => {
                KittyDeleteMode::PlacementsAndBackingData
            },
            "a" | "i" | "n" | "c" | "p" | "q" | "r" | "x" | "y" | "z" => {
                KittyDeleteMode::PlacementsOnly
            },
            _ => return None,
        };
        let placement_id = self.placement_id();
        let protocol_cell_coord =
            |value: Option<&str>| -> Option<u32> { value?.parse::<u32>().ok()?.checked_sub(1) };
        let selector = match delete_selector {
            "a" | "A" => KittyDeleteSelector::AllVisible,
            "i" | "I" => KittyDeleteSelector::ImageId {
                image_id: self.parsed_image_id()?,
                placement_id,
            },
            "n" | "N" => KittyDeleteSelector::ImageNumber {
                image_number: self.image_number()?,
                placement_id,
            },
            "c" | "C" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Cursor),
            "p" | "P" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                x: protocol_cell_coord(self.source_x)?,
                y: protocol_cell_coord(self.source_y)?,
                z: None,
            }),
            "q" | "Q" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                x: protocol_cell_coord(self.source_x)?,
                y: protocol_cell_coord(self.source_y)?,
                z: Some(self.z_index?.parse::<i32>().ok()?),
            }),
            "x" | "X" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Column {
                x: protocol_cell_coord(self.source_x)?,
            }),
            "y" | "Y" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Row {
                y: protocol_cell_coord(self.source_y)?,
            }),
            "z" | "Z" => KittyDeleteSelector::Geometry(KittyGeometrySelector::Z {
                z: self.z_index?.parse::<i32>().ok()?,
            }),
            "r" | "R" => KittyDeleteSelector::Range {
                first_image_id: self.source_x?.parse::<u32>().ok()?,
                last_image_id: self.source_y?.parse::<u32>().ok()?,
            },
            _ => return None,
        };
        Some(KittyDeleteRequest { selector, mode })
    }

    fn placement(&self) -> KittyPlacement {
        KittyPlacement {
            image_id: 0,
            placement_id: self.placement_id(),
            relative_to: self
                .parent_image_id
                .and_then(|parent_image_id| parent_image_id.parse::<u32>().ok())
                .map(|parent_image_id| KittyRelativePlacement {
                    parent_image_id,
                    parent_placement_id: self
                        .parent_placement_id
                        .and_then(|placement_id| placement_id.parse::<u32>().ok())
                        .filter(|placement_id| *placement_id != 0)
                        .map(PlacementId::Protocol),
                    offset_x: self
                        .parent_offset_x
                        .and_then(|offset| offset.parse::<i32>().ok())
                        .unwrap_or(0),
                    offset_y: self
                        .parent_offset_y
                        .and_then(|offset| offset.parse::<i32>().ok())
                        .unwrap_or(0),
                }),
            placement_mode: if self.placement_mode == Some("1") {
                KittyImagePlacementMode::Placeholder
            } else {
                KittyImagePlacementMode::Explicit
            },
            source_x: self.source_x.and_then(|v| v.parse::<u32>().ok()),
            source_y: self.source_y.and_then(|v| v.parse::<u32>().ok()),
            source_width: self.source_width.and_then(|v| v.parse::<u32>().ok()),
            source_height: self.source_height.and_then(|v| v.parse::<u32>().ok()),
            columns: self.columns.and_then(|v| v.parse::<u32>().ok()),
            rows: self.rows.and_then(|v| v.parse::<u32>().ok()),
            x_offset: self.x_offset.and_then(|v| v.parse::<u32>().ok()),
            y_offset: self.y_offset.and_then(|v| v.parse::<u32>().ok()),
            z_index: self.z_index.and_then(|v| v.parse::<i32>().ok()),
            cursor_movement_policy: match self.cursor_policy {
                Some("1") => KittyCursorMovementPolicy::NoMovement,
                _ => KittyCursorMovementPolicy::AfterPlacement,
            },
            ..Default::default()
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
        media_source: KittyMediaSource,
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
        let source_x = self.source_x.unwrap_or(0).min(image_width);
        let source_y = self.source_y.unwrap_or(0).min(image_height);
        let source_width = self
            .source_width
            .unwrap_or_else(|| image_width.saturating_sub(source_x))
            .min(image_width.saturating_sub(source_x));
        let source_height = self
            .source_height
            .unwrap_or_else(|| image_height.saturating_sub(source_y))
            .min(image_height.saturating_sub(source_y));
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
            columns_specified: self.columns.is_some(),
            rows_specified: self.rows.is_some(),
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
    fn validate_current_direct_payload_size(&self) -> Result<(), String> {
        let KittyMediaSource::Direct(payload) = &self.media_source else {
            return Ok(());
        };
        self.validate_direct_payload_size(payload.len())
    }

    fn append_direct_payload(&mut self, payload: Vec<u8>) -> Result<(), String> {
        let limit = self.direct_payload_limit()?;
        match &mut self.media_source {
            KittyMediaSource::Direct(existing_payload) => {
                let new_len = existing_payload
                    .len()
                    .checked_add(payload.len())
                    .ok_or_else(kitty_payload_too_large_message)?;
                if new_len > limit {
                    return Err(kitty_payload_too_large_message());
                }
                existing_payload.extend(payload);
                Ok(())
            },
            KittyMediaSource::External(_) => {
                Err(non_query_failure_message(self.reply_context.kind).to_string())
            },
        }
    }

    fn validate_direct_payload_size(&self, payload_len: usize) -> Result<(), String> {
        let limit = self.direct_payload_limit()?;
        if payload_len > limit {
            Err(kitty_payload_too_large_message())
        } else {
            Ok(())
        }
    }

    fn direct_payload_limit(&self) -> Result<usize, String> {
        match self.image_format {
            KittyImageFormat::Png => Ok(KITTY_MAX_PENDING_DIRECT_TRANSMIT_BYTES),
            KittyImageFormat::Rgb => self.raw_direct_payload_limit(3),
            KittyImageFormat::Rgba => self.raw_direct_payload_limit(4),
        }
    }

    fn raw_direct_payload_limit(&self, bytes_per_pixel: usize) -> Result<usize, String> {
        let expected_len =
            expected_nonzero_raw_payload_size(self.width, self.height, bytes_per_pixel)?;
        if self.compression.is_some() {
            Ok(expected_len
                .saturating_add(KITTY_COMPRESSED_RAW_DIRECT_TRANSMIT_PADDING_BYTES)
                .min(KITTY_MAX_PENDING_DIRECT_TRANSMIT_BYTES))
        } else {
            Ok(expected_len)
        }
    }

    fn into_asset_data(self) -> Result<KittyAssetData, String> {
        if self.compression.is_none() {
            let image_format = self.image_format;
            if matches!(image_format, KittyImageFormat::Rgb | KittyImageFormat::Rgba) {
                if let Some(media) = self.media_source.external_media() {
                    let bytes_per_pixel = match image_format {
                        KittyImageFormat::Rgb => 3,
                        KittyImageFormat::Rgba => 4,
                        KittyImageFormat::Png => unreachable!(),
                    };
                    let expected_len = expected_nonzero_raw_payload_size(
                        self.width,
                        self.height,
                        bytes_per_pixel,
                    )?;
                    match media.payload_len() {
                        Some(payload_len) if payload_len == expected_len => {
                            let format = match image_format {
                                KittyImageFormat::Rgb => KittyAssetFormat::Rgb,
                                KittyImageFormat::Rgba => KittyAssetFormat::Rgba,
                                KittyImageFormat::Png => unreachable!(),
                            };
                            return Ok(KittyAssetData::External {
                                media,
                                format,
                                width: self.width,
                                height: self.height,
                            });
                        },
                        Some(payload_len) if payload_len < expected_len => {
                            return Err(format!(
                                "ENODATA:Insufficient image data: {payload_len} < {expected_len}"
                            ));
                        },
                        Some(_) => {
                            return Err(
                                "EINVAL:Invalid image payload for requested format".to_string()
                            );
                        },
                        None => {
                            return Err("EINVAL:Invalid or unsupported kitty command".to_string());
                        },
                    }
                }
            }
        }
        self.into_image_data().map(KittyAssetData::Image)
    }

    fn into_image_data(self) -> Result<KittyImageData, String> {
        let payload = match self.compression {
            Some(KittyTransportCompression::Zlib) => {
                decompress_to_vec_zlib(&self.media_source.into_payload()?)
                    .map_err(|_| "EINVAL:Invalid image payload encoding".to_string())?
            },
            None => self.media_source.into_payload()?,
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
    let expected_len = expected_nonzero_raw_payload_size(width, height, bytes_per_pixel)?;
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

fn expected_nonzero_raw_payload_size(
    width: u32,
    height: u32,
    bytes_per_pixel: usize,
) -> Result<usize, String> {
    if width == 0 || height == 0 {
        return Err("EINVAL:Zero width/height not allowed".to_string());
    }
    expected_raw_payload_size(width, height, bytes_per_pixel)
}

fn expected_raw_payload_size(
    width: u32,
    height: u32,
    bytes_per_pixel: usize,
) -> Result<usize, String> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixel_count| pixel_count.checked_mul(bytes_per_pixel))
        .ok_or_else(|| "EINVAL:Invalid image payload for requested format".to_string())
}

fn kitty_payload_too_large_message() -> String {
    "EFBIG:Too much data".to_string()
}

fn kitty_asset_id_space_exhausted_message() -> String {
    "ENOMEM:No available kitty image ids".to_string()
}

fn kitty_image_dimensions(image_data: &KittyImageData) -> (u32, u32) {
    match image_data {
        KittyImageData::Png { width, height, .. }
        | KittyImageData::Rgb { width, height, .. }
        | KittyImageData::Rgba { width, height, .. } => (*width, *height),
    }
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
        Some("s") => Some(KittyTransmissionMedium::SharedMemory),
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
    KittyExternalMedia::new(
        KittyExternalMediaLocation::RegularFile(kitty_path_payload(path_payload)?.to_path_buf()),
        KittyByteRange { offset, size },
    )
    .into_payload()
}

fn read_kitty_temporary_file_payload(
    path_payload: &[u8],
    size: Option<usize>,
    offset: u64,
) -> Option<Vec<u8>> {
    KittyExternalMedia::new(
        KittyExternalMediaLocation::TemporaryFile(kitty_path_payload(path_payload)?.to_path_buf()),
        KittyByteRange { offset, size },
    )
    .into_payload()
}

fn read_kitty_shared_memory_payload(
    name_payload: &[u8],
    size: Option<usize>,
    offset: u64,
) -> Option<Vec<u8>> {
    KittyExternalMedia::new(
        KittyExternalMediaLocation::SharedMemory(
            std::str::from_utf8(name_payload).ok()?.to_string(),
        ),
        KittyByteRange { offset, size },
    )
    .into_payload()
}

fn read_kitty_transmission_payload(
    payload_b64: &[u8],
    medium: Option<&str>,
    size: Option<&str>,
    offset: Option<&str>,
    inferred_raw_payload_size: Option<usize>,
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
        KittyTransmissionMedium::SharedMemory => {
            let (size, offset) = parse_kitty_payload_byte_range(size, offset)?;
            read_kitty_shared_memory_payload(&payload, size.or(inferred_raw_payload_size), offset)
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
    KittyApc::parse(apc_bytes)?.delete_request()
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
    KittyApc::parse(apc_bytes)?.query_response()
}

pub fn kitty_non_query_response(
    apc_bytes: &[u8],
    command_succeeded: bool,
    resolved_image_id: Option<u32>,
    resolved_image_number: Option<u32>,
) -> Option<KittyQueryResponse> {
    let reply_context = KittyApc::parse(apc_bytes)?.reply_context()?;
    let error =
        (!command_succeeded).then(|| non_query_failure_message(reply_context.kind).to_string());
    build_non_query_reply(
        &reply_context,
        resolved_image_id.or(reply_context.parsed_image_id),
        resolved_image_number.or(reply_context.image_number),
        error,
    )
}

pub fn kitty_terminal_image_response(apc_bytes: &[u8]) -> Option<KittyTerminalImageResponse> {
    let apc = KittyApc::parse(apc_bytes)?;
    let message = String::from_utf8_lossy(apc.payload).to_string();
    if message.is_empty() {
        return None;
    }
    Some(KittyTerminalImageResponse {
        image_id: apc.parsed_image_id(),
        placement_id: apc.placement_id_u32(),
        image_number: apc.image_number(),
        is_ok: message == "OK",
        message,
    })
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
    fn parse(apc: &KittyApc<'_>) -> Option<Self> {
        let more = apc.more();
        let transmission_medium = parse_kitty_transmission_medium(apc.medium)?;
        if more && transmission_medium != KittyTransmissionMedium::Direct {
            return None;
        }

        if let Some(action) = apc.action {
            let placement = apc.placement();
            match action {
                "T" | "t" => {
                    let media_source = KittyMediaSource::from_apc(apc, transmission_medium)?;
                    let image_format = match apc.format.unwrap_or("32") {
                        "100" => KittyImageFormat::Png,
                        "24" => KittyImageFormat::Rgb,
                        "32" => KittyImageFormat::Rgba,
                        _ => return None,
                    };
                    let compression = parse_kitty_transport_compression(apc.compression)?;
                    let protocol_image_id = apc.parsed_image_id();
                    let image_number = apc.image_number();
                    let width = apc.width.and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                    let height = apc.height.and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
                    let should_create_placement = action == "T"
                        || apc.placement_id.is_some()
                        || apc.columns.is_some()
                        || apc.rows.is_some()
                        || apc.source_x.is_some()
                        || apc.source_y.is_some()
                        || apc.source_width.is_some()
                        || apc.source_height.is_some()
                        || apc.x_offset.is_some()
                        || apc.y_offset.is_some()
                        || apc.z_index.is_some()
                        || apc.placement_mode == Some("1");
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
                        media_source,
                    })
                },
                "p" => {
                    let protocol_image_id = apc.parsed_image_id();
                    let image_number = apc.image_number();
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
            let payload = match transmission_medium {
                KittyTransmissionMedium::Direct => decode_kitty_transport_payload(apc.payload)?,
                KittyTransmissionMedium::RegularFile
                | KittyTransmissionMedium::TemporaryFile
                | KittyTransmissionMedium::SharedMemory => return None,
            };
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

fn serialize_transmit_file(
    image_id: u32,
    format: KittyAssetFormat,
    width: u32,
    height: u32,
    path: &Path,
    quiet: u8,
) -> String {
    let mut command = String::new();
    let _ = write!(command, "a=t,i={image_id},q={quiet},");
    match format {
        KittyAssetFormat::Png => {
            command.push_str("f=100,");
        },
        KittyAssetFormat::Rgb => {
            let _ = write!(command, "f=24,s={width},v={height},");
        },
        KittyAssetFormat::Rgba => {
            let _ = write!(command, "f=32,s={width},v={height},");
        },
    };
    let payload = base64::encode(path.to_string_lossy().as_bytes());
    command.push_str("t=f;");
    command.push_str(&payload);
    command
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
            "tty-graphics-protocol-zellij-{name}-{}-{unique}.bin",
            std::process::id()
        ));
        std::fs::write(&path, bytes).expect("fixture file should be writable");
        path
    }

    fn write_kitty_safe_temp_dir_media_fixture(name: &str, bytes: &[u8]) -> (PathBuf, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "tty-graphics-protocol-zellij-{name}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).expect("fixture directory should be writable");
        let path = dir.join("payload.bin");
        std::fs::write(&path, bytes).expect("fixture file should be writable");
        (dir, path)
    }

    fn write_kitty_nontemp_magic_media_fixture(name: &str, bytes: &[u8]) -> (PathBuf, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let dir = std::env::current_dir()
            .expect("current dir should be available")
            .join("target")
            .join(format!(
                "zellij-kitty-file-media-{name}-{}-{unique}",
                std::process::id()
            ));
        std::fs::create_dir_all(&dir).expect("fixture directory should be writable");
        let path = dir.join("payload-tty-graphics-protocol.bin");
        std::fs::write(&path, bytes).expect("fixture file should be writable");
        (dir, path)
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

    fn kitty_shared_memory_transmit_apc(
        image_id: u32,
        name: &str,
        image_format: u32,
        size: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<u8> {
        let mut control = format!("Gq=0,a=t,t=s,f={image_format},i={image_id}");
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
        control.push_str(&base64::encode(name.as_bytes()));
        control.into_bytes()
    }

    fn kitty_shared_memory_query_apc(
        image_id: u32,
        name: &str,
        image_format: u32,
        size: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<u8> {
        let mut control = format!("Gq=0,a=q,t=s,f={image_format},i={image_id}");
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
        control.push_str(&base64::encode(name.as_bytes()));
        control.into_bytes()
    }

    #[cfg(unix)]
    fn unique_kitty_shared_memory_name(label: &str) -> String {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
            % 0xfffff;
        format!(
            "/zj{:x}{:x}{}",
            std::process::id() % 0xffff,
            unique,
            label.chars().next().unwrap_or('x')
        )
    }

    #[cfg(unix)]
    fn create_kitty_shared_memory_fixture(label: &str, bytes: &[u8]) -> String {
        use std::ffi::CString;

        let name = unique_kitty_shared_memory_name(label);
        let c_name = CString::new(name.clone()).expect("shm name should not contain nul");
        unsafe {
            libc::shm_unlink(c_name.as_ptr());
            let fd = libc::shm_open(
                c_name.as_ptr(),
                libc::O_CREAT | libc::O_EXCL | libc::O_RDWR,
                0o600,
            );
            assert!(
                fd >= 0,
                "shm_open fixture failed for {name}: {}",
                std::io::Error::last_os_error()
            );
            assert_eq!(
                libc::ftruncate(fd, bytes.len() as libc::off_t),
                0,
                "ftruncate fixture shm failed for {name}"
            );
            let mapping = libc::mmap(
                std::ptr::null_mut(),
                bytes.len(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );
            assert_ne!(
                mapping,
                libc::MAP_FAILED,
                "mmap fixture shm failed for {name}"
            );
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapping as *mut u8, bytes.len());
            libc::msync(mapping, bytes.len(), libc::MS_SYNC);
            libc::munmap(mapping, bytes.len());
            libc::close(fd);
        }
        name
    }

    #[cfg(unix)]
    fn kitty_shared_memory_exists(name: &str) -> bool {
        use std::ffi::CString;

        let c_name = CString::new(name).expect("shm name should not contain nul");
        unsafe {
            let fd = libc::shm_open(c_name.as_ptr(), libc::O_RDONLY, 0o600);
            if fd < 0 {
                false
            } else {
                libc::close(fd);
                true
            }
        }
    }

    #[cfg(unix)]
    fn unlink_kitty_shared_memory(name: &str) {
        use std::ffi::CString;

        let c_name = CString::new(name).expect("shm name should not contain nul");
        unsafe {
            libc::shm_unlink(c_name.as_ptr());
        }
    }

    #[cfg(unix)]
    fn platform_page_size() -> usize {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE).max(1) as usize }
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
            .borrow_mut()
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

        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("OK"),
            "regular file upload should succeed, got {reply:?}"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 601, &payload);
        std::fs::remove_file(path).ok();
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

        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("OK"),
            "regular file upload with unaligned O should succeed, got {reply:?}"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 602, &payload);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn kitty_temporary_file_rgba_upload_deletes_safe_temp_file_after_materialization() {
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
            path.exists(),
            "safe temporary file should not be read or deleted until materialization"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 605, &payload);
        assert!(
            !path.exists(),
            "safe temporary file should be deleted after materialization"
        );
    }

    #[test]
    fn kitty_temporary_file_rgba_upload_deletes_when_magic_is_in_temp_directory_name() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let (dir, path) = write_kitty_safe_temp_dir_media_fixture("directory-marker", &payload);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_file_media_transmit_apc("t", 619, 32, &path, None, None),
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
            "temporary file upload with magic directory should succeed, got {reply:?}"
        );
        assert!(
            path.exists(),
            "safe temporary file should not be read or deleted until materialization"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 619, &payload);
        assert!(
            !path.exists(),
            "safe temporary file should be deleted when the marker is in the full temp path"
        );
        std::fs::remove_dir(dir).ok();
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
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 606, &payload);
        assert!(path.exists(), "unsafe temporary file name should be kept");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn kitty_temporary_file_rgba_upload_keeps_magic_path_outside_temp_dirs() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let (dir, path) = write_kitty_nontemp_magic_media_fixture("nontemp-marker", &payload);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_file_media_transmit_apc("t", 620, 32, &path, None, None),
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
            "temporary file upload outside temp dirs should still read, got {reply:?}"
        );
        assert!(
            path.exists(),
            "temporary file outside known temp dirs should not be deleted before materialization"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 620, &payload);
        assert!(
            path.exists(),
            "temporary file outside known temp dirs should not be deleted even with the marker"
        );
        std::fs::remove_dir_all(dir).ok();
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
            path.exists(),
            "safe temporary file with S/O should not be deleted until materialization"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 607, &payload);
        assert!(
            !path.exists(),
            "safe temporary file with S/O should be deleted after materialization"
        );
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

    #[cfg(unix)]
    #[test]
    fn kitty_shared_memory_rgba_upload_unlinks_after_materialization() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let name = create_kitty_shared_memory_fixture("whole", &payload);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_shared_memory_transmit_apc(611, &name, 32, None, None),
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
            "shared memory upload should succeed, got {reply:?}"
        );
        assert!(
            kitty_shared_memory_exists(&name),
            "shared memory object should not be read or unlinked until materialization"
        );
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 611, &payload);
        assert!(
            !kitty_shared_memory_exists(&name),
            "shared memory object should be unlinked after materialization"
        );
    }

    #[cfg(unix)]
    #[test]
    fn kitty_shared_memory_rgba_upload_honors_zero_unaligned_and_page_offsets() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let cases = [
            (612, 0usize, "zero"),
            (613, 3usize, "unaligned"),
            (614, platform_page_size(), "page"),
        ];
        for (image_id, offset, label) in cases {
            let mut shm_bytes = vec![0xAA; offset];
            shm_bytes.extend_from_slice(&payload);
            shm_bytes.extend_from_slice(&[0xDD, 0xEE]);
            let name = create_kitty_shared_memory_fixture(label, &shm_bytes);
            let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
            let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

            let reply = kitty_state.handle_apc(
                &kitty_shared_memory_transmit_apc(
                    image_id,
                    &name,
                    32,
                    Some(payload.len()),
                    Some(offset),
                ),
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
                "shared memory upload with offset {offset} should succeed, got {reply:?}"
            );
            assert!(
                kitty_shared_memory_exists(&name),
                "shared memory object should not be unlinked before materialization"
            );
            assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, image_id, &payload);
            assert!(
                !kitty_shared_memory_exists(&name),
                "shared memory object should be unlinked after materialization"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn kitty_shared_memory_invalid_png_unlinks_after_read() {
        let name = create_kitty_shared_memory_fixture("badpng", b"not a png");
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store);

        let reply = kitty_state.handle_apc(
            &kitty_shared_memory_transmit_apc(615, &name, 100, None, None),
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
            "invalid shared-memory PNG should fail after read, got {reply:?}"
        );
        assert!(
            !kitty_shared_memory_exists(&name),
            "invalid shared-memory payload should still be unlinked after read"
        );
    }

    #[cfg(unix)]
    #[test]
    fn kitty_shared_memory_missing_returns_error_without_creating_asset() {
        let name = unique_kitty_shared_memory_name("missing");
        unlink_kitty_shared_memory(&name);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_shared_memory_transmit_apc(616, &name, 32, None, None),
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
            reply.contains("EINVAL:Invalid or unsupported kitty command"),
            "missing shared memory should fail, got {reply:?}"
        );
        assert!(
            !kitty_state
                .protocol_image_id_to_internal_id
                .contains_key(&616),
            "missing shared memory should not create a protocol image mapping"
        );
        assert!(kitty_asset_store.borrow_mut().image_data(616).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn kitty_query_response_accepts_shared_memory_and_unlinks() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let name = create_kitty_shared_memory_fixture("query", &payload);
        let query = kitty_shared_memory_query_apc(617, &name, 32, None, None);

        let reply = kitty_query_response(&query).unwrap().to_apc_response();

        assert_eq!(reply, "\u{1b}_Gi=617;OK\u{1b}\\");
        assert!(
            !kitty_shared_memory_exists(&name),
            "query should unlink shared memory object after read"
        );
    }

    #[cfg(unix)]
    #[test]
    fn kitty_query_response_invalid_shared_memory_png_unlinks_after_read() {
        let name = create_kitty_shared_memory_fixture("qbadpng", b"not a png");
        let query = kitty_shared_memory_query_apc(618, &name, 100, None, None);

        let reply = kitty_query_response(&query).unwrap().to_apc_response();

        assert!(
            reply.contains("i=618;EINVAL:Invalid image payload for requested format"),
            "invalid shared-memory PNG query should fail after read, got {reply:?}"
        );
        assert!(
            !kitty_shared_memory_exists(&name),
            "invalid shared-memory query should unlink after read"
        );
    }

    #[test]
    fn kitty_parser_rejects_chunked_external_media() {
        let path = write_kitty_file_media_fixture("chunked-external", b"not image bytes");
        let mut command = String::from("Gq=0,a=t,t=f,m=1,f=32,s=1,v=1,i=619;");
        command.push_str(&base64::encode(path.to_string_lossy().as_bytes()));

        assert!(
            ParsedKittyCommand::parse(&KittyApc::parse(command.as_bytes()).unwrap()).is_none(),
            "chunked external media should be rejected before reading the external object"
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn kitty_parser_defers_regular_file_reads_until_materialization() {
        let path = std::env::temp_dir().join(format!(
            "zellij-kitty-missing-file-media-{}",
            std::process::id()
        ));
        std::fs::remove_file(&path).ok();
        let apc = kitty_regular_file_transmit_apc(620, &path, None, None);

        assert!(
            matches!(
                ParsedKittyCommand::parse(&KittyApc::parse(&apc).unwrap()),
                Some(ParsedKittyCommand::ImmediateTransmit {
                    protocol_image_id: Some(620),
                    ..
                })
            ),
            "regular-file transmit parsing should not read the referenced file"
        );
    }

    #[test]
    fn kitty_regular_file_rgba_upload_caches_after_first_materialization() {
        let payload = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = write_kitty_file_media_fixture("file-backed", &payload);
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_regular_file_transmit_apc(622, &path, None, None),
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
            "regular file upload should succeed, got {reply:?}"
        );
        let internal_image_id = *kitty_state
            .protocol_image_id_to_internal_id
            .get(&622)
            .expect("protocol id should be mapped to an internal asset");
        assert_eq!(
            kitty_asset_store
                .borrow()
                .image_dimensions(internal_image_id),
            Some((2, 2))
        );

        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 622, &payload);
        std::fs::remove_file(path).ok();
        assert_stored_rgba_payload(&kitty_state, &kitty_asset_store, 622, &payload);
    }

    #[test]
    fn kitty_missing_regular_file_does_not_register_protocol_mapping() {
        let path = std::env::temp_dir().join(format!(
            "zellij-kitty-missing-file-media-{}",
            std::process::id()
        ));
        std::fs::remove_file(&path).ok();
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store.clone());

        let reply = kitty_state.handle_apc(
            &kitty_regular_file_transmit_apc(621, &path, None, None),
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
            reply.contains("EINVAL:Invalid or unsupported kitty command"),
            "missing regular file should preserve the previous parse-failure reply, got {reply:?}"
        );
        assert!(
            !kitty_state
                .protocol_image_id_to_internal_id
                .contains_key(&621),
            "failed materialization should not leave a protocol image mapping"
        );
        assert!(kitty_asset_store.borrow_mut().image_data(621).is_none());
    }

    #[test]
    fn interrupted_image_number_transmit_removes_orphan_synthetic_mapping() {
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store);
        let anchor = FlowAnchor::LogicalRow {
            logical_row: 0,
            column: 0,
        };

        let first_reply = kitty_state.handle_apc(
            b"Gq=0,a=t,f=24,s=1,v=1,I=77,m=1;EjRW",
            anchor.clone(),
            0,
            0,
            None,
        );
        assert!(first_reply.reply.is_none());
        assert!(
            kitty_state.protocol_image_id_for_image_number(77).is_none(),
            "chunked image-number upload should not publish a protocol id until it succeeds"
        );

        let second_reply =
            kitty_state.handle_apc(b"Gq=0,a=t,f=24,s=1,v=1,I=77;EjRW", anchor, 0, 0, None);
        assert!(matches!(
            second_reply.reply,
            Some(KittyQueryResponse::Ok { .. })
        ));
        let second_protocol_id = kitty_state
            .protocol_image_id_for_image_number(77)
            .expect("completed image-number upload should keep its protocol id");

        assert_eq!(
            kitty_state
                .image_number_to_protocol_image_ids
                .get(&77)
                .map(Vec::as_slice),
            Some(&[second_protocol_id][..])
        );
    }

    #[test]
    fn interrupted_same_id_transmit_keeps_completed_replacement_mapping() {
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store);
        let anchor = FlowAnchor::LogicalRow {
            logical_row: 0,
            column: 0,
        };

        let first_reply = kitty_state.handle_apc(
            b"Gq=0,a=t,f=24,s=1,v=1,i=622,m=1;EjRW",
            anchor.clone(),
            0,
            0,
            None,
        );
        assert!(first_reply.reply.is_none());

        let second_reply =
            kitty_state.handle_apc(b"Gq=0,a=t,f=24,s=1,v=1,i=622;EjRW", anchor, 0, 0, None);
        assert!(matches!(
            second_reply.reply,
            Some(KittyQueryResponse::Ok { .. })
        ));
        assert!(
            kitty_state.protocol_image_id_to_internal_id.contains_key(&622),
            "completed upload should keep the protocol mapping after replacing an abandoned pending upload with the same id"
        );
    }

    #[test]
    fn chunked_raw_upload_with_zero_dimensions_is_rejected_without_pending_transmit() {
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store);
        let anchor = FlowAnchor::LogicalRow {
            logical_row: 0,
            column: 0,
        };

        let reply = kitty_state.handle_apc(b"Gq=0,a=t,f=24,i=623,m=1;AA==", anchor, 0, 0, None);

        let reply = reply.reply.unwrap().to_apc_response();
        assert!(
            reply.contains("EINVAL:Zero width/height not allowed"),
            "zero-sized raw chunked upload should fail immediately, got {reply:?}"
        );
        assert!(
            kitty_state.pending_transmit.is_none(),
            "failed zero-sized upload should not leave a pending transmit"
        );
        assert!(
            !kitty_state
                .protocol_image_id_to_internal_id
                .contains_key(&623),
            "failed zero-sized upload should not leave a protocol mapping"
        );
    }

    #[test]
    fn chunked_raw_upload_rejects_payload_beyond_declared_size_and_clears_pending() {
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store);
        let anchor = FlowAnchor::LogicalRow {
            logical_row: 0,
            column: 0,
        };

        let first_reply = kitty_state.handle_apc(
            b"Gq=0,a=t,f=24,s=1,v=1,i=624,m=1;AAA=",
            anchor.clone(),
            0,
            0,
            None,
        );
        assert!(first_reply.reply.is_none());
        assert!(
            kitty_state.pending_transmit.is_some(),
            "opening chunk should remain pending while within the declared raw size"
        );

        let overflow_reply = kitty_state.handle_apc(b"Gq=0,m=1;AAA=", anchor.clone(), 0, 0, None);
        let overflow_reply = overflow_reply.reply.unwrap().to_apc_response();
        assert!(
            overflow_reply.contains("EFBIG:Too much data"),
            "oversized chunked upload should report EFBIG, got {overflow_reply:?}"
        );
        assert!(
            kitty_state.pending_transmit.is_none(),
            "oversized chunked upload should be dropped immediately"
        );
        assert!(
            !kitty_state
                .protocol_image_id_to_internal_id
                .contains_key(&624),
            "oversized chunked upload should not leave a protocol mapping"
        );

        let stale_final_reply = kitty_state.handle_apc(b"Gq=0,m=0;AA==", anchor, 0, 0, None);
        assert!(
            stale_final_reply.reply.is_none(),
            "final chunk after an overflowed upload should be ignored"
        );
    }

    #[test]
    fn pending_png_direct_upload_has_a_hard_payload_cap() {
        let pending = PendingKittyTransmit {
            protocol_image_id: Some(625),
            image_number: None,
            image_id: Some(1),
            image_format: KittyImageFormat::Png,
            compression: None,
            width: 0,
            height: 0,
            placement: None,
            reply_context: PendingKittyReplyContext {
                kind: PendingKittyReplyKind::Transmit,
                quiet: 0,
                parsed_image_id: Some(625),
                placement_id: None,
                image_number: None,
            },
            media_source: KittyMediaSource::Direct(Vec::new()),
        };

        assert!(
            pending
                .validate_direct_payload_size(KITTY_MAX_PENDING_DIRECT_TRANSMIT_BYTES)
                .is_ok(),
            "PNG direct upload should accept payloads at the hard cap"
        );
        assert_eq!(
            pending
                .validate_direct_payload_size(KITTY_MAX_PENDING_DIRECT_TRANSMIT_BYTES + 1)
                .unwrap_err(),
            "EFBIG:Too much data"
        );
    }

    #[test]
    fn failed_new_upload_does_not_consume_internal_asset_id() {
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::default()));
        let mut kitty_state = KittyImageState::new(kitty_asset_store);
        let anchor = FlowAnchor::LogicalRow {
            logical_row: 0,
            column: 0,
        };

        let failed_reply =
            kitty_state.handle_apc(b"Gq=0,a=t,f=24,i=626,m=1;AA==", anchor.clone(), 0, 0, None);
        assert!(failed_reply
            .reply
            .unwrap()
            .to_apc_response()
            .contains("EINVAL:Zero width/height not allowed"));

        let successful_reply =
            kitty_state.handle_apc(b"Gq=0,a=t,f=24,s=1,v=1,i=627;AAAA", anchor, 0, 0, None);
        assert!(matches!(
            successful_reply.reply,
            Some(KittyQueryResponse::Ok { .. })
        ));
        assert_eq!(
            kitty_state.protocol_image_id_to_internal_id.get(&627),
            Some(&1),
            "failed new uploads should not advance the internal asset id allocator"
        );
        assert!(
            !kitty_state
                .protocol_image_id_to_internal_id
                .contains_key(&626),
            "failed new uploads should not leave a protocol mapping"
        );
    }

    #[test]
    fn parsed_kitty_apc_matches_query_delete_and_transmit_parsing() {
        let query = KittyApc::parse(b"Gq=0,a=q,t=d,f=24,s=1,v=1,i=41;EjRW").unwrap();
        assert_eq!(
            query.query_response().map(|reply| reply.to_apc_response()),
            kitty_query_response(b"Gq=0,a=q,t=d,f=24,s=1,v=1,i=41;EjRW")
                .map(|reply| reply.to_apc_response())
        );

        let delete = KittyApc::parse(b"Ga=d,d=I,i=51,p=7").unwrap();
        assert_eq!(
            delete.delete_request(),
            kitty_delete_request(b"Ga=d,d=I,i=51,p=7")
        );

        let transmit = KittyApc::parse(b"Ga=t,f=24,s=1,v=1,i=7;EjRW").unwrap();
        assert!(matches!(
            ParsedKittyCommand::parse(&transmit),
            Some(ParsedKittyCommand::ImmediateTransmit {
                protocol_image_id: Some(7),
                image_format: KittyImageFormat::Rgb,
                width: 1,
                height: 1,
                media_source: KittyMediaSource::Direct(ref payload),
                ..
            }) if payload == &[0x12, 0x34, 0x56]
        ));
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
                .borrow_mut()
                .image_data(anchor_internal_image_id)
                .is_some(),
            "visible anchor image should be protected from quota eviction"
        );
        assert!(
            kitty_asset_store
                .borrow_mut()
                .image_data(first_stored_internal_image_id)
                .is_none(),
            "oldest unplaced image should be evicted under quota pressure"
        );
        assert!(
            !kitty_state
                .internal_image_id_to_protocol_image_ids
                .contains_key(&first_stored_internal_image_id),
            "reverse protocol-image index should forget evicted assets"
        );
        let second_stored_internal_image_id = *kitty_state
            .protocol_image_id_to_internal_id
            .get(&133)
            .expect("second stored image should have an internal image id");
        assert!(
            kitty_asset_store
                .borrow_mut()
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
    fn local_quota_eviction_keeps_assets_visible_in_other_panes() {
        let kitty_asset_store = Rc::new(RefCell::new(KittyAssetStore::with_decoded_byte_quota(8)));
        let mut first_pane_kitty_state = KittyImageState::new(kitty_asset_store.clone());
        let mut second_pane_kitty_state = KittyImageState::new(kitty_asset_store.clone());
        let anchor = FlowAnchor::LogicalRow {
            logical_row: 0,
            column: 0,
        };
        let cell_size = Some(SizeInPixels {
            width: 1,
            height: 1,
        });

        let visible_reply = first_pane_kitty_state.handle_apc(
            b"Gq=0,a=T,C=1,f=24,s=1,v=1,i=141,p=1,c=1,r=1;EjRW",
            anchor.clone(),
            0,
            0,
            cell_size,
        );
        assert!(
            visible_reply
                .reply
                .unwrap()
                .to_apc_response()
                .contains("OK"),
            "first pane visible placement should succeed"
        );
        let visible_internal_image_id = *first_pane_kitty_state
            .protocol_image_id_to_internal_id
            .get(&141)
            .expect("visible image should have an internal id");

        let stored_first_reply = second_pane_kitty_state.handle_apc(
            b"Gq=0,a=t,f=24,s=1,v=1,i=142;EjRW",
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
            "second pane first stored image should succeed"
        );

        let stored_second_reply = second_pane_kitty_state.handle_apc(
            b"Gq=0,a=t,f=24,s=1,v=1,i=143;EjRW",
            anchor,
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
            "second pane second stored image should succeed"
        );

        assert!(
            kitty_asset_store
                .borrow_mut()
                .image_data(visible_internal_image_id)
                .is_some(),
            "quota eviction in one pane must not remove another pane's visible asset"
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
                    x: 23,
                    y: 10,
                    z: None,
                }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=P,x=24,y=11",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 23,
                    y: 10,
                    z: None,
                }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=q,x=24,y=11,z=-1",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 23,
                    y: 10,
                    z: Some(-1),
                }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=Q,x=24,y=11,z=-1",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 23,
                    y: 10,
                    z: Some(-1),
                }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=x,x=8",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Column { x: 7 }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=X,x=8",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Column { x: 7 }),
                mode: KittyDeleteMode::PlacementsAndBackingData,
            },
        );
        assert_delete_request(
            b"Ga=d,d=y,y=11",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Row { y: 10 }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_delete_request(
            b"Ga=d,d=Y,y=11",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Row { y: 10 }),
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
    fn kitty_delete_request_converts_protocol_origin_to_zero_based_geometry_selector() {
        assert_delete_request(
            b"Ga=d,d=p,x=1,y=1",
            KittyDeleteRequest {
                selector: KittyDeleteSelector::Geometry(KittyGeometrySelector::Cell {
                    x: 0,
                    y: 0,
                    z: None,
                }),
                mode: KittyDeleteMode::PlacementsOnly,
            },
        );
        assert_eq!(kitty_delete_request(b"Ga=d,d=p,x=0,y=1"), None);
        assert_eq!(kitty_delete_request(b"Ga=d,d=x,x=0"), None);
        assert_eq!(kitty_delete_request(b"Ga=d,d=y,y=0"), None);
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
            b"Gq=0,a=q,t=x,f=24,s=1,v=1,i=47;a2l0dHktcXVlcnktc2ht" as &[u8],
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
            kitty_query_response(b"Gq=2,a=q,t=x,f=24,s=1,v=1,i=48;a2l0dHktcXVlcnktc2ht");
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
    fn kitty_terminal_image_response_parses_ok_and_error_replies() {
        assert_eq!(
            kitty_terminal_image_response(b"Gi=7,p=3;OK"),
            Some(KittyTerminalImageResponse {
                image_id: Some(7),
                placement_id: Some(3),
                image_number: None,
                is_ok: true,
                message: "OK".to_string(),
            })
        );
        assert_eq!(
            kitty_terminal_image_response(b"GI=9;ENOENT:missing image"),
            Some(KittyTerminalImageResponse {
                image_id: None,
                placement_id: None,
                image_number: Some(9),
                is_ok: false,
                message: "ENOENT:missing image".to_string(),
            })
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
        let apc = KittyApc::parse(b"Ga=t,f=24,s=1,v=1,i=7;EjRW").unwrap();
        let parsed = ParsedKittyCommand::parse(&apc).unwrap();
        let ParsedKittyCommand::ImmediateTransmit {
            protocol_image_id,
            image_format,
            width,
            height,
            media_source: KittyMediaSource::Direct(parsed_payload),
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
            image_id: Some(99),
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
            media_source: KittyMediaSource::Direct(payload.clone()),
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
