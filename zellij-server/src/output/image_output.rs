use super::{
    CharacterChunk, FloatingPanesStack, HighlightSelection, ImageRenderBundle, KittyImageChunk,
    KittyPlaceholderRender, SixelImageChunk,
};
use crate::{
    panes::kitty::KittyImageState,
    panes::pane_image_scene::KittyRenderBundle,
    panes::terminal_character::{AnsiCode, CharacterStyles},
    panes::{LinkHandler, DEFAULT_STYLES},
    ClientId,
};
use std::fmt::Write;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};
use zellij_utils::errors::prelude::*;
use zellij_utils::pane_size::{PaneGeom, Size, SizeInPixels};

use crate::panes::sixel::SixelImageStore;

fn kitty_delete_all_vte() -> String {
    "\u{1b}_Ga=d,d=A\u{1b}\\".to_string()
}

fn scale_u32(total: u32, kept: usize, original: usize) -> u32 {
    if original == 0 {
        0
    } else {
        ((total as u64 * kept as u64) / original as u64) as u32
    }
}

fn vte_goto_instruction(x_coords: usize, y_coords: usize, vte_output: &mut String) -> Result<()> {
    write!(
        vte_output,
        "\u{1b}[{};{}H\u{1b}[m",
        y_coords + 1,
        x_coords + 1,
    )
    .with_context(|| {
        format!(
            "failed to execute VTE instruction to go to ({}, {})",
            x_coords, y_coords
        )
    })
}

fn adjust_styles_for_possible_selection(
    chunk_selection_and_colors: &[HighlightSelection],
    character_styles: CharacterStyles,
    chunk_y: usize,
    chunk_width: usize,
) -> CharacterStyles {
    chunk_selection_and_colors
        .iter()
        .find(|hs| hs.selection.contains(chunk_y, chunk_width))
        .map(|hs| {
            let mut styles = character_styles;
            if let Some(bg) = hs.bg {
                styles = styles.background(Some(bg));
            }
            if let Some(fg) = hs.fg {
                styles = styles.foreground(Some(fg));
            }
            if hs.bold {
                styles = styles.bold(Some(AnsiCode::On));
            }
            if hs.italic {
                styles = styles.italic(Some(AnsiCode::On));
            }
            if hs.underline {
                styles = styles.underline(Some(AnsiCode::Underline(None)));
            }
            styles
        })
        .unwrap_or(character_styles)
}

fn adjust_styles_for_custom_bg_fg(
    character_styles: CharacterStyles,
    pane_default_fg: Option<AnsiCode>,
    pane_default_bg: Option<AnsiCode>,
) -> CharacterStyles {
    let mut character_styles = character_styles;
    if character_styles.foreground.is_none() || character_styles.foreground == Some(AnsiCode::Reset)
    {
        if let Some(fg) = pane_default_fg {
            character_styles.foreground = Some(fg);
        }
    }
    if character_styles.background.is_none() || character_styles.background == Some(AnsiCode::Reset)
    {
        if let Some(bg) = pane_default_bg {
            character_styles.background = Some(bg);
        }
    }
    character_styles
}

fn write_changed_styles(
    character_styles: &mut CharacterStyles,
    current_character_styles: CharacterStyles,
    chunk_changed_colors: Option<[Option<AnsiCode>; 256]>,
    link_handler: Option<&std::cell::Ref<LinkHandler>>,
    osc8_hyperlinks: bool,
    vte_output: &mut String,
) -> Result<()> {
    let err_context = "failed to format changed styles to VTE string";

    if let Some(new_styles) =
        character_styles.update_and_return_diff(&current_character_styles, chunk_changed_colors)
    {
        if osc8_hyperlinks {
            if let Some(osc8_link) =
                link_handler.and_then(|l_h| l_h.output_osc8(new_styles.link_anchor))
            {
                write!(vte_output, "{}{}", new_styles, osc8_link).context(err_context)?;
            } else {
                write!(vte_output, "{}", new_styles).context(err_context)?;
            }
        } else {
            write!(vte_output, "{}", new_styles).context(err_context)?;
        }
    }
    Ok(())
}

