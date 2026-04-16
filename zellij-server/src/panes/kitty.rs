use base64;
use zellij_utils::pane_size::SizeInPixels;

use crate::output::{KittyImageChunk, KittyImageData, KittyImagePlacementMode};

const KITTY_UNICODE_PLACEHOLDER_CHAR: char = '\u{10EEEE}';
const KITTY_ROWCOL_DIACRITICS: [char; 297] = [
    '\u{305}',
    '\u{30D}',
    '\u{30E}',
    '\u{310}',
    '\u{312}',
    '\u{33D}',
    '\u{33E}',
    '\u{33F}',
    '\u{346}',
    '\u{34A}',
    '\u{34B}',
    '\u{34C}',
    '\u{350}',
    '\u{351}',
    '\u{352}',
    '\u{357}',
    '\u{35B}',
    '\u{363}',
    '\u{364}',
    '\u{365}',
    '\u{366}',
    '\u{367}',
    '\u{368}',
    '\u{369}',
    '\u{36A}',
    '\u{36B}',
    '\u{36C}',
    '\u{36D}',
    '\u{36E}',
    '\u{36F}',
    '\u{483}',
    '\u{484}',
    '\u{485}',
    '\u{486}',
    '\u{487}',
    '\u{592}',
    '\u{593}',
    '\u{594}',
    '\u{595}',
    '\u{597}',
    '\u{598}',
    '\u{599}',
    '\u{59C}',
    '\u{59D}',
    '\u{59E}',
    '\u{59F}',
    '\u{5A0}',
    '\u{5A1}',
    '\u{5A8}',
    '\u{5A9}',
    '\u{5AB}',
    '\u{5AC}',
    '\u{5AF}',
    '\u{5C4}',
    '\u{610}',
    '\u{611}',
    '\u{612}',
    '\u{613}',
    '\u{614}',
    '\u{615}',
    '\u{616}',
    '\u{617}',
    '\u{657}',
    '\u{658}',
    '\u{659}',
    '\u{65A}',
    '\u{65B}',
    '\u{65D}',
    '\u{65E}',
    '\u{6D6}',
    '\u{6D7}',
    '\u{6D8}',
    '\u{6D9}',
    '\u{6DA}',
    '\u{6DB}',
    '\u{6DC}',
    '\u{6DF}',
    '\u{6E0}',
    '\u{6E1}',
    '\u{6E2}',
    '\u{6E4}',
    '\u{6E7}',
    '\u{6E8}',
    '\u{6EB}',
    '\u{6EC}',
    '\u{730}',
    '\u{732}',
    '\u{733}',
    '\u{735}',
    '\u{736}',
    '\u{73A}',
    '\u{73D}',
    '\u{73F}',
    '\u{740}',
    '\u{741}',
    '\u{743}',
    '\u{745}',
    '\u{747}',
    '\u{749}',
    '\u{74A}',
    '\u{7EB}',
    '\u{7EC}',
    '\u{7ED}',
    '\u{7EE}',
    '\u{7EF}',
    '\u{7F0}',
    '\u{7F1}',
    '\u{7F3}',
    '\u{816}',
    '\u{817}',
    '\u{818}',
    '\u{819}',
    '\u{81B}',
    '\u{81C}',
    '\u{81D}',
    '\u{81E}',
    '\u{81F}',
    '\u{820}',
    '\u{821}',
    '\u{822}',
    '\u{823}',
    '\u{825}',
    '\u{826}',
    '\u{827}',
    '\u{829}',
    '\u{82A}',
    '\u{82B}',
    '\u{82C}',
    '\u{82D}',
    '\u{951}',
    '\u{953}',
    '\u{954}',
    '\u{F82}',
    '\u{F83}',
    '\u{F86}',
    '\u{F87}',
    '\u{135D}',
    '\u{135E}',
    '\u{135F}',
    '\u{17DD}',
    '\u{193A}',
    '\u{1A17}',
    '\u{1A75}',
    '\u{1A76}',
    '\u{1A77}',
    '\u{1A78}',
    '\u{1A79}',
    '\u{1A7A}',
    '\u{1A7B}',
    '\u{1A7C}',
    '\u{1B6B}',
    '\u{1B6D}',
    '\u{1B6E}',
    '\u{1B6F}',
    '\u{1B70}',
    '\u{1B71}',
    '\u{1B72}',
    '\u{1B73}',
    '\u{1CD0}',
    '\u{1CD1}',
    '\u{1CD2}',
    '\u{1CDA}',
    '\u{1CDB}',
    '\u{1CE0}',
    '\u{1DC0}',
    '\u{1DC1}',
    '\u{1DC3}',
    '\u{1DC4}',
    '\u{1DC5}',
    '\u{1DC6}',
    '\u{1DC7}',
    '\u{1DC8}',
    '\u{1DC9}',
    '\u{1DCB}',
    '\u{1DCC}',
    '\u{1DD1}',
    '\u{1DD2}',
    '\u{1DD3}',
    '\u{1DD4}',
    '\u{1DD5}',
    '\u{1DD6}',
    '\u{1DD7}',
    '\u{1DD8}',
    '\u{1DD9}',
    '\u{1DDA}',
    '\u{1DDB}',
    '\u{1DDC}',
    '\u{1DDD}',
    '\u{1DDE}',
    '\u{1DDF}',
    '\u{1DE0}',
    '\u{1DE1}',
    '\u{1DE2}',
    '\u{1DE3}',
    '\u{1DE4}',
    '\u{1DE5}',
    '\u{1DE6}',
    '\u{1DFE}',
    '\u{20D0}',
    '\u{20D1}',
    '\u{20D4}',
    '\u{20D5}',
    '\u{20D6}',
    '\u{20D7}',
    '\u{20DB}',
    '\u{20DC}',
    '\u{20E1}',
    '\u{20E7}',
    '\u{20E9}',
    '\u{20F0}',
    '\u{2CEF}',
    '\u{2CF0}',
    '\u{2CF1}',
    '\u{2DE0}',
    '\u{2DE1}',
    '\u{2DE2}',
    '\u{2DE3}',
    '\u{2DE4}',
    '\u{2DE5}',
    '\u{2DE6}',
    '\u{2DE7}',
    '\u{2DE8}',
    '\u{2DE9}',
    '\u{2DEA}',
    '\u{2DEB}',
    '\u{2DEC}',
    '\u{2DED}',
    '\u{2DEE}',
    '\u{2DEF}',
    '\u{2DF0}',
    '\u{2DF1}',
    '\u{2DF2}',
    '\u{2DF3}',
    '\u{2DF4}',
    '\u{2DF5}',
    '\u{2DF6}',
    '\u{2DF7}',
    '\u{2DF8}',
    '\u{2DF9}',
    '\u{2DFA}',
    '\u{2DFB}',
    '\u{2DFC}',
    '\u{2DFD}',
    '\u{2DFE}',
    '\u{2DFF}',
    '\u{A66F}',
    '\u{A67C}',
    '\u{A67D}',
    '\u{A6F0}',
    '\u{A6F1}',
    '\u{A8E0}',
    '\u{A8E1}',
    '\u{A8E2}',
    '\u{A8E3}',
    '\u{A8E4}',
    '\u{A8E5}',
    '\u{A8E6}',
    '\u{A8E7}',
    '\u{A8E8}',
    '\u{A8E9}',
    '\u{A8EA}',
    '\u{A8EB}',
    '\u{A8EC}',
    '\u{A8ED}',
    '\u{A8EE}',
    '\u{A8EF}',
    '\u{A8F0}',
    '\u{A8F1}',
    '\u{AAB0}',
    '\u{AAB2}',
    '\u{AAB3}',
    '\u{AAB7}',
    '\u{AAB8}',
    '\u{AABE}',
    '\u{AABF}',
    '\u{AAC1}',
    '\u{FE20}',
    '\u{FE21}',
    '\u{FE22}',
    '\u{FE23}',
    '\u{FE24}',
    '\u{FE25}',
    '\u{FE26}',
    '\u{10A0F}',
    '\u{10A38}',
    '\u{1D185}',
    '\u{1D186}',
    '\u{1D187}',
    '\u{1D188}',
    '\u{1D189}',
    '\u{1D1AA}',
    '\u{1D1AB}',
    '\u{1D1AC}',
    '\u{1D1AD}',
    '\u{1D242}',
    '\u{1D243}',
    '\u{1D244}',
];
use crate::panes::pane_image_scene::{
    project_placement_to_viewport, FlowAnchor, ImageAssetId, ImagePlacementGeometry,
    PlacementOccupancy,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Debug)]
