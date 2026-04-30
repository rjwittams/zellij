use std::collections::VecDeque;

mod image_fragment;
mod image_output;
mod kitty_diff;
mod kitty_output_media;

use crate::panes::Row;

use crate::panes::Selection;
use crate::{
    panes::kitty_asset_store::KittyAssetStore,
    panes::sixel::SixelImageStore,
    panes::terminal_character::{AnsiCode, CharacterStyles},
    panes::{LinkHandler, PaneId, TerminalCharacter, DEFAULT_STYLES, EMPTY_TERMINAL_CHARACTER},
    ClientId,
};
use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;
use std::{
    collections::{HashMap, HashSet},
    str,
};
use zellij_utils::data::{HighlightLayer, PaneContents, PaneRenderReport};
use zellij_utils::errors::prelude::*;
use zellij_utils::pane_size::SizeInPixels;

use self::image_fragment::PreparedImageOutput;
use self::image_output::ImageOutput;
pub use self::kitty_output_media::{KittyOutputMediaCache, KittyOutputMediaRetention};
use crate::panes::pane_image_scene::KittyRenderBundle;
use zellij_utils::pane_size::{PaneGeom, Size};

fn vte_goto_instruction(x_coords: usize, y_coords: usize, vte_output: &mut String) -> Result<()> {
    write!(
        vte_output,
        "\u{1b}[{};{}H\u{1b}[m",
        y_coords + 1, // + 1 because VTE is 1 indexed
        x_coords + 1,
    )
    .with_context(|| {
        format!(
            "failed to execute VTE instruction to go to ({}, {})",
            x_coords, y_coords
        )
    })
}

fn vte_hide_cursor_instruction(vte_output: &mut String) -> Result<()> {
    write!(vte_output, "\u{1b}[?25l").context("failed to execute VTE instruction to hide cursor")
}