fn serialize_chunks(
    character_chunks: Vec<CharacterChunk>,
    sixel_chunks: Option<&Vec<SixelImageChunk>>,
    kitty_chunks: Option<&Vec<KittyImageChunk>>,
    kitty_placeholder_renders: Option<&Vec<KittyPlaceholderRender>>,
    link_handler: Option<&mut Rc<RefCell<LinkHandler>>>,
    sixel_image_store: Option<&mut SixelImageStore>,
    styled_underlines: bool,
    osc8_hyperlinks: bool,
    max_size: Option<Size>,
) -> Result<String> {
    let err_context = || "failed to serialize input chunks".to_string();

    let mut vte_output = String::new();
    let mut sixel_vte: Option<String> = None;
    let link_handler = link_handler.map(|l_h| l_h.borrow());
    for character_chunk in character_chunks {
        if let Some(size) = max_size {
            if character_chunk.y >= size.rows || character_chunk.x >= size.cols {
                continue;
            }
        }

        let chunk_changed_colors = character_chunk.changed_colors();
        let pane_default_fg = character_chunk.pane_default_fg;
        let pane_default_bg = character_chunk.pane_default_bg;
        let mut character_styles = DEFAULT_STYLES.enable_styled_underlines(styled_underlines);
        vte_goto_instruction(character_chunk.x, character_chunk.y, &mut vte_output)
            .with_context(err_context)?;
        let mut chunk_width = character_chunk.x;
        for t_character in character_chunk.terminal_characters.iter() {
            if let Some(size) = max_size {
                if chunk_width + t_character.width() > size.cols {
                    break;
                }
            }

            let current_character_styles = adjust_styles_for_custom_bg_fg(
                adjust_styles_for_possible_selection(
                    character_chunk.selection_and_colors(),
                    *t_character.styles,
                    character_chunk.y,
                    chunk_width,
                ),
                pane_default_fg,
                pane_default_bg,
            );
            write_changed_styles(
                &mut character_styles,
                current_character_styles,
                chunk_changed_colors,
                link_handler.as_ref(),
                osc8_hyperlinks,
                &mut vte_output,
            )
            .with_context(err_context)?;
            chunk_width += t_character.width();
            vte_output.push(t_character.character);
        }
    }
    if let Some(sixel_image_store) = sixel_image_store {
        if let Some(sixel_chunks) = sixel_chunks {
            for sixel_chunk in sixel_chunks {
                if let Some(size) = max_size {
                    if sixel_chunk.cell_y >= size.rows || sixel_chunk.cell_x >= size.cols {
                        continue;
                    }
                }

                let serialized_sixel_image = sixel_image_store.serialize_image(
                    sixel_chunk.sixel_image_id,
                    sixel_chunk.sixel_image_pixel_x,
                    sixel_chunk.sixel_image_pixel_y,
                    sixel_chunk.sixel_image_pixel_width,
                    sixel_chunk.sixel_image_pixel_height,
                );
                if let Some(serialized_sixel_image) = serialized_sixel_image {
                    let sixel_vte = sixel_vte.get_or_insert_with(String::new);
                    vte_goto_instruction(sixel_chunk.cell_x, sixel_chunk.cell_y, sixel_vte)
                        .with_context(err_context)?;
                    sixel_vte.push_str(&serialized_sixel_image);
                }
            }
        }
    }
    if let Some(ref sixel_vte) = sixel_vte {
        vte_output.push_str("\u{1b}[s");
        vte_output.push_str(sixel_vte);
        vte_output.push_str("\u{1b}[u");
    }
    if let Some(kitty_chunks) = kitty_chunks {
        vte_output.push_str(&KittyImageState::serialize_chunks(kitty_chunks));
    }
    if let Some(kitty_placeholder_renders) = kitty_placeholder_renders {
        vte_output.push_str(&KittyImageState::serialize_placeholder_renders(
            kitty_placeholder_renders,
        ));
    }
    Ok(vte_output)
}