pub enum KittyStoredImageData {
    Png {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Rgba {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
}

#[derive(Clone, Debug)]
pub struct KittyImage {
    pub id: u32,
    pub data: KittyStoredImageData,
}

impl KittyImage {
    pub fn width(&self) -> u32 {
        match &self.data {
            KittyStoredImageData::Png { width, .. } | KittyStoredImageData::Rgba { width, .. } => {
                *width
            },
        }
    }

    pub fn height(&self) -> u32 {
        match &self.data {
            KittyStoredImageData::Png { height, .. }
            | KittyStoredImageData::Rgba { height, .. } => *height,
        }
    }

    pub fn chunk_data(&self) -> KittyImageData {
        match &self.data {
            KittyStoredImageData::Png { data, .. } => KittyImageData::Png { data: data.clone() },
            KittyStoredImageData::Rgba {
                data,
                width,
                height,
            } => KittyImageData::Rgba {
                data: data.clone(),
                width: *width,
                height: *height,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct KittyPlacement {
    pub image_id: u32,
    pub placement_id: Option<u32>,
    pub placement_mode: KittyImagePlacementMode,
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
    image_id: u32,
    image_format: KittyImageFormat,
    width: u32,
    height: u32,
    placement: Option<KittyPlacement>,
    payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KittyImageFormat {
    Png,
    Rgba,
}

#[derive(Clone, Debug, Default)]
pub struct KittyImageState {
    images: HashMap<u32, KittyImage>,
    placements: Vec<KittyPlacement>,
    protocol_image_id_to_internal_id: HashMap<u32, u32>,
    pending_transmit: Option<PendingKittyTransmit>,
}

static NEXT_GLOBAL_KITTY_IMAGE_ID: AtomicU32 = AtomicU32::new(1);

fn next_global_kitty_image_id() -> u32 {
    NEXT_GLOBAL_KITTY_IMAGE_ID.fetch_add(1, Ordering::Relaxed)
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
    pub geometry: ImagePlacementGeometry,
    pub protocol_image_id: Option<u32>,
    pub protocol_placement_id: Option<u32>,
    pub placement_mode: KittyImagePlacementMode,
}

impl KittyImageState {
    pub fn image_chunk_data(&self, image_id: u32) -> Option<KittyImageData> {
        self.images.get(&image_id).map(|image| image.chunk_data())
    }

    pub fn image_dimensions(&self, image_id: u32) -> Option<(u32, u32)> {
        self.images
            .get(&image_id)
            .map(|image| (image.width(), image.height()))
    }

    pub fn placement(&self, image_id: u32, placement_id: Option<u32>) -> Option<&KittyPlacement> {
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
    ) -> Option<KittyImageInsertion> {
        let pending = self.pending_transmit.take()?;
        let protocol_image_id = pending.protocol_image_id;
        let mut placement = pending.placement.clone();
        let image = pending.into_image()?;
        if let Some(placement) = placement.as_mut() {
            placement.anchor = anchor;
        }
        self.images.insert(image.id, image.clone());
        let asset_id = ImageAssetId(image.id as u64);
        let Some(placement) = placement else {
            return None;
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
        let geometry =
            placement.geometry_for_image(&image, cursor_x, scrollback_row, character_cell_size);
        self.placements.push(placement);
        Some(KittyImageInsertion {
            asset_id,
            geometry,
            protocol_image_id,
            protocol_placement_id,
            placement_mode,
        })
    }

    pub fn handle_apc(
        &mut self,
        apc_bytes: &[u8],
        anchor: FlowAnchor,
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> Option<KittyImageInsertion> {
        let command = ParsedKittyCommand::parse(apc_bytes)?;
        match command {
            ParsedKittyCommand::ImmediateTransmit {
                protocol_image_id,
                image_format,
                width,
                height,
                mut placement,
                more,
                payload,
            } => {
                let image_id = if let Some(protocol_image_id) = protocol_image_id {
                    *self
                        .protocol_image_id_to_internal_id
                        .entry(protocol_image_id)
                        .or_insert_with(next_global_kitty_image_id)
                } else {
                    next_global_kitty_image_id()
                };
                if let Some(placement) = placement.as_mut() {
                    placement.image_id = image_id;
                }
                self.pending_transmit = Some(PendingKittyTransmit {
                    protocol_image_id,
                    image_id,
                    image_format,
                    width,
                    height,
                    placement,
                    payload,
                });
                if more {
                    None
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
                mut placement,
            } => {
                let image_id = *self
                    .protocol_image_id_to_internal_id
                    .get(&protocol_image_id)?;
                let image = self.images.get(&image_id)?.clone();
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
                    &image,
                    cursor_x,
                    scrollback_row,
                    character_cell_size,
                );
                let protocol_placement_id = placement.placement_id;
                let placement_mode = placement.placement_mode;
                self.placements.push(placement);
                Some(KittyImageInsertion {
                    asset_id: ImageAssetId(image_id as u64),
                    geometry,
                    protocol_image_id: Some(protocol_image_id),
                    protocol_placement_id,
                    placement_mode,
                })
            },
            ParsedKittyCommand::TransmitChunk { more, payload } => {
                let pending = self.pending_transmit.as_mut()?;
                pending.payload.extend(payload);
                if more {
                    None
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
        for placement in &self.placements {
            let Some(image) = self.images.get(&placement.image_id) else {
                continue;
            };
            let mut source_x = placement.source_x.unwrap_or(0);
            let mut source_y = placement.source_y.unwrap_or(0);
            let mut source_width = placement
                .source_width
                .unwrap_or_else(|| image.width().saturating_sub(source_x));
            let mut source_height = placement
                .source_height
                .unwrap_or_else(|| image.height().saturating_sub(source_y));
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
                image_id: image.id,
                placement_id: placement.placement_id,
                placement_mode: placement.placement_mode,
                cell_x,
                cell_y,
                columns,
                rows,
                source_x,
                source_y,
                source_width,
                source_height,
                z_index: placement.z_index.unwrap_or(0),
                x_offset: placement.x_offset.unwrap_or(0),
                y_offset: placement.y_offset.unwrap_or(0),
                image_data: image.chunk_data(),
            });
        }
        chunks
    }

    pub fn clear(&mut self) {
        self.images.clear();
        self.placements.clear();
        self.protocol_image_id_to_internal_id.clear();
        self.pending_transmit = None;
    }

    pub fn delete_protocol_placement(&mut self, protocol_image_id: u32, placement_id: Option<u32>) {
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
    }

    pub fn serialize_chunks(chunks: &[KittyImageChunk]) -> String {
        if chunks.is_empty() {
            return String::new();
        }
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str("\u{1b}[s");

        let mut transmitted_image_ids = std::collections::HashSet::new();
        for chunk in chunks {
            if transmitted_image_ids.insert(chunk.image_id) {
                for transmit_command in serialize_transmit(chunk.image_id, &chunk.image_data) {
                    raw_vte_output.push_str("\u{1b}_G");
                    raw_vte_output.push_str(&transmit_command);
                    raw_vte_output.push_str("\u{1b}\\");
                }
            }
        }

        for (placement_id, chunk) in chunks.iter().enumerate() {
            let placement_id = placement_id as u32 + 1;
            let cursor_x = chunk.cell_x + 1;
            let cursor_y = chunk.cell_y + 1;
            raw_vte_output.push_str(&format!("\u{1b}[{};{}H", cursor_y, cursor_x));
            raw_vte_output.push_str("\u{1b}_G");
            raw_vte_output.push_str(&serialize_display(chunk, placement_id));
            raw_vte_output.push_str("\u{1b}\\");
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }

    pub fn serialize_placeholder_renders(
        renders: &[crate::output::KittyPlaceholderRender],
    ) -> String {
        if renders.is_empty() {
            return String::new();
        }
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str("\u{1b}[s");

        let mut transmitted_image_ids = std::collections::HashSet::new();
        for render in renders {
            if transmitted_image_ids.insert(render.image_id) {
                for transmit_command in serialize_transmit(render.image_id, &render.image_data) {
                    raw_vte_output.push_str("\u{1b}_G");
                    raw_vte_output.push_str(&transmit_command);
                    raw_vte_output.push_str("\u{1b}\\");
                }
            }
        }

        for (placement_index, render) in renders.iter().enumerate() {
            let placement_id = render.placement_id.unwrap_or(placement_index as u32 + 1);
            raw_vte_output.push_str(&serialize_placeholder_render(render, placement_id));
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }
}

#[derive(Clone, Debug)]
pub enum KittyQueryResponse {
    Ok {
        image_id: Option<u32>,
        placement_id: Option<u32>,
    },
}

impl KittyQueryResponse {
    pub fn to_apc_response(&self) -> String {
        match self {
            KittyQueryResponse::Ok {
                image_id,
                placement_id,
            } => {
                let mut control_data = String::new();
                if let Some(image_id) = image_id {
                    control_data.push_str(&format!("i={}", image_id));
                    if let Some(placement_id) = placement_id {
                        control_data.push_str(&format!(",p={}", placement_id));
                    }
                    control_data.push(';');
                } else {
                    control_data.push(';');
                }
                format!("\u{1b}_G{}OK\u{1b}\\", control_data)
            },
        }
    }
}

#[derive(Clone, Debug)]
enum ParsedKittyCommand {
    ImmediateTransmit {
        protocol_image_id: Option<u32>,
        image_format: KittyImageFormat,
        width: u32,
        height: u32,
        placement: Option<KittyPlacement>,
        more: bool,
        payload: Vec<u8>,
    },
    DisplayPlacement {
        protocol_image_id: u32,
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
        image: &KittyImage,
        cursor_x: usize,
        scrollback_row: usize,
        character_cell_size: Option<SizeInPixels>,
    ) -> ImagePlacementGeometry {
        let source_x = self.source_x.unwrap_or(0);
        let source_y = self.source_y.unwrap_or(0);
        let source_width = self
            .source_width
            .unwrap_or_else(|| image.width().saturating_sub(source_x));
        let source_height = self
            .source_height
            .unwrap_or_else(|| image.height().saturating_sub(source_y));
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
    fn into_image(self) -> Option<KittyImage> {
        let data = match self.image_format {
            KittyImageFormat::Png => {
                let (width, height) = parse_png_dimensions(&self.payload)?;
                KittyStoredImageData::Png {
                    data: self.payload,
                    width,
                    height,
                }
            },
            KittyImageFormat::Rgba => KittyStoredImageData::Rgba {
                data: self.payload,
                width: self.width,
                height: self.height,
            },
        };
        Some(KittyImage {
            id: self.image_id,
            data,
        })
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

pub fn kitty_delete_all_visible(apc_bytes: &[u8]) -> bool {
    let Some(kv) = kitty_delete_header(apc_bytes) else {
        return false;
    };
    kv.get("a").copied() == Some("d")
        && matches!(kv.get("d").copied(), None | Some("a") | Some("A"))
}

pub fn kitty_delete_by_image_id(apc_bytes: &[u8]) -> Option<(u32, Option<u32>)> {
    let kv = kitty_delete_header(apc_bytes)?;
    if kv.get("a").copied() != Some("d") || kv.get("d").copied() != Some("i") {
        return None;
    }
    let image_id = kv.get("i")?.parse::<u32>().ok()?;
    let placement_id = kv.get("p").and_then(|p| p.parse::<u32>().ok());
    Some((image_id, placement_id))
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

    let format = kv.get("f").copied().unwrap_or("32");
    let payload = base64::decode(payload_b64).ok()?;
    let valid = match format {
        "24" => {
            let width = kv.get("s").and_then(|v| v.parse::<usize>().ok())?;
            let height = kv.get("v").and_then(|v| v.parse::<usize>().ok())?;
            payload.len() == width * height * 3
        },
        "32" => {
            let width = kv.get("s").and_then(|v| v.parse::<usize>().ok())?;
            let height = kv.get("v").and_then(|v| v.parse::<usize>().ok())?;
            payload.len() == width * height * 4
        },
        "100" => parse_png_dimensions(&payload).is_some(),
        _ => false,
    };
    if !valid {
        return None;
    }

    Some(KittyQueryResponse::Ok {
        image_id: kv.get("i").and_then(|i| i.parse::<u32>().ok()),
        placement_id: kv.get("p").and_then(|p| p.parse::<u32>().ok()),
    })
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
        let payload = base64::decode(payload).ok()?;

        if let Some(action) = kv.get("a") {
            let placement = KittyPlacement {
                image_id: 0,
                placement_id: kv.get("p").and_then(|p| p.parse::<u32>().ok()),
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
                ..Default::default()
            };
            match *action {
                "T" | "t" => {
                    let image_format = match kv.get("f").copied().unwrap_or("32") {
                        "100" => KittyImageFormat::Png,
                        "32" => KittyImageFormat::Rgba,
                        _ => return None,
                    };
                    let protocol_image_id = kv.get("i").and_then(|i| i.parse::<u32>().ok());
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
                        image_format,
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
                    let protocol_image_id = kv.get("i").and_then(|i| i.parse::<u32>().ok())?;
                    Some(ParsedKittyCommand::DisplayPlacement {
                        protocol_image_id,
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
        KittyImageData::Png { data } => {
            parts.push("f=100".to_string());
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
    if chunk.columns > 0 {
        parts.push(format!("c={}", chunk.columns));
    }
    if chunk.rows > 0 {
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

fn serialize_placeholder_render(
    render: &crate::output::KittyPlaceholderRender,
    placement_id: u32,
) -> String {
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
    render: &crate::output::KittyPlaceholderRender,
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

    fn test_image(width: u32, height: u32) -> KittyImage {
        KittyImage {
            id: 1,
            data: KittyStoredImageData::Rgba {
                data: vec![0; (width * height * 4) as usize],
                width,
                height,
            },
        }
    }

    #[test]
    fn one_dimensional_kitty_sizing_preserves_prediction_and_wire_intent() {
        let image = test_image(40, 20);
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
                placement_id: Some(7),
                placement_mode: KittyImagePlacementMode::Explicit,
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
            let geometry = placement.geometry_for_image(&image, 0, 0, cell_size);
            assert_eq!(geometry.columns, expected_columns);
            assert_eq!(geometry.rows, expected_rows);
            assert_eq!(geometry.columns_specified, expect_c);
            assert_eq!(geometry.rows_specified, expect_r);

            let chunk = KittyImageChunk {
                image_id: 1,
                placement_id: Some(7),
                placement_mode: KittyImagePlacementMode::Explicit,
                cell_x: 0,
                cell_y: 0,
                columns: if geometry.columns_specified {
                    geometry.columns
                } else {
                    0
                },
                rows: if geometry.rows_specified {
                    geometry.rows
                } else {
                    0
                },
                source_x: geometry.source_x,
                source_y: geometry.source_y,
                source_width: geometry.source_width,
                source_height: geometry.source_height,
                z_index: geometry.z_index,
                x_offset: geometry.x_offset,
                y_offset: geometry.y_offset,
                image_data: image.chunk_data(),
            };
            let serialized = serialize_display(&chunk, 7);
            assert_eq!(serialized.contains("c="), expect_c);
            assert_eq!(serialized.contains("r="), expect_r);
        }
    }
}
