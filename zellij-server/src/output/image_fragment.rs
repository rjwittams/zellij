use super::{
    kitty_diff::{KittyScenePlan, KittySceneState},
    FloatingPanesStack, KittyImageChunk, KittyPlaceholderRender, PlacementId, SixelImageChunk,
};
use crate::ClientId;
use zellij_utils::pane_size::{PaneGeom, SizeInPixels};

#[derive(Debug, Clone)]
pub enum ImageFragment {
    Sixel(SixelImageChunk),
    KittyExplicit(KittyImageChunk),
    KittyPlaceholder(KittyPlaceholderRender),
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedAfterTextImages {
    pub client_id: ClientId,
    pub fragments: Vec<ImageFragment>,
    pub kitty_plan: KittyScenePlan,
    pub current_kitty_scene: Option<KittySceneState>,
    pub current_kitty_chunks: Vec<KittyImageChunk>,
    pub current_kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
}

impl Default for PreparedAfterTextImages {
    fn default() -> Self {
        Self {
            client_id: 0,
            fragments: vec![],
            kitty_plan: KittyScenePlan::Diff {
                asset_ops: vec![],
                placement_ops: vec![],
            },
            current_kitty_scene: None,
            current_kitty_chunks: vec![],
            current_kitty_placeholder_renders: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PreparedImageOutput {
    pub before_text_vte: Option<String>,
    pub after_text: PreparedAfterTextImages,
}

fn scale_u32(total: u32, kept: usize, original: usize) -> u32 {
    if original == 0 {
        0
    } else {
        ((total as u64 * kept as u64) / original as u64) as u32
    }
}

fn remap_split_fragment_placement_id(chunk: &KittyImageChunk) -> Option<PlacementId> {
    let base = match chunk.placement_id? {
        PlacementId::Protocol(value) | PlacementId::Synthetic(value) => value,
    };
    let mut hash = (base as u64) ^ 0x9e37_79b9_7f4a_7c15;
    let fields = [
        chunk.cell_x as u64,
        chunk.cell_y as u64,
        chunk.columns as u64,
        chunk.rows as u64,
        chunk.source_x as u64,
        chunk.source_y as u64,
        chunk.source_width as u64,
        chunk.source_height as u64,
        chunk.x_offset as u64,
        chunk.y_offset as u64,
    ];
    for field in fields {
        hash ^= field
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(hash << 6)
            .wrapping_add(hash >> 2);
    }
    Some(PlacementId::Synthetic((hash & 0xffff_ffff) as u32))
}

fn promote_split_explicit_chunk_to_bounded_geometry(chunk: KittyImageChunk) -> KittyImageChunk {
    KittyImageChunk {
        columns_specified: true,
        rows_specified: true,
        ..chunk
    }
}

fn clip_kitty_explicit_fragment(
    pane_geom: &PaneGeom,
    chunk: &KittyImageChunk,
) -> Vec<ImageFragment> {
    let pane_top_edge = pane_geom.y;
    let pane_left_edge = pane_geom.x;
    let pane_bottom_edge = pane_geom.y + pane_geom.rows.as_usize();
    let pane_right_edge = pane_geom.x + pane_geom.cols.as_usize();
    let chunk_top_edge = chunk.cell_y;
    let chunk_left_edge = chunk.cell_x;
    let chunk_bottom_edge = chunk.cell_y + chunk.rows;
    let chunk_right_edge = chunk.cell_x + chunk.columns;

    let intersection_top = pane_top_edge.max(chunk_top_edge);
    let intersection_left = pane_left_edge.max(chunk_left_edge);
    let intersection_bottom = pane_bottom_edge.min(chunk_bottom_edge);
    let intersection_right = pane_right_edge.min(chunk_right_edge);

    if intersection_top >= intersection_bottom || intersection_left >= intersection_right {
        return vec![ImageFragment::KittyExplicit(chunk.clone())];
    }

    let mut uncovered = vec![];
    if intersection_top == chunk_top_edge
        && intersection_bottom == chunk_bottom_edge
        && intersection_left == chunk_left_edge
        && intersection_right == chunk_right_edge
    {
        return uncovered;
    }

    if intersection_top > chunk_top_edge {
        let kept_rows = intersection_top - chunk_top_edge;
        let chunk = KittyImageChunk {
            rows: kept_rows,
            source_height: scale_u32(chunk.source_height, kept_rows, chunk.rows),
            ..chunk.clone()
        };
        let chunk = promote_split_explicit_chunk_to_bounded_geometry(chunk);
        uncovered.push(ImageFragment::KittyExplicit(
            KittyImageChunk {
                placement_id: remap_split_fragment_placement_id(&chunk),
                ..chunk
            },
        ));
    }
    if intersection_bottom < chunk_bottom_edge {
        let removed_rows = intersection_bottom - chunk_top_edge;
        let kept_rows = chunk_bottom_edge - intersection_bottom;
        let chunk = KittyImageChunk {
            cell_y: intersection_bottom,
            rows: kept_rows,
            source_y: chunk.source_y + scale_u32(chunk.source_height, removed_rows, chunk.rows),
            source_height: scale_u32(chunk.source_height, kept_rows, chunk.rows),
            ..chunk.clone()
        };
        let chunk = promote_split_explicit_chunk_to_bounded_geometry(chunk);
        uncovered.push(ImageFragment::KittyExplicit(
            KittyImageChunk {
                placement_id: remap_split_fragment_placement_id(&chunk),
                ..chunk
            },
        ));
    }
    if intersection_left > chunk_left_edge {
        let kept_cols = intersection_left - chunk_left_edge;
        let kept_rows = intersection_bottom - intersection_top;
        let chunk = KittyImageChunk {
            cell_y: intersection_top,
            columns: kept_cols,
            rows: kept_rows,
            source_y: chunk.source_y
                + scale_u32(
                    chunk.source_height,
                    intersection_top - chunk_top_edge,
                    chunk.rows,
                ),
            source_width: scale_u32(chunk.source_width, kept_cols, chunk.columns),
            source_height: scale_u32(chunk.source_height, kept_rows, chunk.rows),
            ..chunk.clone()
        };
        let chunk = promote_split_explicit_chunk_to_bounded_geometry(chunk);
        uncovered.push(ImageFragment::KittyExplicit(
            KittyImageChunk {
                placement_id: remap_split_fragment_placement_id(&chunk),
                ..chunk
            },
        ));
    }
    if intersection_right < chunk_right_edge {
        let removed_cols = intersection_right - chunk_left_edge;
        let kept_cols = chunk_right_edge - intersection_right;
        let kept_rows = intersection_bottom - intersection_top;
        let chunk = KittyImageChunk {
            cell_x: intersection_right,
            cell_y: intersection_top,
            columns: kept_cols,
            rows: kept_rows,
            source_x: chunk.source_x + scale_u32(chunk.source_width, removed_cols, chunk.columns),
            source_y: chunk.source_y
                + scale_u32(
                    chunk.source_height,
                    intersection_top - chunk_top_edge,
                    chunk.rows,
                ),
            source_width: scale_u32(chunk.source_width, kept_cols, chunk.columns),
            source_height: scale_u32(chunk.source_height, kept_rows, chunk.rows),
            ..chunk.clone()
        };
        let chunk = promote_split_explicit_chunk_to_bounded_geometry(chunk);
        uncovered.push(ImageFragment::KittyExplicit(
            KittyImageChunk {
                placement_id: remap_split_fragment_placement_id(&chunk),
                ..chunk
            },
        ));
    }
    uncovered
}

fn clip_sixel_fragment(
    pane_geom: &PaneGeom,
    chunk: &SixelImageChunk,
    character_cell_size: &SizeInPixels,
) -> Vec<ImageFragment> {
    let rounded_sixel_image_pixel_height =
        if chunk.sixel_image_pixel_height % character_cell_size.height > 0 {
            let modulus = chunk.sixel_image_pixel_height % character_cell_size.height;
            chunk.sixel_image_pixel_height + (character_cell_size.height - modulus)
        } else {
            chunk.sixel_image_pixel_height
        };
    let rounded_sixel_image_pixel_width =
        if chunk.sixel_image_pixel_width % character_cell_size.width > 0 {
            let modulus = chunk.sixel_image_pixel_width % character_cell_size.width;
            chunk.sixel_image_pixel_width + (character_cell_size.width - modulus)
        } else {
            chunk.sixel_image_pixel_width
        };

    let pane_top_edge = pane_geom.y * character_cell_size.height;
    let pane_left_edge = pane_geom.x * character_cell_size.width;
    let pane_bottom_edge =
        (pane_geom.y + pane_geom.rows.as_usize().saturating_sub(1)) * character_cell_size.height;
    let pane_right_edge =
        (pane_geom.x + pane_geom.cols.as_usize().saturating_sub(1)) * character_cell_size.width;
    let s_chunk_top_edge = chunk.cell_y * character_cell_size.height;
    let s_chunk_bottom_edge = s_chunk_top_edge + rounded_sixel_image_pixel_height;
    let s_chunk_left_edge = chunk.cell_x * character_cell_size.width;
    let s_chunk_right_edge = s_chunk_left_edge + rounded_sixel_image_pixel_width;

    let mut uncovered_chunks = vec![];
    let pane_covers_chunk_completely = pane_top_edge <= s_chunk_top_edge
        && pane_bottom_edge >= s_chunk_bottom_edge
        && pane_left_edge <= s_chunk_left_edge
        && pane_right_edge >= s_chunk_right_edge;
    let pane_intersects_with_chunk_vertically = (pane_left_edge >= s_chunk_left_edge
        && pane_left_edge <= s_chunk_right_edge)
        || (pane_right_edge >= s_chunk_left_edge && pane_right_edge <= s_chunk_right_edge)
        || (pane_left_edge <= s_chunk_left_edge && pane_right_edge >= s_chunk_right_edge);
    let pane_intersects_with_chunk_horizontally = (pane_top_edge >= s_chunk_top_edge
        && pane_top_edge <= s_chunk_bottom_edge)
        || (pane_bottom_edge >= s_chunk_top_edge && pane_bottom_edge <= s_chunk_bottom_edge)
        || (pane_top_edge <= s_chunk_top_edge && pane_bottom_edge >= s_chunk_bottom_edge);
    if pane_covers_chunk_completely {
        return uncovered_chunks;
    }
    if pane_top_edge >= s_chunk_top_edge
        && pane_top_edge <= s_chunk_bottom_edge
        && pane_intersects_with_chunk_vertically
    {
        uncovered_chunks.push(ImageFragment::Sixel(SixelImageChunk {
            cell_x: chunk.cell_x,
            cell_y: chunk.cell_y,
            sixel_image_pixel_x: chunk.sixel_image_pixel_x,
            sixel_image_pixel_y: chunk.sixel_image_pixel_y,
            sixel_image_pixel_width: rounded_sixel_image_pixel_width,
            sixel_image_pixel_height: pane_top_edge - s_chunk_top_edge,
            sixel_image_id: chunk.sixel_image_id,
        }));
    }
    if pane_bottom_edge <= s_chunk_bottom_edge
        && pane_bottom_edge >= s_chunk_top_edge
        && pane_intersects_with_chunk_vertically
    {
        uncovered_chunks.push(ImageFragment::Sixel(SixelImageChunk {
            cell_x: chunk.cell_x,
            cell_y: (pane_bottom_edge / character_cell_size.height) + 1,
            sixel_image_pixel_x: chunk.sixel_image_pixel_x,
            sixel_image_pixel_y: chunk.sixel_image_pixel_y
                + (pane_bottom_edge - s_chunk_top_edge)
                + character_cell_size.height,
            sixel_image_pixel_width: rounded_sixel_image_pixel_width,
            sixel_image_pixel_height: (rounded_sixel_image_pixel_height
                - (pane_bottom_edge - s_chunk_top_edge))
                .saturating_sub(character_cell_size.height),
            sixel_image_id: chunk.sixel_image_id,
        }));
    }
    if pane_left_edge >= s_chunk_left_edge
        && pane_left_edge <= s_chunk_right_edge
        && pane_intersects_with_chunk_horizontally
    {
        uncovered_chunks.push(ImageFragment::Sixel(SixelImageChunk {
            cell_x: chunk.cell_x,
            cell_y: chunk.cell_y,
            sixel_image_pixel_x: chunk.sixel_image_pixel_x,
            sixel_image_pixel_y: chunk.sixel_image_pixel_y,
            sixel_image_pixel_width: pane_left_edge - s_chunk_left_edge,
            sixel_image_pixel_height: rounded_sixel_image_pixel_height,
            sixel_image_id: chunk.sixel_image_id,
        }));
    }
    if pane_right_edge <= s_chunk_right_edge
        && pane_right_edge >= s_chunk_left_edge
        && pane_intersects_with_chunk_horizontally
    {
        uncovered_chunks.push(ImageFragment::Sixel(SixelImageChunk {
            cell_x: (pane_right_edge / character_cell_size.width) + 1,
            cell_y: chunk.cell_y,
            sixel_image_pixel_x: chunk.sixel_image_pixel_x
                + (pane_right_edge - s_chunk_left_edge)
                + character_cell_size.width,
            sixel_image_pixel_y: chunk.sixel_image_pixel_y,
            sixel_image_pixel_width: (rounded_sixel_image_pixel_width
                - (pane_right_edge - s_chunk_left_edge))
                .saturating_sub(character_cell_size.width),
            sixel_image_pixel_height: rounded_sixel_image_pixel_height,
            sixel_image_id: chunk.sixel_image_id,
        }));
    }
    if uncovered_chunks.is_empty() {
        uncovered_chunks.push(ImageFragment::Sixel(chunk.clone()));
    }
    uncovered_chunks
}

fn clip_kitty_placeholder_fragment(
    pane_geom: &PaneGeom,
    render: &KittyPlaceholderRender,
) -> Vec<ImageFragment> {
    let pane_top_edge = pane_geom.y;
    let pane_left_edge = pane_geom.x;
    let pane_bottom_edge = pane_geom.y + pane_geom.rows.as_usize().saturating_sub(1);
    let pane_right_edge = pane_geom.x + pane_geom.cols.as_usize().saturating_sub(1);
    let visible_cells = render
        .cells
        .iter()
        .filter(|cell| {
            cell.cell_y < pane_top_edge
                || cell.cell_y > pane_bottom_edge
                || cell.cell_x < pane_left_edge
                || cell.cell_x > pane_right_edge
        })
        .cloned()
        .collect::<Vec<_>>();

    if visible_cells.is_empty() {
        vec![]
    } else {
        vec![ImageFragment::KittyPlaceholder(KittyPlaceholderRender {
            cells: visible_cells,
            ..render.clone()
        })]
    }
}

pub(crate) fn clip_image_fragment(
    pane_geom: &PaneGeom,
    fragment: &ImageFragment,
    character_cell_size: Option<&SizeInPixels>,
) -> Vec<ImageFragment> {
    match fragment {
        ImageFragment::Sixel(sixel_chunk) => {
            if let Some(character_cell_size) = character_cell_size {
                clip_sixel_fragment(pane_geom, sixel_chunk, character_cell_size)
            } else {
                vec![fragment.clone()]
            }
        },
        ImageFragment::KittyExplicit(kitty_explicit_chunk) => {
            clip_kitty_explicit_fragment(pane_geom, kitty_explicit_chunk)
        },
        ImageFragment::KittyPlaceholder(kitty_placeholder_render) => {
            clip_kitty_placeholder_fragment(pane_geom, kitty_placeholder_render)
        },
    }
}

pub(crate) fn visible_image_fragments(
    floating_panes_stack: &FloatingPanesStack,
    mut image_fragments: Vec<ImageFragment>,
    z_index: Option<usize>,
    character_cell_size: Option<&SizeInPixels>,
) -> Vec<ImageFragment> {
    let z_index = z_index.unwrap_or(0);
    let mut fragments_to_check: Vec<ImageFragment> = image_fragments.drain(..).collect();
    let panes_to_check = floating_panes_stack.layers.iter().skip(z_index);
    for pane_geom in panes_to_check {
        let fragments_against_this_pane: Vec<ImageFragment> =
            fragments_to_check.drain(..).collect();
        for fragment in fragments_against_this_pane {
            let mut unclipped = clip_image_fragment(pane_geom, &fragment, character_cell_size);
            fragments_to_check.append(&mut unclipped);
        }
    }
    fragments_to_check
}