impl FloatingPanesStack {
    pub fn visible_sixel_image_chunks(
        &self,
        mut sixel_image_chunks: Vec<SixelImageChunk>,
        z_index: Option<usize>,
        character_cell_size: &SizeInPixels,
    ) -> Vec<SixelImageChunk> {
        let z_index = z_index.unwrap_or(0);
        let mut chunks_to_check: Vec<SixelImageChunk> = sixel_image_chunks.drain(..).collect();
        let panes_to_check = self.layers.iter().skip(z_index);
        for pane_geom in panes_to_check {
            let chunks_to_check_against_this_pane: Vec<SixelImageChunk> =
                chunks_to_check.drain(..).collect();
            for s_chunk in chunks_to_check_against_this_pane {
                let mut uncovered_chunks =
                    self.remove_covered_sixel_parts(pane_geom, &s_chunk, character_cell_size);
                chunks_to_check.append(&mut uncovered_chunks);
            }
        }
        chunks_to_check
    }
    pub(super) fn remove_covered_parts(
        &self,
        pane_geom: &PaneGeom,
        c_chunk: &mut CharacterChunk,
    ) -> Result<Option<CharacterChunk>> {
        let err_context = || {
            format!(
                "failed to remove covered parts from floating panes: {:#?}",
                self
            )
        };

        let pane_top_edge = pane_geom.y;
        let pane_left_edge = pane_geom.x;
        let pane_bottom_edge = pane_geom.y + pane_geom.rows.as_usize().saturating_sub(1);
        let pane_right_edge = pane_geom.x + pane_geom.cols.as_usize().saturating_sub(1);
        let c_chunk_left_side = c_chunk.x;
        let c_chunk_right_side = c_chunk.x + (c_chunk.width()).saturating_sub(1);
        if pane_top_edge <= c_chunk.y && pane_bottom_edge >= c_chunk.y {
            if pane_left_edge <= c_chunk_left_side && pane_right_edge >= c_chunk_right_side {
                // pane covers chunk completely
                drop(c_chunk.terminal_characters.drain(..));
                return Ok(None);
            } else if pane_right_edge >= c_chunk_left_side
                && pane_right_edge < c_chunk_right_side
                && pane_left_edge <= c_chunk_left_side
            {
                // pane covers chunk partially to the left
                let covered_part = c_chunk.drain_by_width(pane_right_edge + 1 - c_chunk_left_side);
                drop(covered_part);
                c_chunk.x = pane_right_edge + 1;
                return Ok(None);
            } else if pane_left_edge >= c_chunk_left_side
                && pane_left_edge >= c_chunk_left_side
                && pane_right_edge >= c_chunk_right_side
            {
                // pane covers chunk partially to the right
                c_chunk.retain_by_width(pane_left_edge - c_chunk_left_side);
                return Ok(None);
            } else if pane_left_edge >= c_chunk_left_side && pane_right_edge <= c_chunk_right_side {
                // pane covers chunk middle
                let (left_chunk_characters, right_chunk_characters) = c_chunk
                    .cut_middle_out(
                        pane_left_edge - c_chunk_left_side,
                        (pane_right_edge + 1) - c_chunk_left_side,
                    )
                    .with_context(err_context)?;
                let left_chunk_x = c_chunk_left_side;
                let right_chunk_x = pane_right_edge + 1;
                let mut left_chunk =
                    CharacterChunk::new(left_chunk_characters, left_chunk_x, c_chunk.y);
                left_chunk.pane_default_fg = c_chunk.pane_default_fg;
                left_chunk.pane_default_bg = c_chunk.pane_default_bg;
                left_chunk.changed_colors = c_chunk.changed_colors;
                if !c_chunk.selection_and_colors.is_empty() {
                    left_chunk.selection_and_colors = c_chunk.selection_and_colors.clone();
                }

                c_chunk.x = right_chunk_x;
                c_chunk.terminal_characters = right_chunk_characters;
                return Ok(Some(left_chunk));
            }
        };
        Ok(None)
    }
    fn remove_covered_sixel_parts(
        &self,
        pane_geom: &PaneGeom,
        s_chunk: &SixelImageChunk,
        character_cell_size: &SizeInPixels,
    ) -> Vec<SixelImageChunk> {
        // round these up to the nearest cell edge
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
            // pane covers image bottom
            let top_image_chunk = SixelImageChunk {
                cell_x: s_chunk.cell_x,
                cell_y: s_chunk.cell_y,
                sixel_image_pixel_x: s_chunk.sixel_image_pixel_x,
                sixel_image_pixel_y: s_chunk.sixel_image_pixel_y,
                sixel_image_pixel_width: rounded_sixel_image_pixel_width,
                sixel_image_pixel_height: pane_top_edge - s_chunk_top_edge,
                sixel_image_id: s_chunk.sixel_image_id,
            };
            uncovered_chunks.push(top_image_chunk);
        }
        if pane_bottom_edge <= s_chunk_bottom_edge
            && pane_bottom_edge >= s_chunk_top_edge
            && pane_intersects_with_chunk_vertically
        {
            // pane covers image top
            let bottom_image_chunk = SixelImageChunk {
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
            };
            uncovered_chunks.push(bottom_image_chunk);
        }
        if pane_left_edge >= s_chunk_left_edge
            && pane_left_edge <= s_chunk_right_edge
            && pane_intersects_with_chunk_horizontally
        {
            // pane covers image right
            let sixel_image_pixel_y = if s_chunk_top_edge < pane_top_edge {
                s_chunk.sixel_image_pixel_y + (pane_top_edge - s_chunk_top_edge)
            } else {
                s_chunk.sixel_image_pixel_y
            };
            let max_image_height = if s_chunk_top_edge < pane_top_edge {
                rounded_sixel_image_pixel_height.saturating_sub(pane_top_edge - s_chunk_top_edge)
            } else {
                rounded_sixel_image_pixel_height
            };
            let left_image_chunk = SixelImageChunk {
                cell_x: s_chunk.cell_x,
                // if the pane_top_edge is lower than the image, we want to start there, because we
                // already cut that part above when checking if the pane covered the chunk bottom
                cell_y: std::cmp::max(s_chunk.cell_y, pane_top_edge / character_cell_size.height),
                sixel_image_pixel_x: s_chunk.sixel_image_pixel_x,
                sixel_image_pixel_y,
                sixel_image_pixel_width: rounded_sixel_image_pixel_width
                    .saturating_sub(s_chunk_right_edge.saturating_sub(pane_left_edge)),
                sixel_image_pixel_height: std::cmp::min(
                    pane_bottom_edge - pane_top_edge + character_cell_size.height,
                    max_image_height,
                ),
                sixel_image_id: s_chunk.sixel_image_id,
            };
            uncovered_chunks.push(left_image_chunk);
        }
        if pane_right_edge <= s_chunk_right_edge
            && pane_right_edge >= s_chunk_left_edge
            && pane_intersects_with_chunk_horizontally
        {
            // pane covers image left
            let sixel_image_pixel_y = if s_chunk_top_edge < pane_top_edge {
                s_chunk.sixel_image_pixel_y + (pane_top_edge - s_chunk_top_edge)
            } else {
                s_chunk.sixel_image_pixel_y
            };
            let max_image_height = if s_chunk_top_edge < pane_top_edge {
                rounded_sixel_image_pixel_height.saturating_sub(pane_top_edge - s_chunk_top_edge)
            } else {
                rounded_sixel_image_pixel_height
            };
            let sixel_image_pixel_x = s_chunk.sixel_image_pixel_x
                + (pane_right_edge - s_chunk_left_edge)
                + character_cell_size.width;
            let right_image_chunk = SixelImageChunk {
                cell_x: (pane_right_edge / character_cell_size.width) + 1,
                // if the pane_top_edge is lower than the image, we want to start there, because we
                // already cut that part above when checking if the pane covered the chunk bottom
                cell_y: std::cmp::max(s_chunk.cell_y, pane_top_edge / character_cell_size.height),
                sixel_image_pixel_x,
                sixel_image_pixel_y,
                sixel_image_pixel_width: (rounded_sixel_image_pixel_width
                    .saturating_sub(pane_right_edge - s_chunk_left_edge))
                .saturating_sub(character_cell_size.width),
                sixel_image_pixel_height: std::cmp::min(
                    pane_bottom_edge - pane_top_edge + character_cell_size.height,
                    max_image_height,
                ),
                sixel_image_id: s_chunk.sixel_image_id,
            };
            uncovered_chunks.push(right_image_chunk);
        }
        if uncovered_chunks.is_empty() {
            // the pane doesn't cover the chunk at all, so we return it as is
            uncovered_chunks.push(*s_chunk);
        }
        uncovered_chunks
    }
    pub fn visible_kitty_image_chunks(
        &self,
        mut kitty_image_chunks: Vec<KittyImageChunk>,
        z_index: Option<usize>,
    ) -> Vec<KittyImageChunk> {
        let z_index = z_index.unwrap_or(0);
        let mut chunks_to_check: Vec<KittyImageChunk> = kitty_image_chunks.drain(..).collect();
        let panes_to_check = self.layers.iter().skip(z_index);
        for pane_geom in panes_to_check {
            let chunks_against_this_pane: Vec<KittyImageChunk> =
                chunks_to_check.drain(..).collect();
            for k_chunk in chunks_against_this_pane {
                let mut uncovered_chunks = self.remove_covered_kitty_parts(pane_geom, &k_chunk);
                chunks_to_check.append(&mut uncovered_chunks);
            }
        }
        chunks_to_check
    }
    fn remove_covered_kitty_parts(
        &self,
        pane_geom: &PaneGeom,
        k_chunk: &KittyImageChunk,
    ) -> Vec<KittyImageChunk> {
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
            uncovered.push(KittyImageChunk {
                rows: kept_rows,
                source_height: scale_u32(k_chunk.source_height, kept_rows, k_chunk.rows),
                ..k_chunk.clone()
            });
        }
        if pane_bottom_edge >= chunk_top_edge
            && pane_bottom_edge < chunk_bottom_edge
            && intersects_vertically
        {
            let removed_rows = pane_bottom_edge + 1 - chunk_top_edge;
            let kept_rows = k_chunk.rows.saturating_sub(removed_rows);
            uncovered.push(KittyImageChunk {
                cell_y: pane_bottom_edge + 1,
                rows: kept_rows,
                source_y: k_chunk.source_y
                    + scale_u32(k_chunk.source_height, removed_rows, k_chunk.rows),
                source_height: scale_u32(k_chunk.source_height, kept_rows, k_chunk.rows),
                ..k_chunk.clone()
            });
        }
        if pane_left_edge > chunk_left_edge
            && pane_left_edge <= chunk_right_edge
            && intersects_horizontally
        {
            let kept_cols = pane_left_edge - chunk_left_edge;
            uncovered.push(KittyImageChunk {
                columns: kept_cols,
                source_width: scale_u32(k_chunk.source_width, kept_cols, k_chunk.columns),
                ..k_chunk.clone()
            });
        }
        if pane_right_edge >= chunk_left_edge
            && pane_right_edge < chunk_right_edge
            && intersects_horizontally
        {
            let removed_cols = pane_right_edge + 1 - chunk_left_edge;
            let kept_cols = k_chunk.columns.saturating_sub(removed_cols);
            uncovered.push(KittyImageChunk {
                cell_x: pane_right_edge + 1,
                columns: kept_cols,
                source_x: k_chunk.source_x
                    + scale_u32(k_chunk.source_width, removed_cols, k_chunk.columns),
                source_width: scale_u32(k_chunk.source_width, kept_cols, k_chunk.columns),
                ..k_chunk.clone()
            });
        }
        if uncovered.is_empty() {
            uncovered.push(k_chunk.clone());
        }
        uncovered
    }
}

