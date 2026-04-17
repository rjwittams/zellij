use super::{FloatingPanesStack, KittyImageChunk, KittyPlaceholderRender, SixelImageChunk};
use crate::ClientId;
use zellij_utils::pane_size::{PaneGeom, SizeInPixels};

#[derive(Debug, Clone)]
pub struct SixelFragment {
    pub chunk: SixelImageChunk,
}

#[derive(Debug, Clone)]
pub struct KittyExplicitFragment {
    pub chunk: KittyImageChunk,
}

#[derive(Debug, Clone)]
pub struct KittyPlaceholderFragment {
    pub render: KittyPlaceholderRender,
}

#[derive(Debug, Clone)]
pub enum ImageFragment {
    Sixel(SixelFragment),
    KittyExplicit(KittyExplicitFragment),
    KittyPlaceholder(KittyPlaceholderFragment),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PreparedAfterTextImages {
    pub client_id: ClientId,
    pub fragments: Vec<ImageFragment>,
    pub current_kitty_chunks: Vec<KittyImageChunk>,
    pub current_kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
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

fn clip_kitty_explicit_fragment(
    pane_geom: &PaneGeom,
    fragment: &KittyExplicitFragment,
) -> Vec<ImageFragment> {
    let k_chunk = &fragment.chunk;
    let pane_top_edge = pane_geom.y;
    let pane_left_edge = pane_geom.x;
    let pane_bottom_edge = pane_geom.y + pane_geom.rows.as_usize().saturating_sub(1);
    let pane_right_edge = pane_geom.x + pane_geom.cols.as_usize().saturating_sub(1);
    let chunk_top_edge = k_chunk.cell_y;
    let chunk_left_edge = k_chunk.cell_x;
    let chunk_bottom_edge = k_chunk.cell_y + k_chunk.rows.saturating_sub(1);
    let chunk_right_edge = k_chunk.cell_x + k_chunk.columns.saturating_sub(1);
    let mut uncovered = vec![];
    let covers_completely = pane_top_edge <= chunk_top_edge
        && pane_bottom_edge >= chunk_bottom_edge
        && pane_left_edge <= chunk_left_edge
        && pane_right_edge >= chunk_right_edge;
    if covers_completely {
        return uncovered;
    }
    let intersects_vertically = (pane_left_edge >= chunk_left_edge
        && pane_left_edge <= chunk_right_edge)
        || (pane_right_edge >= chunk_left_edge && pane_right_edge <= chunk_right_edge)
        || (pane_left_edge <= chunk_left_edge && pane_right_edge >= chunk_right_edge);
    let intersects_horizontally = (pane_top_edge >= chunk_top_edge
        && pane_top_edge <= chunk_bottom_edge)
        || (pane_bottom_edge >= chunk_top_edge && pane_bottom_edge <= chunk_bottom_edge)
        || (pane_top_edge <= chunk_top_edge && pane_bottom_edge >= chunk_bottom_edge);
    if pane_top_edge > chunk_top_edge
        && pane_top_edge <= chunk_bottom_edge
        && intersects_vertically
    {
        let kept_rows = pane_top_edge - chunk_top_edge;
        uncovered.push(ImageFragment::KittyExplicit(KittyExplicitFragment {
            chunk: KittyImageChunk {
                rows: kept_rows,
                source_height: scale_u32(k_chunk.source_height, kept_rows, k_chunk.rows),
                ..k_chunk.clone()
            },
        }));
    }
    if pane_bottom_edge >= chunk_top_edge
        && pane_bottom_edge < chunk_bottom_edge
        && intersects_vertically
    {
        let removed_rows = pane_bottom_edge + 1 - chunk_top_edge;
        let kept_rows = k_chunk.rows.saturating_sub(removed_rows);
        uncovered.push(ImageFragment::KittyExplicit(KittyExplicitFragment {
            chunk: KittyImageChunk {
                cell_y: pane_bottom_edge + 1,
                rows: kept_rows,
                source_y: k_chunk.source_y
                    + scale_u32(k_chunk.source_height, removed_rows, k_chunk.rows),
                source_height: scale_u32(k_chunk.source_height, kept_rows, k_chunk.rows),
                ..k_chunk.clone()
            },
        }));
    }
    if pane_left_edge > chunk_left_edge
        && pane_left_edge <= chunk_right_edge
        && intersects_horizontally
    {
        let kept_cols = pane_left_edge - chunk_left_edge;
        uncovered.push(ImageFragment::KittyExplicit(KittyExplicitFragment {
            chunk: KittyImageChunk {
                columns: kept_cols,
                source_width: scale_u32(k_chunk.source_width, kept_cols, k_chunk.columns),
                ..k_chunk.clone()
            },
        }));
    }
    if pane_right_edge >= chunk_left_edge
        && pane_right_edge < chunk_right_edge
        && intersects_horizontally
    {
        let removed_cols = pane_right_edge + 1 - chunk_left_edge;
        let kept_cols = k_chunk.columns.saturating_sub(removed_cols);
        uncovered.push(ImageFragment::KittyExplicit(KittyExplicitFragment {
            chunk: KittyImageChunk {
                cell_x: pane_right_edge + 1,
                columns: kept_cols,
                source_x: k_chunk.source_x
                    + scale_u32(k_chunk.source_width, removed_cols, k_chunk.columns),
                source_width: scale_u32(k_chunk.source_width, kept_cols, k_chunk.columns),
                ..k_chunk.clone()
            },
        }));
    }
    if uncovered.is_empty() {
        uncovered.push(ImageFragment::KittyExplicit(fragment.clone()));
    }
    uncovered
}

fn clip_sixel_fragment(
    pane_geom: &PaneGeom,
    fragment: &SixelFragment,
    character_cell_size: &SizeInPixels,
) -> Vec<ImageFragment> {
    let s_chunk = &fragment.chunk;
    let rounded_sixel_image_pixel_height =
        if s_chunk.sixel_image_pixel_height % character_cell_size.height > 0 {
            let modulus = s_chunk.sixel_image_pixel_height % character_cell_size.height;
            s_chunk.sixel_image_pixel_height + (character_cell_size.height - modulus)
        } else {
            s_chunk.sixel_image_pixel_height
        };
    let rounded_sixel_image_pixel_width =
        if s_chunk.sixel_image_pixel_width % character_cell_size.width > 0 {
            let modulus = s_chunk.sixel_image_pixel_width % character_cell_size.width;
            s_chunk.sixel_image_pixel_width + (character_cell_size.width - modulus)
        } else {
            s_chunk.sixel_image_pixel_width
        };

    let pane_top_edge = pane_geom.y * character_cell_size.height;
    let pane_left_edge = pane_geom.x * character_cell_size.width;
    let pane_bottom_edge = (pane_geom.y + pane_geom.rows.as_usize().saturating_sub(1))
        * character_cell_size.height;
    let pane_right_edge =
        (pane_geom.x + pane_geom.cols.as_usize().saturating_sub(1)) * character_cell_size.width;
    let s_chunk_top_edge = s_chunk.cell_y * character_cell_size.height;
    let s_chunk_bottom_edge = s_chunk_top_edge + rounded_sixel_image_pixel_height;
    let s_chunk_left_edge = s_chunk.cell_x * character_cell_size.width;
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
        uncovered_chunks.push(ImageFragment::Sixel(SixelFragment {
            chunk: SixelImageChunk {
                cell_x: s_chunk.cell_x,
                cell_y: s_chunk.cell_y,
                sixel_image_pixel_x: s_chunk.sixel_image_pixel_x,
                sixel_image_pixel_y: s_chunk.sixel_image_pixel_y,
                sixel_image_pixel_width: rounded_sixel_image_pixel_width,
                sixel_image_pixel_height: pane_top_edge - s_chunk_top_edge,
                sixel_image_id: s_chunk.sixel_image_id,
            },
        }));
    }
    if pane_bottom_edge <= s_chunk_bottom_edge
        && pane_bottom_edge >= s_chunk_top_edge
        && pane_intersects_with_chunk_vertically
    {
        uncovered_chunks.push(ImageFragment::Sixel(SixelFragment {
            chunk: SixelImageChunk {
                cell_x: s_chunk.cell_x,
                cell_y: (pane_bottom_edge / character_cell_size.height) + 1,
                sixel_image_pixel_x: s_chunk.sixel_image_pixel_x,
                sixel_image_pixel_y: s_chunk.sixel_image_pixel_y
                    + (pane_bottom_edge - s_chunk_top_edge)
                    + character_cell_size.height,
                sixel_image_pixel_width: rounded_sixel_image_pixel_width,
                sixel_image_pixel_height: (rounded_sixel_image_pixel_height
                    - (pane_bottom_edge - s_chunk_top_edge))
                    .saturating_sub(character_cell_size.height),
                sixel_image_id: s_chunk.sixel_image_id,
            },
        }));
    }
    if pane_left_edge >= s_chunk_left_edge
        && pane_left_edge <= s_chunk_right_edge
        && pane_intersects_with_chunk_horizontally
    {
        uncovered_chunks.push(ImageFragment::Sixel(SixelFragment {
            chunk: SixelImageChunk {
                cell_x: s_chunk.cell_x,
                cell_y: s_chunk.cell_y,
                sixel_image_pixel_x: s_chunk.sixel_image_pixel_x,
                sixel_image_pixel_y: s_chunk.sixel_image_pixel_y,
                sixel_image_pixel_width: pane_left_edge - s_chunk_left_edge,
                sixel_image_pixel_height: rounded_sixel_image_pixel_height,
                sixel_image_id: s_chunk.sixel_image_id,
            },
        }));
    }
    if pane_right_edge <= s_chunk_right_edge
        && pane_right_edge >= s_chunk_left_edge
        && pane_intersects_with_chunk_horizontally
    {
        uncovered_chunks.push(ImageFragment::Sixel(SixelFragment {
            chunk: SixelImageChunk {
                cell_x: (pane_right_edge / character_cell_size.width) + 1,
                cell_y: s_chunk.cell_y,
                sixel_image_pixel_x: s_chunk.sixel_image_pixel_x
                    + (pane_right_edge - s_chunk_left_edge)
                    + character_cell_size.width,
                sixel_image_pixel_y: s_chunk.sixel_image_pixel_y,
                sixel_image_pixel_width: (rounded_sixel_image_pixel_width
                    - (pane_right_edge - s_chunk_left_edge))
                    .saturating_sub(character_cell_size.width),
                sixel_image_pixel_height: rounded_sixel_image_pixel_height,
                sixel_image_id: s_chunk.sixel_image_id,
            },
        }));
    }
    if uncovered_chunks.is_empty() {
        uncovered_chunks.push(ImageFragment::Sixel(fragment.clone()));
    }
    uncovered_chunks
}

fn clip_kitty_placeholder_fragment(
    pane_geom: &PaneGeom,
    fragment: &KittyPlaceholderFragment,
) -> Vec<ImageFragment> {
    let pane_top_edge = pane_geom.y;
    let pane_left_edge = pane_geom.x;
    let pane_bottom_edge = pane_geom.y + pane_geom.rows.as_usize().saturating_sub(1);
    let pane_right_edge = pane_geom.x + pane_geom.cols.as_usize().saturating_sub(1);
    let visible_cells = fragment
        .render
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
        vec![ImageFragment::KittyPlaceholder(KittyPlaceholderFragment {
            render: KittyPlaceholderRender {
                cells: visible_cells,
                ..fragment.render.clone()
            },
        })]
    }
}

pub(crate) fn clip_image_fragment(
    pane_geom: &PaneGeom,
    fragment: &ImageFragment,
    character_cell_size: Option<&SizeInPixels>,
) -> Vec<ImageFragment> {
    match fragment {
        ImageFragment::Sixel(sixel_fragment) => {
            if let Some(character_cell_size) = character_cell_size {
                clip_sixel_fragment(pane_geom, sixel_fragment, character_cell_size)
            } else {
                vec![fragment.clone()]
            }
        },
        ImageFragment::KittyExplicit(kitty_explicit_fragment) => {
            clip_kitty_explicit_fragment(pane_geom, kitty_explicit_fragment)
        },
        ImageFragment::KittyPlaceholder(kitty_placeholder_fragment) => {
            clip_kitty_placeholder_fragment(pane_geom, kitty_placeholder_fragment)
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
        let fragments_against_this_pane: Vec<ImageFragment> = fragments_to_check.drain(..).collect();
        for fragment in fragments_against_this_pane {
            let mut unclipped =
                clip_image_fragment(pane_geom, &fragment, character_cell_size);
            fragments_to_check.append(&mut unclipped);
        }
    }
    fragments_to_check
}
