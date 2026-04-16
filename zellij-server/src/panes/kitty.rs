use base64;
use zellij_utils::pane_size::SizeInPixels;

use crate::output::{KittyImageChunk, KittyImageData};
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
            KittyStoredImageData::Rgba { data, width, height } => KittyImageData::Rgba {
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
            placement_id: None,
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

#[derive(Clone, Debug)]
struct PendingKittyTransmit {
    protocol_image_id: Option<u32>,
    image_id: u32,
    image_format: KittyImageFormat,
    width: u32,
    height: u32,
    placement: KittyPlacement,
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
        placement.anchor = anchor;
        self.images.insert(image.id, image.clone());
        let asset_id = ImageAssetId(image.id as u64);
        self.placements.retain(|p| {
            if let Some(new_placement_id) = placement.placement_id {
                !(p.image_id == placement.image_id && p.placement_id == Some(new_placement_id))
            } else {
                true
            }
        });
        let protocol_placement_id = placement.placement_id;
        let geometry = placement.geometry_for_image(
            &image,
            cursor_x,
            scrollback_row,
            character_cell_size,
        );
        self.placements.push(placement);
        Some(KittyImageInsertion {
            asset_id,
            geometry,
            protocol_image_id,
            protocol_placement_id,
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
                placement.image_id = image_id;
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
                source_x = source_x + scale_u32(source_width, projection.clipped_left_cols, columns);
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

    pub fn serialize_chunks(chunks: &[KittyImageChunk]) -> String {
        if chunks.is_empty() {
            return String::new();
        }
        let mut raw_vte_output = String::new();
        raw_vte_output.push_str("\u{1b}[s");

        let mut transmitted_image_ids = std::collections::HashSet::new();
        for chunk in chunks {
            if transmitted_image_ids.insert(chunk.image_id) {
                raw_vte_output.push_str("\u{1b}_G");
                raw_vte_output.push_str(&serialize_transmit(chunk));
                raw_vte_output.push_str("\u{1b}\\");
            }
        }

        for (placement_id, chunk) in chunks.iter().enumerate() {
            let cursor_x = chunk.cell_x + 1;
            let cursor_y = chunk.cell_y + 1;
            raw_vte_output.push_str(&format!("\u{1b}[{};{}H", cursor_y, cursor_x));
            raw_vte_output.push_str("\u{1b}_G");
            raw_vte_output.push_str(&serialize_display(chunk, placement_id as u32 + 1));
            raw_vte_output.push_str("\u{1b}\\");
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
        placement: KittyPlacement,
        more: bool,
        payload: Vec<u8>,
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
            (
                self.columns.unwrap_or_else(|| {
                    ((source_width as usize + cell_size.width.saturating_sub(1))
                        / cell_size.width)
                        .max(1) as u32
                }) as usize,
                self.rows.unwrap_or_else(|| {
                    ((source_height as usize + cell_size.height.saturating_sub(1))
                        / cell_size.height)
                        .max(1) as u32
                }) as usize,
            )
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

pub fn kitty_delete_all_visible(apc_bytes: &[u8]) -> bool {
    let Some(rest) = apc_bytes.strip_prefix(b"G") else {
        return false;
    };
    let mut parts = rest.splitn(2, |b| *b == b';');
    let Some(header) = parts.next() else {
        return false;
    };
    let Ok(header) = std::str::from_utf8(header) else {
        return false;
    };
    let mut action = None;
    let mut delete_kind = None;
    for part in header.split(',') {
        if part.is_empty() {
            continue;
        }
        let mut split = part.splitn(2, '=');
        let Some(key) = split.next() else {
            continue;
        };
        let value = split.next().unwrap_or("");
        match key {
            "a" => action = Some(value),
            "d" => delete_kind = Some(value),
            _ => {},
        }
    }
    action == Some("d") && matches!(delete_kind, None | Some("a") | Some("A"))
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
            if *action != "T" {
                return None;
            }
            let image_format = match kv.get("f").copied().unwrap_or("32") {
                "100" => KittyImageFormat::Png,
                "32" => KittyImageFormat::Rgba,
                _ => return None,
            };
            let protocol_image_id = kv.get("i").and_then(|i| i.parse::<u32>().ok());
            let width = kv.get("s").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
            let height = kv.get("v").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
            let placement = KittyPlacement {
                image_id: 0,
            placement_id: kv.get("p").and_then(|p| p.parse::<u32>().ok()),
            source_x: kv.get("x").and_then(|v| v.parse::<u32>().ok()),
            source_y: kv.get("y").and_then(|v| v.parse::<u32>().ok()),
            source_width: kv.get("w").and_then(|v| v.parse::<u32>().ok()),
            source_height: kv.get("h").and_then(|v| v.parse::<u32>().ok()),
            columns: kv.get("c").and_then(|v| v.parse::<u32>().ok()),
            rows: kv.get("r").and_then(|v| v.parse::<u32>().ok()),
            x_offset: kv.get("X").and_then(|v| v.parse::<u32>().ok()),
            y_offset: kv.get("Y").and_then(|v| v.parse::<u32>().ok()),
            z_index: kv.get("z").and_then(|v| v.parse::<i32>().ok()),
            ..Default::default()
        };
            Some(ParsedKittyCommand::ImmediateTransmit {
                protocol_image_id,
                image_format,
                width,
                height,
                placement,
                more,
                payload,
            })
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

fn serialize_transmit(chunk: &KittyImageChunk) -> String {
    let mut parts = vec![
        "a=t".to_string(),
        format!("i={}", chunk.image_id),
        "q=2".to_string(),
    ];
    let payload = match &chunk.image_data {
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
    format!("{};{}", parts.join(","), payload)
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
        format!("c={}", chunk.columns),
        format!("r={}", chunk.rows),
    ];
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