#[derive(Clone, Default)]
pub(crate) struct ImageOutput {
    sixel_chunks: HashMap<ClientId, Vec<SixelImageChunk>>,
    kitty_chunks: HashMap<ClientId, Vec<KittyImageChunk>>,
    kitty_placeholder_renders: HashMap<ClientId, Vec<KittyPlaceholderRender>>,
    // Kitty owns a persistent composed scene, but some frames still need a same-frame
    // restoration pass after text/frame damage. These maps hold only that per-frame
    // redraw subset; they do not represent the full visible kitty scene.
    kitty_damage_redraw_chunks: HashMap<ClientId, Vec<KittyImageChunk>>,
    kitty_damage_redraw_placeholder_renders: HashMap<ClientId, Vec<KittyPlaceholderRender>>,
    last_rendered_kitty_chunks: HashMap<ClientId, Vec<KittyImageChunk>>,
    last_rendered_kitty_placeholder_renders: HashMap<ClientId, Vec<KittyPlaceholderRender>>,
    sixel_image_store: Rc<RefCell<SixelImageStore>>,
    character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
    floating_panes_stack: Option<FloatingPanesStack>,
}

struct ClientImageOutput {
    sixel_chunks: Vec<SixelImageChunk>,
    kitty_chunks_to_serialize: Vec<KittyImageChunk>,
    kitty_placeholder_renders_to_serialize: Vec<KittyPlaceholderRender>,
    current_kitty_chunks: Vec<KittyImageChunk>,
    current_kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
    kitty_scene_changed: bool,
}