/// A selection region with associated styling for highlights and text selection.
#[derive(Debug, Clone, Copy)]
pub struct HighlightSelection {
    pub selection: Selection,
    pub bg: Option<AnsiCode>,
    pub fg: Option<AnsiCode>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub layer: HighlightLayer,
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

fn serialize_chunks_with_newlines(
    character_chunks: Vec<CharacterChunk>,
    _sixel_chunks: Option<&Vec<SixelImageChunk>>, // TODO: fix this sometime
    link_handler: Option<&mut Rc<RefCell<LinkHandler>>>,
    styled_underlines: bool,
    osc8_hyperlinks: bool,
    max_size: Option<Size>,
) -> Result<String> {
    let err_context = || "failed to serialize input chunks".to_string();

    let mut vte_output = String::new();
    let link_handler = link_handler.map(|l_h| l_h.borrow());
    for character_chunk in character_chunks {
        // Skip chunks that are completely outside the size bounds
        if let Some(size) = max_size {
            if character_chunk.y >= size.rows {
                continue; // Chunk is below visible area
            }
            if character_chunk.x >= size.cols {
                continue; // Chunk starts outside visible area
            }
        }

        let chunk_changed_colors = character_chunk.changed_colors();
        let pane_default_fg = character_chunk.pane_default_fg;
        let pane_default_bg = character_chunk.pane_default_bg;
        let mut character_styles = DEFAULT_STYLES.enable_styled_underlines(styled_underlines);
        vte_output.push_str("\n\r");
        let mut chunk_width = character_chunk.x;
        for t_character in character_chunk.terminal_characters.iter() {
            // Stop rendering if the next character would exceed max_size.cols
            if let Some(size) = max_size {
                if chunk_width + t_character.width() > size.cols {
                    break; // Stop rendering this chunk
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
    Ok(vte_output)
}
fn serialize_chunks(
    character_chunks: Vec<CharacterChunk>,
    image_output: &mut ImageOutput,
    prepared_image_output: PreparedImageOutput,
    link_handler: Option<&mut Rc<RefCell<LinkHandler>>>,
    styled_underlines: bool,
    osc8_hyperlinks: bool,
    max_size: Option<Size>,
) -> Result<String> {
    let err_context = || "failed to serialize input chunks".to_string();

    let mut vte_output = String::new();

    if let Some(image_prelude) = prepared_image_output.before_text_vte.as_ref() {
        vte_output.push_str(&image_prelude);
    }

    let link_handler = link_handler.map(|l_h| l_h.borrow());
    for character_chunk in character_chunks {
        // Skip chunks that are completely outside the size bounds
        if let Some(size) = max_size {
            if character_chunk.y >= size.rows {
                continue; // Chunk is below visible area
            }
            if character_chunk.x >= size.cols {
                continue; // Chunk starts outside visible area
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
            // Stop rendering if the next character would exceed max_size.cols
            if let Some(size) = max_size {
                if chunk_width + t_character.width() > size.cols {
                    break; // Stop rendering this chunk
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
    prepared_image_output
        .after_text
        .serialize(image_output, max_size, &mut vte_output)?;
    Ok(vte_output)
}

type AbsoluteMiddleStart = usize;
type AbsoluteMiddleEnd = usize;
type PadLeftEndBy = usize;
type PadRightStartBy = usize;
fn adjust_middle_segment_for_wide_chars(
    middle_start: usize,
    middle_end: usize,
    terminal_characters: &[TerminalCharacter],
) -> Result<(
    AbsoluteMiddleStart,
    AbsoluteMiddleEnd,
    PadLeftEndBy,
    PadRightStartBy,
)> {
    let err_context = || {
        format!(
            "failed to adjust middle segment (from {} to {}) for wide chars: '{:?}'",
            middle_start, middle_end, terminal_characters
        )
    };

    let mut absolute_middle_start_index = None;
    let mut absolute_middle_end_index = None;
    let mut current_x = 0;
    let mut pad_left_end_by = 0;
    let mut pad_right_start_by = 0;
    for (absolute_index, t_character) in terminal_characters.iter().enumerate() {
        current_x += t_character.width();
        if current_x >= middle_start && absolute_middle_start_index.is_none() {
            if current_x > middle_start {
                pad_left_end_by = current_x - middle_start;
                absolute_middle_start_index = Some(absolute_index);
            } else {
                absolute_middle_start_index = Some(absolute_index + 1);
            }
        }
        if current_x >= middle_end && absolute_middle_end_index.is_none() {
            absolute_middle_end_index = Some(absolute_index + 1);
            if current_x > middle_end {
                pad_right_start_by = current_x - middle_end;
            }
        }
    }
    Ok((
        absolute_middle_start_index.with_context(err_context)?,
        absolute_middle_end_index.with_context(err_context)?,
        pad_left_end_by,
        pad_right_start_by,
    ))
}

#[derive(Clone, Debug, Default)]
pub struct Output {
    pre_vte_instructions: HashMap<ClientId, Vec<PreVteInstruction>>,
    post_vte_instructions: HashMap<ClientId, Vec<String>>,
    client_character_chunks: HashMap<ClientId, Vec<CharacterChunk>>,
    link_handler: Option<Rc<RefCell<LinkHandler>>>,
    image_output: ImageOutput,
    floating_panes_stack: Option<FloatingPanesStack>,
    styled_underlines: bool,
    osc8_hyperlinks: bool,
    pane_render_report: PaneRenderReport,
    pub collect_ansi_pane_contents: bool,
    cursor_coordinates: Option<(usize, usize)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderedImageState {
    pub explicit_chunks: Vec<KittyImageChunk>,
    pub placeholder_renders: Vec<KittyPlaceholderRender>,
    pub resident_asset_generations: HashMap<u32, u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LastRenderedImageState {
    rendered_image_state: RenderedImageState,
    kitty_scene_state: Option<kitty_diff::KittySceneState>,
}

impl LastRenderedImageState {
    pub fn new(rendered_image_state: RenderedImageState) -> Self {
        Self {
            rendered_image_state,
            kitty_scene_state: None,
        }
    }

    pub fn rendered_image_state(&self) -> &RenderedImageState {
        &self.rendered_image_state
    }

    pub fn resident_asset_generations(&self) -> &HashMap<u32, u64> {
        self.kitty_scene_state
            .as_ref()
            .map(|scene_state| &scene_state.resident_asset_generations)
            .unwrap_or(&self.rendered_image_state.resident_asset_generations)
    }

    pub(crate) fn with_kitty_scene_state(
        rendered_image_state: RenderedImageState,
        kitty_scene_state: Option<kitty_diff::KittySceneState>,
    ) -> Self {
        Self {
            rendered_image_state,
            kitty_scene_state,
        }
    }

    pub(crate) fn kitty_scene_state(&self) -> Option<&kitty_diff::KittySceneState> {
        self.kitty_scene_state.as_ref()
    }

    pub(crate) fn is_empty(&self) -> bool {
        let scene_has_placements = self
            .kitty_scene_state
            .as_ref()
            .map(|scene_state| !scene_state.placements.is_empty())
            .unwrap_or(false);
        self.rendered_image_state.explicit_chunks.is_empty()
            && self.rendered_image_state.placeholder_renders.is_empty()
            && self.resident_asset_generations().is_empty()
            && !scene_has_placements
    }
}

pub trait IntoLastRenderedImageState {
    fn into_last_rendered_image_state(self) -> Rc<LastRenderedImageState>;
}

impl IntoLastRenderedImageState for Rc<LastRenderedImageState> {
    fn into_last_rendered_image_state(self) -> Rc<LastRenderedImageState> {
        self
    }
}

impl IntoLastRenderedImageState for LastRenderedImageState {
    fn into_last_rendered_image_state(self) -> Rc<LastRenderedImageState> {
        Rc::new(self)
    }
}

impl IntoLastRenderedImageState for RenderedImageState {
    fn into_last_rendered_image_state(self) -> Rc<LastRenderedImageState> {
        Rc::new(LastRenderedImageState::new(self))
    }
}

#[derive(Clone, Debug)]
struct PreVteInstruction {
    bytes: String,
    clears_display: bool,
}

impl Output {
    pub fn new(
        sixel_image_store: Rc<RefCell<SixelImageStore>>,
        kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
        kitty_output_media_cache: Rc<RefCell<KittyOutputMediaCache>>,
        character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
        styled_underlines: bool,
        osc8_hyperlinks: bool,
    ) -> Self {
        Output {
            image_output: ImageOutput::new(
                sixel_image_store,
                kitty_asset_store,
                kitty_output_media_cache,
                character_cell_size,
            ),
            styled_underlines,
            osc8_hyperlinks,
            ..Default::default()
        }
    }

    pub fn set_last_rendered_image_states<T>(
        &mut self,
        last_rendered_image_states: HashMap<ClientId, T>,
    ) where
        T: IntoLastRenderedImageState,
    {
        self.image_output
            .set_last_rendered_image_states(last_rendered_image_states);
    }

    pub fn set_last_rendered_image_state_for_client<T>(
        &mut self,
        client_id: ClientId,
        last_rendered_image_state: T,
    ) where
        T: IntoLastRenderedImageState,
    {
        self.image_output
            .set_last_rendered_image_state_for_client(client_id, last_rendered_image_state);
    }

    pub fn set_kitty_file_output_enabled_for_client(&mut self, client_id: ClientId, enabled: bool) {
        self.image_output
            .set_kitty_file_output_enabled_for_client(client_id, enabled);
    }

    pub fn last_rendered_image_states(&self) -> HashMap<ClientId, Rc<LastRenderedImageState>> {
        self.image_output.last_rendered_image_states()
    }

    pub fn last_rendered_image_state_for_client(
        &self,
        client_id: ClientId,
    ) -> Option<Rc<LastRenderedImageState>> {
        self.image_output
            .last_rendered_image_state_for_client(client_id)
    }

    fn ensure_client_slot(&mut self, client_id: ClientId) {
        self.client_character_chunks.entry(client_id).or_default();
    }

    fn add_pre_vte_instruction(
        &mut self,
        client_id: ClientId,
        vte_instruction: &str,
        clears_display: bool,
    ) {
        self.ensure_client_slot(client_id);
        let entry = self
            .pre_vte_instructions
            .entry(client_id)
            .or_insert_with(Vec::new);
        entry.push(PreVteInstruction {
            bytes: String::from(vte_instruction),
            clears_display,
        });
    }

    pub fn add_clients(
        &mut self,
        client_ids: &HashSet<ClientId>,
        link_handler: Rc<RefCell<LinkHandler>>,
        floating_panes_stack: Option<FloatingPanesStack>,
    ) {
        self.link_handler = Some(link_handler);
        self.floating_panes_stack = floating_panes_stack;
        for client_id in client_ids {
            self.client_character_chunks.insert(*client_id, vec![]);
        }
    }
    pub fn add_character_chunks_to_client(
        &mut self,
        client_id: ClientId,
        mut character_chunks: Vec<CharacterChunk>,
        z_index: Option<usize>,
    ) -> Result<()> {
        if let Some(client_character_chunks) = self.client_character_chunks.get_mut(&client_id) {
            if let Some(floating_panes_stack) = &self.floating_panes_stack {
                let mut visible_character_chunks = floating_panes_stack
                    .visible_character_chunks(character_chunks, z_index)
                    .with_context(|| {
                        format!("failed to add character chunks for client {}", client_id)
                    })?;
                client_character_chunks.append(&mut visible_character_chunks);
            } else {
                client_character_chunks.append(&mut character_chunks);
            }
        }
        Ok(())
    }
    pub fn add_character_chunks_to_multiple_clients(
        &mut self,
        character_chunks: Vec<CharacterChunk>,
        client_ids: impl Iterator<Item = ClientId>,
        z_index: Option<usize>,
    ) -> Result<()> {
        for client_id in client_ids {
            self.add_character_chunks_to_client(client_id, character_chunks.clone(), z_index)
                .context("failed to add character chunks for multiple clients")?;
            // TODO: forgo clone by adding an all_clients thing?
        }
        Ok(())
    }
    pub fn add_post_vte_instruction_to_multiple_clients(
        &mut self,
        client_ids: impl Iterator<Item = ClientId>,
        vte_instruction: &str,
    ) {
        for client_id in client_ids {
            self.ensure_client_slot(client_id);
            let entry = self
                .post_vte_instructions
                .entry(client_id)
                .or_insert_with(Vec::new);
            entry.push(String::from(vte_instruction));
        }
    }
    pub fn add_pre_vte_instruction_to_multiple_clients(
        &mut self,
        client_ids: impl Iterator<Item = ClientId>,
        vte_instruction: &str,
    ) {
        for client_id in client_ids {
            self.add_pre_vte_instruction(client_id, vte_instruction, false);
        }
    }

    pub fn add_display_clearing_pre_vte_instruction_to_multiple_clients(
        &mut self,
        client_ids: impl Iterator<Item = ClientId>,
        vte_instruction: &str,
    ) {
        for client_id in client_ids {
            self.add_pre_vte_instruction(client_id, vte_instruction, true);
        }
    }
    pub fn add_post_vte_instruction_to_client(
        &mut self,
        client_id: ClientId,
        vte_instruction: &str,
    ) {
        self.ensure_client_slot(client_id);
        let entry = self
            .post_vte_instructions
            .entry(client_id)
            .or_insert_with(Vec::new);
        entry.push(String::from(vte_instruction));
    }
    pub fn add_pre_vte_instruction_to_client(
        &mut self,
        client_id: ClientId,
        vte_instruction: &str,
    ) {
        self.add_pre_vte_instruction(client_id, vte_instruction, false);
    }

    pub fn add_display_clearing_pre_vte_instruction_to_client(
        &mut self,
        client_id: ClientId,
        vte_instruction: &str,
    ) {
        self.add_pre_vte_instruction(client_id, vte_instruction, true);
    }
    pub fn add_pane_image_output_to_client(
        &mut self,
        client_id: ClientId,
        pane_image_output: PaneImageRenderOutput,
        z_index: Option<usize>,
    ) {
        self.ensure_client_slot(client_id);
        self.image_output.add_pane_image_output_to_client(
            client_id,
            pane_image_output,
            self.floating_panes_stack.as_ref(),
            z_index,
        );
    }
    pub fn add_pane_image_output_to_multiple_clients(
        &mut self,
        pane_image_output: PaneImageRenderOutput,
        client_ids: impl Iterator<Item = ClientId>,
        z_index: Option<usize>,
    ) {
        self.image_output.add_pane_image_output_to_multiple_clients(
            pane_image_output,
            client_ids,
            self.floating_panes_stack.as_ref(),
            z_index,
        );
    }
    pub fn serialize(&mut self) -> Result<HashMap<ClientId, String>> {
        let err_context = || "failed to serialize output to clients".to_string();

        let mut serialized_render_instructions = HashMap::new();

        for (client_id, client_character_chunks) in self.client_character_chunks.drain() {
            let mut client_serialized_render_instructions = String::new();

            // append pre-vte instructions for this client
            let mut pre_vte_clears_display = false;
            if let Some(pre_vte_instructions_for_client) =
                self.pre_vte_instructions.remove(&client_id)
            {
                for vte_instruction in pre_vte_instructions_for_client {
                    if vte_instruction.clears_display {
                        pre_vte_clears_display = true;
                    }
                    client_serialized_render_instructions.push_str(&vte_instruction.bytes);
                }
            }

            // append the actual text+image output
            let prepared_image_output = self
                .image_output
                .prepare_render_body_for_client(client_id, pre_vte_clears_display);
            client_serialized_render_instructions.push_str(
                &serialize_chunks(
                    client_character_chunks,
                    &mut self.image_output,
                    prepared_image_output,
                    self.link_handler.as_mut(),
                    self.styled_underlines,
                    self.osc8_hyperlinks,
                    None,
                )
                .with_context(err_context)?,
            );

            // append post-vte instructions for this client
            if let Some(post_vte_instructions_for_client) =
                self.post_vte_instructions.remove(&client_id)
            {
                for vte_instruction in post_vte_instructions_for_client {
                    client_serialized_render_instructions.push_str(&vte_instruction);
                }
            }
            serialized_render_instructions.insert(client_id, client_serialized_render_instructions);
        }
        Ok(serialized_render_instructions)
    }
    pub fn serialize_with_size(
        &mut self,
        max_size: Option<Size>,
        content_size: Option<Size>,
    ) -> Result<HashMap<ClientId, String>> {
        let err_context =
            || "failed to serialize output to clients with size constraints".to_string();

        let mut serialized_render_instructions = HashMap::new();

        for (client_id, client_character_chunks) in self.client_character_chunks.drain() {
            let mut client_serialized_render_instructions = String::new();

            // append pre-vte instructions for this client
            let mut pre_vte_clears_display = false;
            if let Some(pre_vte_instructions_for_client) =
                self.pre_vte_instructions.remove(&client_id)
            {
                for vte_instruction in pre_vte_instructions_for_client {
                    if vte_instruction.clears_display {
                        pre_vte_clears_display = true;
                    }
                    client_serialized_render_instructions.push_str(&vte_instruction.bytes);
                }
            }

            // Add padding instructions if max_size is larger than content_size
            if let (Some(max_size), Some(content_size)) = (max_size, content_size) {
                if max_size.rows > content_size.rows || max_size.cols > content_size.cols {
                    // Clear each line from the end of rendered content to the end of the watcher's line
                    for y in 0..content_size.rows {
                        let padding_instruction = format!(
                            "\u{1b}[{};{}H\u{1b}[m\u{1b}[K",
                            y + 1,
                            content_size.cols + 1
                        );
                        client_serialized_render_instructions.push_str(&padding_instruction);
                    }

                    // Clear all content below the last rendered line
                    let clear_below_instruction =
                        format!("\u{1b}[{};{}H\u{1b}[m\u{1b}[J", content_size.rows + 1, 1);
                    client_serialized_render_instructions.push_str(&clear_below_instruction);
                }
            }

            let prepared_image_output = self
                .image_output
                .prepare_render_body_for_client(client_id, pre_vte_clears_display);
            // append the actual vte with size constraints
            client_serialized_render_instructions.push_str(
                &serialize_chunks(
                    client_character_chunks,
                    &mut self.image_output,
                    prepared_image_output,
                    self.link_handler.as_mut(),
                    self.styled_underlines,
                    self.osc8_hyperlinks,
                    max_size,
                )
                .with_context(err_context)?,
            );

            // append post-vte instructions for this client
            if let Some(post_vte_instructions_for_client) =
                self.post_vte_instructions.remove(&client_id)
            {
                for vte_instruction in post_vte_instructions_for_client {
                    client_serialized_render_instructions.push_str(&vte_instruction);
                }
            }

            // Check if cursor was cropped and hide it if necessary
            if let (Some(max_size), Some((cursor_x, cursor_y))) =
                (max_size, self.cursor_coordinates)
            {
                let cursor_was_cropped = cursor_y >= max_size.rows || cursor_x >= max_size.cols;
                if cursor_was_cropped {
                    vte_hide_cursor_instruction(&mut client_serialized_render_instructions)
                        .with_context(err_context)?;
                }
            }

            serialized_render_instructions.insert(client_id, client_serialized_render_instructions);
        }
        Ok(serialized_render_instructions)
    }
    pub fn is_dirty(&self) -> bool {
        !self.pre_vte_instructions.is_empty()
            || !self.post_vte_instructions.is_empty()
            || self.client_character_chunks.values().any(|c| !c.is_empty())
            || self.image_output.is_dirty()
    }
    pub fn has_rendered_assets(&self) -> bool {
        // pre_vte and post_vte are not considered rendered assets as they should not be visible
        self.client_character_chunks.values().any(|c| !c.is_empty())
            || self.image_output.has_rendered_assets()
    }
    pub fn cursor_is_visible(
        &mut self,
        cursor_x: usize,
        cursor_y: usize,
        z_index: Option<usize>,
    ) -> bool {
        self.cursor_coordinates = Some((cursor_x, cursor_y));
        self.floating_panes_stack
            .as_ref()
            .map(|s| s.cursor_is_visible(cursor_x, cursor_y, z_index))
            .unwrap_or(true)
    }
    pub fn add_pane_contents(
        &mut self,
        client_ids: &[ClientId],
        pane_id: PaneId,
        pane_contents: PaneContents,
    ) {
        self.pane_render_report
            .add_pane_contents(client_ids, pane_id.into(), pane_contents);
    }
    pub fn add_pane_contents_with_ansi(
        &mut self,
        client_ids: &[ClientId],
        pane_id: PaneId,
        pane_contents: PaneContents,
    ) {
        self.pane_render_report.add_pane_contents_with_ansi(
            client_ids,
            pane_id.into(),
            pane_contents,
        );
    }
    pub fn drain_pane_render_report(&mut self) -> PaneRenderReport {
        let empty_pane_render_report = PaneRenderReport::default();
        std::mem::replace(&mut self.pane_render_report, empty_pane_render_report)
    }
}

// this struct represents the geometry of a group of floating panes
// we use it to filter out CharacterChunks who are behind these geometries
// and so would not be visible. If a chunk is partially covered, it is adjusted
// to include only the non-covered parts
#[derive(Debug, Clone, Default)]
pub struct FloatingPanesStack {
    pub layers: Vec<PaneGeom>,
}

impl FloatingPanesStack {
    pub fn visible_character_chunks(
        &self,
        mut character_chunks: Vec<CharacterChunk>,
        z_index: Option<usize>,
    ) -> Result<Vec<CharacterChunk>> {
        let err_context = || {
            format!(
                "failed to determine visible character chunks at z-index {:?}",
                z_index
            )
        };

        let z_index = z_index.unwrap_or(0);
        let mut chunks_to_check: Vec<CharacterChunk> = character_chunks.drain(..).collect();
        let mut visible_chunks = vec![];
        'chunk_loop: loop {
            match chunks_to_check.pop() {
                Some(mut c_chunk) => {
                    let panes_to_check = self.layers.iter().skip(z_index);
                    for pane_geom in panes_to_check {
                        let new_chunk_to_check = self
                            .remove_covered_parts(pane_geom, &mut c_chunk)
                            .with_context(err_context)?;
                        if let Some(new_chunk_to_check) = new_chunk_to_check {
                            // this happens when the pane covers the middle of the chunk, and so we
                            // end up with an extra chunk we need to check (eg. against panes above
                            // this one)
                            chunks_to_check.push(new_chunk_to_check);
                        }
                        if c_chunk.terminal_characters.is_empty() {
                            continue 'chunk_loop;
                        }
                    }
                    visible_chunks.push(c_chunk);
                },
                None => {
                    break 'chunk_loop;
                },
            }
        }
        Ok(visible_chunks)
    }
    fn remove_covered_parts(
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
    pub fn cursor_is_visible(
        &self,
        cursor_x: usize,
        cursor_y: usize,
        z_index: Option<usize>,
    ) -> bool {
        let z_index = z_index.map(|z| z + 1).unwrap_or(0); // +1 because we only check panes above the active pane
        let panes_to_check = self.layers.iter().skip(z_index);
        for pane_geom in panes_to_check {
            let pane_top_edge = pane_geom.y;
            let pane_left_edge = pane_geom.x;
            let pane_bottom_edge = pane_geom.y + pane_geom.rows.as_usize().saturating_sub(1);
            let pane_right_edge = pane_geom.x + pane_geom.cols.as_usize().saturating_sub(1);
            if pane_top_edge <= cursor_y
                && pane_bottom_edge >= cursor_y
                && pane_left_edge <= cursor_x
                && pane_right_edge >= cursor_x
            {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Default)]
pub struct CharacterChunk {
    pub terminal_characters: Vec<TerminalCharacter>,
    pub x: usize,
    pub y: usize,
    pub changed_colors: Option<[Option<AnsiCode>; 256]>,
    pub pane_default_fg: Option<AnsiCode>,
    pub pane_default_bg: Option<AnsiCode>,
    selection_and_colors: Vec<HighlightSelection>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SixelImageChunk {
    pub cell_x: usize,
    pub cell_y: usize,
    pub sixel_image_pixel_x: usize,
    pub sixel_image_pixel_y: usize,
    pub sixel_image_pixel_width: usize,
    pub sixel_image_pixel_height: usize,
    pub sixel_image_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KittyImageData {
    Png {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Rgb {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KittyImagePlacementMode {
    Explicit,
    Placeholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlacementId {
    Protocol(u32),
    Synthetic(u32),
}

impl PlacementId {
    pub fn wire_value(self) -> u32 {
        match self {
            PlacementId::Protocol(value) | PlacementId::Synthetic(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KittyPlaceholderCellRender {
    pub cell_x: usize,
    pub cell_y: usize,
    pub placeholder_row: usize,
    pub placeholder_col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KittyPlaceholderRender {
    pub stable_render_id: u64,
    pub image_id: u32,
    pub placement_id: Option<PlacementId>,
    pub columns: usize,
    pub rows: usize,
    pub source_x: u32,
    pub source_y: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub cells: Vec<KittyPlaceholderCellRender>,
}

#[derive(Debug, Clone, Default)]
pub struct ImageRenderBundle {
    pub sixel_chunks: Vec<SixelImageChunk>,
    pub kitty_render_bundle: KittyRenderBundle,
}

#[derive(Debug, Clone, Default)]
pub struct PaneImageRenderOutput {
    pub kitty_scene: KittyRenderBundle,
    pub sixel_chunks: Vec<SixelImageChunk>,
    pub changed_rects: HashMap<usize, usize>,
    pub kitty_host_state_cleared: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PaneRenderOutput {
    pub character_chunks: Vec<CharacterChunk>,
    pub raw_vte_output: Option<String>,
    pub image_output: PaneImageRenderOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KittyImageChunk {
    pub stable_render_id: u64,
    pub image_id: u32,
    pub placement_id: Option<PlacementId>,
    pub placement_mode: KittyImagePlacementMode,
    pub cell_x: usize,
    pub cell_y: usize,
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

impl CharacterChunk {
    pub fn new(terminal_characters: Vec<TerminalCharacter>, x: usize, y: usize) -> Self {
        CharacterChunk {
            terminal_characters,
            x,
            y,
            ..Default::default()
        }
    }
    pub fn add_selection_and_colors(
        &mut self,
        highlight: HighlightSelection,
        offset_x: usize,
        offset_y: usize,
    ) {
        self.selection_and_colors.push(HighlightSelection {
            selection: highlight.selection.offset(offset_x, offset_y),
            ..highlight
        });
    }
    pub fn selection_and_colors(&self) -> &[HighlightSelection] {
        &self.selection_and_colors
    }
    pub fn add_changed_colors(&mut self, changed_colors: Option<[Option<AnsiCode>; 256]>) {
        self.changed_colors = changed_colors;
    }
    pub fn add_pane_defaults(&mut self, fg: Option<AnsiCode>, bg: Option<AnsiCode>) {
        self.pane_default_fg = fg;
        self.pane_default_bg = bg;
    }
    pub fn changed_colors(&self) -> Option<[Option<AnsiCode>; 256]> {
        self.changed_colors
    }
    pub fn width(&self) -> usize {
        let mut width = 0;
        for t_character in &self.terminal_characters {
            width += t_character.width()
        }
        width
    }
    pub fn drain_by_width(&mut self, x: usize) -> impl Iterator<Item = TerminalCharacter> {
        let mut drained_part: VecDeque<TerminalCharacter> = VecDeque::new();
        let mut drained_part_len = 0;
        loop {
            if self.terminal_characters.is_empty() {
                break;
            }
            let next_character = self.terminal_characters.remove(0); // TODO: consider copying self.terminal_characters into a VecDeque to make this process faster?
            if drained_part_len + next_character.width() <= x {
                drained_part_len += next_character.width();
                drained_part.push_back(next_character);
            } else {
                if drained_part_len == x {
                    self.terminal_characters.insert(0, next_character); // put it back
                } else if next_character.width() > 1 {
                    for _ in 1..next_character.width() {
                        self.terminal_characters.insert(0, EMPTY_TERMINAL_CHARACTER);
                        drained_part.push_back(EMPTY_TERMINAL_CHARACTER);
                    }
                }
                break;
            }
        }
        drained_part.into_iter()
    }
    pub fn retain_by_width(&mut self, x: usize) {
        let part_to_retain = self.drain_by_width(x);
        self.terminal_characters = part_to_retain.collect();
    }
    pub fn cut_middle_out(
        &mut self,
        middle_start: usize,
        middle_end: usize,
    ) -> Result<(Vec<TerminalCharacter>, Vec<TerminalCharacter>)> {
        let err_context = || "failed to cut middle out of character chunk".to_string();

        let (
            absolute_middle_start_index,
            absolute_middle_end_index,
            pad_left_end_by,
            pad_right_start_by,
        ) = adjust_middle_segment_for_wide_chars(
            middle_start,
            middle_end,
            &self.terminal_characters,
        )
        .with_context(err_context)?;
        let mut terminal_characters: Vec<TerminalCharacter> =
            self.terminal_characters.drain(..).collect();
        let mut characters_on_the_right: Vec<TerminalCharacter> = terminal_characters
            .drain(absolute_middle_end_index..)
            .collect();
        let mut characters_on_the_left: Vec<TerminalCharacter> = terminal_characters
            .drain(..absolute_middle_start_index)
            .collect();
        if pad_left_end_by > 0 {
            characters_on_the_left.resize(pad_left_end_by, EMPTY_TERMINAL_CHARACTER);
        }
        if pad_right_start_by > 0 {
            for _ in 0..pad_right_start_by {
                characters_on_the_right.insert(0, EMPTY_TERMINAL_CHARACTER);
            }
        }
        Ok((characters_on_the_left, characters_on_the_right))
    }
}

#[derive(Clone, Debug)]
pub struct OutputBuffer {
    pub changed_lines: HashSet<usize>, // line index
    pub should_update_all_lines: bool,
    styled_underlines: bool,
}

impl Default for OutputBuffer {
    fn default() -> Self {
        OutputBuffer {
            changed_lines: HashSet::new(),
            should_update_all_lines: true, // first time we should do a full render
            styled_underlines: true,
        }
    }
}

impl OutputBuffer {
    pub fn update_line(&mut self, line_index: usize) {
        if !self.should_update_all_lines {
            self.changed_lines.insert(line_index);
        }
    }
    pub fn update_lines(&mut self, start: usize, end: usize) {
        if !self.should_update_all_lines {
            for idx in start..=end {
                if !self.changed_lines.contains(&idx) {
                    self.changed_lines.insert(idx);
                }
            }
        }
    }
    pub fn update_all_lines(&mut self) {
        self.clear();
        self.should_update_all_lines = true;
    }
    pub fn clear(&mut self) {
        self.changed_lines.clear();
        self.should_update_all_lines = false;
    }
    pub fn serialize(
        &self,
        viewport: &[Row],
        osc8_hyperlinks: bool,
        max_size: Option<Size>,
    ) -> Result<String> {
        let mut chunks = Vec::new();
        for (line_index, line) in viewport.iter().enumerate() {
            let terminal_characters =
                self.extract_line_from_viewport(line_index, viewport, line.width());

            let x = 0;
            let y = line_index;
            chunks.push(CharacterChunk::new(terminal_characters, x, y));
        }
        serialize_chunks_with_newlines(
            chunks,
            None,
            None,
            self.styled_underlines,
            osc8_hyperlinks,
            max_size,
        )
    }
    pub fn changed_chunks_in_viewport(
        &self,
        viewport: &[Row],
        viewport_width: usize,
        viewport_height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Vec<CharacterChunk> {
        if self.should_update_all_lines {
            let mut changed_chunks = Vec::new();
            for line_index in 0..viewport_height {
                let terminal_characters =
                    self.extract_line_from_viewport(line_index, viewport, viewport_width);

                let x = x_offset; // right now we only buffer full lines as this doesn't seem to have a huge impact on performance, but the infra is here if we want to change this
                let y = line_index + y_offset;
                changed_chunks.push(CharacterChunk::new(terminal_characters, x, y));
            }
            changed_chunks
        } else {
            let mut line_changes: Vec<_> = self
                .changed_lines
                .iter()
                .filter(|i| *i < &viewport_height)
                .copied()
                .collect();
            line_changes.sort_unstable();
            let mut changed_chunks = Vec::new();
            for line_index in line_changes {
                let terminal_characters =
                    self.extract_line_from_viewport(line_index, viewport, viewport_width);
                let x = x_offset;
                let y = line_index + y_offset;
                changed_chunks.push(CharacterChunk::new(terminal_characters, x, y));
            }
            changed_chunks
        }
    }
    fn extract_characters_from_row(
        &self,
        row: &Row,
        viewport_width: usize,
    ) -> Vec<TerminalCharacter> {
        let mut terminal_characters: Vec<TerminalCharacter> = row.columns.iter().cloned().collect();
        // pad row
        let row_width = row.width();
        if row_width < viewport_width {
            let mut pad_character = EMPTY_TERMINAL_CHARACTER;
            if let Some(bg_color) = row.bg_color {
                pad_character
                    .styles
                    .update(|styles| styles.background = Some(bg_color));
            }
            let mut padding = vec![pad_character; viewport_width - row_width];
            terminal_characters.append(&mut padding);
        } else if row_width > viewport_width {
            let width_offset = row.excess_width_until(viewport_width);
            let truncate_position = viewport_width.saturating_sub(width_offset);
            if truncate_position < terminal_characters.len() {
                terminal_characters.truncate(truncate_position);
            }
        }
        terminal_characters
    }
    fn extract_line_from_viewport(
        &self,
        line_index: usize,
        viewport: &[Row],
        viewport_width: usize,
    ) -> Vec<TerminalCharacter> {
        match viewport.get(line_index) {
            // TODO: iterator?
            Some(row) => self.extract_characters_from_row(row, viewport_width),
            None => {
                vec![EMPTY_TERMINAL_CHARACTER; viewport_width]
            },
        }
    }
    pub fn changed_rects_in_viewport(&self, viewport_height: usize) -> HashMap<usize, usize> {
        // group the changed lines into "changed_rects", which indicate where the line starts (the
        // hashmap key) and how many lines are in there (its value)
        let mut changed_rects: HashMap<usize, usize> = HashMap::new(); // <start_line_index, line_count>
        let mut last_changed_line_index: Option<usize> = None;
        let mut changed_line_count = 0;
        let mut add_changed_line = |line_index| match last_changed_line_index.as_mut() {
            Some(changed_line_index) => {
                if *changed_line_index + changed_line_count == line_index {
                    changed_line_count += 1
                } else {
                    changed_rects.insert(*changed_line_index, changed_line_count);
                    last_changed_line_index = Some(line_index);
                    changed_line_count = 1;
                }
            },
            None => {
                last_changed_line_index = Some(line_index);
                changed_line_count = 1;
            },
        };

        // TODO: move this whole thing to output_buffer
        if self.should_update_all_lines {
            // for line_index in 0..self.viewport.len() {
            for line_index in 0..viewport_height {
                add_changed_line(line_index);
            }
        } else {
            for line_index in self.changed_lines.iter().copied() {
                add_changed_line(line_index);
            }
        }
        if let Some(changed_line_index) = last_changed_line_index {
            changed_rects.insert(changed_line_index, changed_line_count);
        }
        changed_rects
    }
}

#[cfg(test)]
mod unit;