impl ImageOutput {
    pub fn new(
        sixel_image_store: Rc<RefCell<SixelImageStore>>,
        character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
    ) -> Self {
        Self {
            sixel_image_store,
            character_cell_size,
            ..Default::default()
        }
    }

    fn kitty_scene_is_dirty(&self) -> bool {
        let mut client_ids = HashSet::new();
        client_ids.extend(self.kitty_chunks.keys().copied());
        client_ids.extend(self.kitty_placeholder_renders.keys().copied());
        client_ids.extend(self.last_rendered_kitty_chunks.keys().copied());
        client_ids.extend(self.last_rendered_kitty_placeholder_renders.keys().copied());
        client_ids.into_iter().any(|client_id| {
            self.kitty_chunks
                .get(&client_id)
                .cloned()
                .unwrap_or_default()
                != self
                    .last_rendered_kitty_chunks
                    .get(&client_id)
                    .cloned()
                    .unwrap_or_default()
                || self
                    .kitty_placeholder_renders
                    .get(&client_id)
                    .cloned()
                    .unwrap_or_default()
                    != self
                        .last_rendered_kitty_placeholder_renders
                        .get(&client_id)
                        .cloned()
                        .unwrap_or_default()
        })
    }

    pub fn set_floating_panes_stack(&mut self, floating_panes_stack: Option<FloatingPanesStack>) {
        self.floating_panes_stack = floating_panes_stack;
    }

    pub fn set_last_rendered_kitty_chunks(
        &mut self,
        last_rendered_kitty_chunks: HashMap<ClientId, Vec<KittyImageChunk>>,
        last_rendered_kitty_placeholder_renders: HashMap<ClientId, Vec<KittyPlaceholderRender>>,
    ) {
        self.last_rendered_kitty_chunks = last_rendered_kitty_chunks;
        self.last_rendered_kitty_placeholder_renders = last_rendered_kitty_placeholder_renders;
    }

    pub fn take_last_rendered_kitty_chunks(
        &mut self,
    ) -> (
        HashMap<ClientId, Vec<KittyImageChunk>>,
        HashMap<ClientId, Vec<KittyPlaceholderRender>>,
    ) {
        (
            std::mem::take(&mut self.last_rendered_kitty_chunks),
            std::mem::take(&mut self.last_rendered_kitty_placeholder_renders),
        )
    }

    fn add_sixel_image_chunks_to_client(
        &mut self,
        client_id: ClientId,
        sixel_image_chunks: Vec<SixelImageChunk>,
        z_index: Option<usize>,
    ) {
        if let Some(character_cell_size) = *self.character_cell_size.borrow() {
            let mut sixel_chunks = if let Some(floating_panes_stack) = &self.floating_panes_stack {
                floating_panes_stack.visible_sixel_image_chunks(
                    sixel_image_chunks,
                    z_index,
                    &character_cell_size,
                )
            } else {
                sixel_image_chunks
            };
            let entry = self.sixel_chunks.entry(client_id).or_default();
            entry.append(&mut sixel_chunks);
        }
    }

    fn add_sixel_image_chunks_to_multiple_clients(
        &mut self,
        sixel_image_chunks: Vec<SixelImageChunk>,
        client_ids: impl Iterator<Item = ClientId>,
        z_index: Option<usize>,
    ) {
        if let Some(character_cell_size) = *self.character_cell_size.borrow() {
            let sixel_chunks = if let Some(floating_panes_stack) = &self.floating_panes_stack {
                floating_panes_stack.visible_sixel_image_chunks(
                    sixel_image_chunks,
                    z_index,
                    &character_cell_size,
                )
            } else {
                sixel_image_chunks
            };
            for client_id in client_ids {
                let entry = self.sixel_chunks.entry(client_id).or_default();
                entry.append(&mut sixel_chunks.clone());
            }
        }
    }

    fn add_kitty_image_chunks_to_client(
        &mut self,
        client_id: ClientId,
        kitty_image_chunks: Vec<KittyImageChunk>,
        z_index: Option<usize>,
    ) {
        let mut kitty_chunks = if let Some(floating_panes_stack) = &self.floating_panes_stack {
            floating_panes_stack.visible_kitty_image_chunks(kitty_image_chunks, z_index)
        } else {
            kitty_image_chunks
        };
        let entry = self.kitty_chunks.entry(client_id).or_default();
        entry.append(&mut kitty_chunks);
    }

    fn add_kitty_image_chunks_to_multiple_clients(
        &mut self,
        kitty_image_chunks: Vec<KittyImageChunk>,
        client_ids: impl Iterator<Item = ClientId>,
        z_index: Option<usize>,
    ) {
        let kitty_chunks = if let Some(floating_panes_stack) = &self.floating_panes_stack {
            floating_panes_stack.visible_kitty_image_chunks(kitty_image_chunks, z_index)
        } else {
            kitty_image_chunks
        };
        for client_id in client_ids {
            let entry = self.kitty_chunks.entry(client_id).or_default();
            entry.append(&mut kitty_chunks.clone());
        }
    }

    fn add_kitty_placeholder_renders_to_client(
        &mut self,
        client_id: ClientId,
        kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
    ) {
        let entry = self.kitty_placeholder_renders.entry(client_id).or_default();
        entry.extend(kitty_placeholder_renders);
    }

    fn add_kitty_placeholder_renders_to_multiple_clients(
        &mut self,
        kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
        client_ids: impl Iterator<Item = ClientId>,
    ) {
        for client_id in client_ids {
            let entry = self.kitty_placeholder_renders.entry(client_id).or_default();
            entry.extend(kitty_placeholder_renders.clone());
        }
    }

    fn add_kitty_render_bundle_to_client(
        &mut self,
        client_id: ClientId,
        kitty_render_bundle: KittyRenderBundle,
        z_index: Option<usize>,
    ) {
        self.add_kitty_image_chunks_to_client(
            client_id,
            kitty_render_bundle.explicit_chunks,
            z_index,
        );
        self.add_kitty_placeholder_renders_to_client(
            client_id,
            kitty_render_bundle.placeholder_renders,
        );
    }

    fn add_kitty_render_bundle_to_multiple_clients(
        &mut self,
        kitty_render_bundle: KittyRenderBundle,
        client_ids: impl Iterator<Item = ClientId>,
        z_index: Option<usize>,
    ) {
        let client_ids: Vec<ClientId> = client_ids.collect();
        self.add_kitty_image_chunks_to_multiple_clients(
            kitty_render_bundle.explicit_chunks,
            client_ids.iter().copied(),
            z_index,
        );
        self.add_kitty_placeholder_renders_to_multiple_clients(
            kitty_render_bundle.placeholder_renders,
            client_ids.iter().copied(),
        );
    }

    pub fn add_damage_redraw_image_render_bundle_to_client(
        &mut self,
        client_id: ClientId,
        image_render_bundle: ImageRenderBundle,
        z_index: Option<usize>,
    ) {
        self.add_sixel_image_chunks_to_client(client_id, image_render_bundle.sixel_chunks, z_index);
        self.kitty_damage_redraw_chunks
            .entry(client_id)
            .or_default()
            .extend(image_render_bundle.kitty_render_bundle.explicit_chunks);
        self.kitty_damage_redraw_placeholder_renders
            .entry(client_id)
            .or_default()
            .extend(image_render_bundle.kitty_render_bundle.placeholder_renders);
    }

    pub fn add_damage_redraw_image_render_bundle_to_multiple_clients(
        &mut self,
        image_render_bundle: ImageRenderBundle,
        client_ids: impl Iterator<Item = ClientId>,
        z_index: Option<usize>,
    ) {
        let client_ids: Vec<ClientId> = client_ids.collect();
        self.add_sixel_image_chunks_to_multiple_clients(
            image_render_bundle.sixel_chunks,
            client_ids.iter().copied(),
            z_index,
        );
        for client_id in client_ids {
            self.kitty_damage_redraw_chunks
                .entry(client_id)
                .or_default()
                .extend(
                    image_render_bundle
                        .kitty_render_bundle
                        .explicit_chunks
                        .clone(),
                );
            self.kitty_damage_redraw_placeholder_renders
                .entry(client_id)
                .or_default()
                .extend(
                    image_render_bundle
                        .kitty_render_bundle
                        .placeholder_renders
                        .clone(),
                );
        }
    }

    pub fn add_image_render_bundle_to_client(
        &mut self,
        client_id: ClientId,
        image_render_bundle: ImageRenderBundle,
        z_index: Option<usize>,
    ) {
        self.add_sixel_image_chunks_to_client(client_id, image_render_bundle.sixel_chunks, z_index);
        self.add_kitty_render_bundle_to_client(
            client_id,
            image_render_bundle.kitty_render_bundle,
            z_index,
        );
    }

    pub fn add_image_render_bundle_to_multiple_clients(
        &mut self,
        image_render_bundle: ImageRenderBundle,
        client_ids: impl Iterator<Item = ClientId>,
        z_index: Option<usize>,
    ) {
        let client_ids: Vec<ClientId> = client_ids.collect();
        self.add_sixel_image_chunks_to_multiple_clients(
            image_render_bundle.sixel_chunks,
            client_ids.iter().copied(),
            z_index,
        );
        self.add_kitty_render_bundle_to_multiple_clients(
            image_render_bundle.kitty_render_bundle,
            client_ids.iter().copied(),
            z_index,
        );
    }

    fn take_client_output_for_serialization(
        &mut self,
        client_id: ClientId,
        pre_vte_clears_display: bool,
    ) -> ClientImageOutput {
        let sixel_chunks = self.sixel_chunks.remove(&client_id).unwrap_or_default();
        let current_kitty_chunks = self.kitty_chunks.remove(&client_id).unwrap_or_default();
        let current_kitty_placeholder_renders = self
            .kitty_placeholder_renders
            .remove(&client_id)
            .unwrap_or_default();
        let kitty_damage_redraw_chunks = self
            .kitty_damage_redraw_chunks
            .remove(&client_id)
            .unwrap_or_default();
        let kitty_damage_redraw_placeholder_renders = self
            .kitty_damage_redraw_placeholder_renders
            .remove(&client_id)
            .unwrap_or_default();

        let previous_kitty_chunks = if pre_vte_clears_display {
            vec![]
        } else {
            self.last_rendered_kitty_chunks
                .get(&client_id)
                .cloned()
                .unwrap_or_default()
        };
        let previous_kitty_placeholder_renders = if pre_vte_clears_display {
            vec![]
        } else {
            self.last_rendered_kitty_placeholder_renders
                .get(&client_id)
                .cloned()
                .unwrap_or_default()
        };
        let kitty_scene_changed = previous_kitty_chunks != current_kitty_chunks
            || previous_kitty_placeholder_renders != current_kitty_placeholder_renders;
        let kitty_chunks_to_serialize = if kitty_scene_changed {
            current_kitty_chunks.clone()
        } else {
            kitty_damage_redraw_chunks
        };
        let kitty_placeholder_renders_to_serialize = if kitty_scene_changed {
            current_kitty_placeholder_renders.clone()
        } else {
            kitty_damage_redraw_placeholder_renders
        };

        ClientImageOutput {
            sixel_chunks,
            kitty_chunks_to_serialize,
            kitty_placeholder_renders_to_serialize,
            current_kitty_chunks,
            current_kitty_placeholder_renders,
            kitty_scene_changed,
        }
    }

    fn finish_client_frame(
        &mut self,
        client_id: ClientId,
        current_kitty_chunks: Vec<KittyImageChunk>,
        current_kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
    ) {
        self.last_rendered_kitty_chunks
            .insert(client_id, current_kitty_chunks);
        self.last_rendered_kitty_placeholder_renders
            .insert(client_id, current_kitty_placeholder_renders);
    }

    pub fn is_dirty(&self) -> bool {
        self.sixel_chunks.values().any(|c| !c.is_empty())
            || self.kitty_chunks.values().any(|c| !c.is_empty())
            || self
                .kitty_placeholder_renders
                .values()
                .any(|c| !c.is_empty())
            || self
                .kitty_damage_redraw_chunks
                .values()
                .any(|c| !c.is_empty())
            || self
                .kitty_damage_redraw_placeholder_renders
                .values()
                .any(|c| !c.is_empty())
            || self.kitty_scene_is_dirty()
    }

    pub fn has_rendered_assets(&self) -> bool {
        self.sixel_chunks.values().any(|c| !c.is_empty())
            || self.kitty_chunks.values().any(|c| !c.is_empty())
            || self
                .kitty_placeholder_renders
                .values()
                .any(|c| !c.is_empty())
            || self
                .kitty_damage_redraw_chunks
                .values()
                .any(|c| !c.is_empty())
            || self
                .kitty_damage_redraw_placeholder_renders
                .values()
                .any(|c| !c.is_empty())
    }

    fn with_sixel_image_store<T>(&mut self, f: impl FnOnce(&mut SixelImageStore) -> T) -> T {
        f(&mut self.sixel_image_store.borrow_mut())
    }

    pub fn serialize_render_body_for_client(
        &mut self,
        client_id: ClientId,
        pre_vte_clears_display: bool,
        character_chunks: Vec<CharacterChunk>,
        link_handler: Option<&mut Rc<RefCell<LinkHandler>>>,
        styled_underlines: bool,
        osc8_hyperlinks: bool,
        max_size: Option<Size>,
    ) -> Result<String> {
        let image_output =
            self.take_client_output_for_serialization(client_id, pre_vte_clears_display);
        let mut serialized = String::new();
        if image_output.kitty_scene_changed {
            serialized.push_str("\u{1b}[s");
            serialized.push_str(&kitty_delete_all_vte());
            serialized.push_str("\u{1b}[u");
        }
        serialized.push_str(&self.with_sixel_image_store(|sixel_image_store| {
            serialize_chunks(
                character_chunks,
                (!image_output.sixel_chunks.is_empty()).then_some(&image_output.sixel_chunks),
                (!image_output.kitty_chunks_to_serialize.is_empty())
                    .then_some(&image_output.kitty_chunks_to_serialize),
                (!image_output
                    .kitty_placeholder_renders_to_serialize
                    .is_empty())
                .then_some(&image_output.kitty_placeholder_renders_to_serialize),
                link_handler,
                Some(sixel_image_store),
                styled_underlines,
                osc8_hyperlinks,
                max_size,
            )
        })?);
        self.finish_client_frame(
            client_id,
            image_output.current_kitty_chunks,
            image_output.current_kitty_placeholder_renders,
        );
        Ok(serialized)
    }
}
