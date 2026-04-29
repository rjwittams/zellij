use super::{
    image_fragment::{
        visible_image_fragments, ImageFragment, PreparedAfterTextImages, PreparedImageOutput,
    },
    kitty_diff::{
        plan_kitty_scene, KittyAssetOp, KittyPlacementKey, KittyPlacementOp, KittyScenePlan,
        KittySceneState, PlannedKittyPlacement,
    },
    vte_goto_instruction, FloatingPanesStack, IntoLastRenderedImageState, KittyImageChunk,
    KittyPlaceholderRender, LastRenderedImageState, PaneImageRenderOutput, RenderedImageState,
    SixelImageChunk,
};
use crate::output::KittyOutputMediaCache;
use crate::panes::kitty::KittyImageState;
use crate::panes::kitty_asset_store::KittyAssetStore;
use crate::panes::pane_image_scene::KittyDamageRedraw;
use crate::panes::sixel::SixelImageStore;
use crate::ClientId;
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet},
    rc::Rc,
};
use zellij_utils::errors::prelude::*;
use zellij_utils::pane_size::{Size, SizeInPixels};

#[derive(Clone, Debug, Default)]
struct CurrentImageState {
    sixel_chunks: Vec<SixelImageChunk>,
    kitty_chunks: Vec<KittyImageChunk>,
    kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
    changed_rects: HashMap<usize, usize>,
    kitty_host_state_cleared: bool,
}

#[derive(Clone, Debug)]
struct PreparedKittyRenderPlan {
    desired_scene: Option<KittySceneState>,
    kitty_plan: KittyScenePlan,
    before_text_vte: Option<String>,
}

impl PreparedKittyRenderPlan {
    fn new(
        kitty_asset_store: &KittyAssetStore,
        current_kitty_chunks: &[KittyImageChunk],
        current_kitty_placeholder_renders: &[KittyPlaceholderRender],
        changed_rects: HashMap<usize, usize>,
        last_rendered_image_state: &RenderedImageState,
        last_rendered_kitty_scene: Option<&KittySceneState>,
        pre_vte_clears_display: bool,
        kitty_host_state_cleared: bool,
    ) -> Self {
        let kitty_damage_redraw = KittyDamageRedraw::from_changed_rects(changed_rects);
        let empty_resident_assets = HashMap::new();
        let desired_scene = ImageOutput::kitty_scene_state_from_rendered(
            &empty_resident_assets,
            current_kitty_chunks,
            current_kitty_placeholder_renders,
            kitty_asset_store,
        );
        let has_current_kitty_scene =
            !current_kitty_chunks.is_empty() || !current_kitty_placeholder_renders.is_empty();
        let has_previous_kitty_scene = !last_rendered_image_state.explicit_chunks.is_empty()
            || !last_rendered_image_state.placeholder_renders.is_empty()
            || !last_rendered_image_state
                .resident_asset_generations
                .is_empty();
        let host_kitty_state_is_empty = pre_vte_clears_display || kitty_host_state_cleared;
        let should_clear_kitty_before_text =
            host_kitty_state_is_empty && (has_current_kitty_scene || has_previous_kitty_scene);
        let assumed_kitty_scene = if host_kitty_state_is_empty {
            Some(KittySceneState::default())
        } else {
            last_rendered_kitty_scene.cloned().or_else(|| {
                ImageOutput::kitty_scene_state_from_rendered(
                    &last_rendered_image_state.resident_asset_generations,
                    &last_rendered_image_state.explicit_chunks,
                    &last_rendered_image_state.placeholder_renders,
                    kitty_asset_store,
                )
            })
        };
        let kitty_plan = match (assumed_kitty_scene, desired_scene.clone()) {
            (Some(assumed_kitty_scene), Some(desired_kitty_scene)) => {
                let mut kitty_plan = plan_kitty_scene(&assumed_kitty_scene, &desired_kitty_scene);
                if let KittyScenePlan::Diff {
                    asset_ops: _,
                    placement_ops,
                } = &mut kitty_plan
                {
                    let place_keys: HashSet<KittyPlacementKey> = placement_ops
                        .iter()
                        .filter_map(|placement_op| match placement_op {
                            KittyPlacementOp::PlaceExplicit { key, .. }
                            | KittyPlacementOp::PlacePlaceholder { key, .. } => Some(*key),
                            KittyPlacementOp::Delete { .. } => None,
                        })
                        .collect();
                    for (key, placement) in desired_kitty_scene.placements.iter() {
                        if place_keys.contains(key)
                            || !ImageOutput::placement_intersects_damage(
                                placement,
                                &kitty_damage_redraw,
                            )
                        {
                            continue;
                        }
                        placement_ops.push(match placement {
                            PlannedKittyPlacement::Explicit { key, chunk } => {
                                KittyPlacementOp::PlaceExplicit {
                                    key: *key,
                                    chunk: chunk.clone(),
                                }
                            },
                            PlannedKittyPlacement::Placeholder { key, render } => {
                                KittyPlacementOp::PlacePlaceholder {
                                    key: *key,
                                    render: render.clone(),
                                }
                            },
                        });
                    }
                }
                kitty_plan
            },
            _ => ImageOutput::render_full_reset_plan(
                current_kitty_chunks,
                current_kitty_placeholder_renders,
            ),
        };
        let before_text_vte = match &kitty_plan {
            KittyScenePlan::Diff {
                asset_ops: _,
                placement_ops,
            } => {
                if should_clear_kitty_before_text {
                    Some(kitty_clear_before_text_vte())
                } else {
                    ImageOutput::serialize_kitty_delete_ops(placement_ops)
                }
            },
            KittyScenePlan::FullResetAndResend { .. } => (has_current_kitty_scene
                || has_previous_kitty_scene)
                .then(kitty_clear_before_text_vte),
        };
        Self {
            desired_scene,
            kitty_plan,
            before_text_vte,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ClientImageRenderState {
    current: CurrentImageState,
    last_rendered_state: Rc<LastRenderedImageState>,
}

impl ClientImageRenderState {
    fn set_last_rendered_state(&mut self, state: Rc<LastRenderedImageState>) {
        self.last_rendered_state = state;
    }

    fn last_rendered_image_state(&self) -> &RenderedImageState {
        self.last_rendered_state.rendered_image_state()
    }

    fn clone_last_rendered_state(&self) -> Option<Rc<LastRenderedImageState>> {
        (!self.last_rendered_state.is_empty()).then(|| Rc::clone(&self.last_rendered_state))
    }

    fn pending_has_rendered_assets(&self) -> bool {
        !self.current.sixel_chunks.is_empty()
            || !self.current.kitty_chunks.is_empty()
            || !self.current.kitty_placeholder_renders.is_empty()
    }

    fn push_fragment(&mut self, fragment: ImageFragment) {
        match fragment {
            ImageFragment::Sixel(sixel_chunk) => self.current.sixel_chunks.push(sixel_chunk),
            ImageFragment::KittyExplicit(kitty_chunk) => {
                self.current.kitty_chunks.push(kitty_chunk)
            },
            ImageFragment::KittyPlaceholder(placeholder_render) => self
                .current
                .kitty_placeholder_renders
                .push(placeholder_render),
        }
    }

    fn add_changed_rects(&mut self, changed_rects: HashMap<usize, usize>) {
        for (start_row, line_count) in changed_rects {
            self.current
                .changed_rects
                .entry(start_row)
                .and_modify(|current_line_count| {
                    *current_line_count = (*current_line_count).max(line_count);
                })
                .or_insert(line_count);
        }
    }

    fn set_kitty_host_state_cleared(&mut self) {
        self.current.kitty_host_state_cleared = true;
    }

    fn take_current_render_state(
        &mut self,
    ) -> (
        Vec<SixelImageChunk>,
        Vec<KittyImageChunk>,
        Vec<KittyPlaceholderRender>,
        HashMap<usize, usize>,
        bool,
    ) {
        (
            std::mem::take(&mut self.current.sixel_chunks),
            std::mem::take(&mut self.current.kitty_chunks),
            std::mem::take(&mut self.current.kitty_placeholder_renders),
            std::mem::take(&mut self.current.changed_rects),
            std::mem::take(&mut self.current.kitty_host_state_cleared),
        )
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ImageOutput {
    client_image_states: HashMap<ClientId, ClientImageRenderState>,
    clients_with_kitty_file_output: HashSet<ClientId>,
    pub(crate) sixel_image_store: Rc<RefCell<SixelImageStore>>,
    pub(crate) kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
    kitty_output_media_cache: Rc<RefCell<KittyOutputMediaCache>>,
    character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
}

impl PreparedAfterTextImages {
    pub(crate) fn serialize(
        self,
        image_output: &mut ImageOutput,
        max_size: Option<Size>,
        vte_output: &mut String,
    ) -> Result<()> {
        let PreparedAfterTextImages {
            client_id,
            fragments,
            kitty_plan,
            current_kitty_scene,
            current_kitty_chunks,
            current_kitty_placeholder_renders,
        } = self;
        let err_context = || "failed to serialize image chunks".to_string();
        let mut sixel_vte: Option<String> = None;
        for fragment in fragments {
            match fragment {
                ImageFragment::Sixel(sixel_chunk) => {
                    // Skip sixel chunks that are completely outside the size bounds
                    if let Some(size) = max_size {
                        if sixel_chunk.cell_y >= size.rows {
                            continue; // Sixel chunk is below visible area
                        }
                        if sixel_chunk.cell_x >= size.cols {
                            continue; // Sixel chunk starts outside visible area
                        }
                    }

                    let serialized_sixel_image =
                        image_output.sixel_image_store.borrow_mut().serialize_image(
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
                },
                ImageFragment::KittyExplicit(_) | ImageFragment::KittyPlaceholder(_) => {},
            }
        }

        if let Some(ref sixel_vte) = sixel_vte {
            // we do this at the end because of the implied z-index,
            // images should be above text unless the text was explicitly inserted after them (the
            // latter being a case we handle in our own internal state and not in the output)
            let save_cursor_position = "\u{1b}[s";
            let restore_cursor_position = "\u{1b}[u";
            vte_output.push_str(save_cursor_position);
            vte_output.push_str(sixel_vte);
            vte_output.push_str(restore_cursor_position);
        }

        vte_output.push_str(&image_output.serialize_kitty_plan(client_id, &kitty_plan));

        image_output.finish_render_body_for_client(
            client_id,
            current_kitty_scene,
            current_kitty_chunks,
            current_kitty_placeholder_renders,
            &kitty_plan,
        );
        Ok(())
    }
}

fn kitty_clear_before_text_vte() -> String {
    let mut vte_output = String::new();
    vte_output.push_str("\u{1b}[s");
    vte_output.push_str("\u{1b}_Ga=d,d=A\u{1b}\\");
    vte_output.push_str("\u{1b}[u");
    vte_output
}

impl ImageOutput {
    pub fn new(
        sixel_image_store: Rc<RefCell<SixelImageStore>>,
        kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
        kitty_output_media_cache: Rc<RefCell<KittyOutputMediaCache>>,
        character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
    ) -> Self {
        Self {
            sixel_image_store,
            kitty_asset_store,
            kitty_output_media_cache,
            character_cell_size,
            ..Default::default()
        }
    }

    pub fn set_kitty_file_output_enabled_for_client(&mut self, client_id: ClientId, enabled: bool) {
        if enabled {
            self.clients_with_kitty_file_output.insert(client_id);
        } else {
            self.clients_with_kitty_file_output.remove(&client_id);
        }
    }

    fn client_image_state_mut(&mut self, client_id: ClientId) -> &mut ClientImageRenderState {
        self.client_image_states.entry(client_id).or_default()
    }

    fn stable_wire_placement_id(
        protocol_placement_id: Option<super::PlacementId>,
        stable_render_id: u64,
    ) -> super::PlacementId {
        protocol_placement_id
            .unwrap_or_else(|| super::PlacementId::Synthetic((stable_render_id as u32).max(1)))
    }

    fn kitty_scene_state_from_rendered(
        resident_kitty_asset_generations: &HashMap<u32, u64>,
        chunks: &[KittyImageChunk],
        placeholder_renders: &[KittyPlaceholderRender],
        kitty_asset_store: &KittyAssetStore,
    ) -> Option<KittySceneState> {
        let placement_count = chunks.len() + placeholder_renders.len();
        let mut scene = KittySceneState {
            resident_asset_generations: HashMap::with_capacity(
                resident_kitty_asset_generations.len() + placement_count,
            ),
            placements: HashMap::with_capacity(placement_count),
        };
        scene.resident_asset_generations.extend(
            resident_kitty_asset_generations
                .iter()
                .map(|(image_id, generation)| (*image_id, *generation)),
        );
        for chunk in chunks {
            let placement_id =
                ImageOutput::stable_wire_placement_id(chunk.placement_id, chunk.stable_render_id);
            if let Entry::Vacant(entry) = scene.resident_asset_generations.entry(chunk.image_id) {
                entry.insert(kitty_asset_store.generation(chunk.image_id)?);
            }
            scene.insert_placement(PlannedKittyPlacement::Explicit {
                key: KittyPlacementKey {
                    stable_render_id: chunk.stable_render_id,
                    image_id: chunk.image_id,
                    wire_placement_id: placement_id,
                },
                chunk: chunk.clone(),
            });
        }
        for render in placeholder_renders {
            let placement_id =
                ImageOutput::stable_wire_placement_id(render.placement_id, render.stable_render_id);
            if let Entry::Vacant(entry) = scene.resident_asset_generations.entry(render.image_id) {
                entry.insert(kitty_asset_store.generation(render.image_id)?);
            }
            scene.insert_placement(PlannedKittyPlacement::Placeholder {
                key: KittyPlacementKey {
                    stable_render_id: render.stable_render_id,
                    image_id: render.image_id,
                    wire_placement_id: placement_id,
                },
                render: render.clone(),
            });
        }
        Some(scene)
    }

    fn placement_intersects_damage(
        placement: &PlannedKittyPlacement,
        kitty_damage_redraw: &KittyDamageRedraw,
    ) -> bool {
        match placement {
            PlannedKittyPlacement::Explicit { chunk, .. } => {
                kitty_damage_redraw.intersects_absolute_rows(0, chunk.cell_y, chunk.rows)
            },
            PlannedKittyPlacement::Placeholder { render, .. } => render
                .cells
                .iter()
                .any(|cell| kitty_damage_redraw.intersects_absolute_rows(0, cell.cell_y, 1)),
        }
    }

    fn render_full_reset_plan(
        current_kitty_chunks: &[KittyImageChunk],
        current_kitty_placeholder_renders: &[KittyPlaceholderRender],
    ) -> KittyScenePlan {
        KittyScenePlan::FullResetAndResend {
            explicit_chunks: current_kitty_chunks.to_vec(),
            placeholder_renders: current_kitty_placeholder_renders.to_vec(),
        }
    }

    fn serialize_kitty_delete_ops(placement_ops: &[KittyPlacementOp]) -> Option<String> {
        let mut vte_output = String::new();
        for placement_op in placement_ops {
            if let KittyPlacementOp::Delete { key } = placement_op {
                vte_output.push_str(&KittyImageState::serialize_delete_placement(
                    key.image_id,
                    key.wire_placement_id.wire_value(),
                ));
            }
        }
        (!vte_output.is_empty()).then_some(vte_output)
    }

    fn next_resident_assets_for_plan(
        kitty_asset_store: &KittyAssetStore,
        mut previous_resident_assets: HashMap<u32, u64>,
        kitty_plan: &KittyScenePlan,
    ) -> HashMap<u32, u64> {
        match kitty_plan {
            KittyScenePlan::Diff {
                asset_ops,
                placement_ops: _,
            } => {
                previous_resident_assets.reserve(asset_ops.len());
                for asset_op in asset_ops {
                    match asset_op {
                        KittyAssetOp::EnsureResident {
                            image_id,
                            generation,
                        } => {
                            previous_resident_assets.insert(*image_id, *generation);
                        },
                    }
                }
                previous_resident_assets
            },
            KittyScenePlan::FullResetAndResend {
                explicit_chunks,
                placeholder_renders,
            } => {
                let mut next_resident_assets =
                    HashMap::with_capacity(explicit_chunks.len() + placeholder_renders.len());
                for chunk in explicit_chunks {
                    if let Some(generation) = kitty_asset_store.generation(chunk.image_id) {
                        next_resident_assets.insert(chunk.image_id, generation);
                    }
                }
                for render in placeholder_renders {
                    if let Some(generation) = kitty_asset_store.generation(render.image_id) {
                        next_resident_assets.insert(render.image_id, generation);
                    }
                }
                next_resident_assets
            },
        }
    }

    fn serialize_kitty_plan(&mut self, client_id: ClientId, kitty_plan: &KittyScenePlan) -> String {
        match kitty_plan {
            KittyScenePlan::Diff {
                asset_ops,
                placement_ops,
            } => {
                let mut vte_output = String::new();
                if asset_ops.is_empty()
                    && placement_ops
                        .iter()
                        .all(|placement_op| matches!(placement_op, KittyPlacementOp::Delete { .. }))
                {
                    return vte_output;
                }
                vte_output.push_str("\u{1b}[s");
                for asset_op in asset_ops {
                    match asset_op {
                        KittyAssetOp::EnsureResident {
                            image_id,
                            generation,
                        } => {
                            let kitty_asset_store = self.kitty_asset_store.borrow();
                            let asset = kitty_asset_store.asset(*image_id);
                            let Some(asset) = asset else {
                                continue;
                            };
                            if asset.generation != *generation {
                                continue;
                            }
                            vte_output.push_str(&Self::serialize_kitty_image_data(
                                &self.clients_with_kitty_file_output,
                                &self.kitty_output_media_cache,
                                client_id,
                                *image_id,
                                *generation,
                                &asset.image_data,
                            ));
                        },
                    }
                }
                for placement_op in placement_ops {
                    match placement_op {
                        KittyPlacementOp::Delete { .. } => {},
                        KittyPlacementOp::PlaceExplicit { key, chunk } => {
                            vte_output.push_str(&KittyImageState::serialize_explicit_placement(
                                chunk,
                                key.wire_placement_id.wire_value(),
                            ))
                        },
                        KittyPlacementOp::PlacePlaceholder { key, render } => {
                            vte_output.push_str(&KittyImageState::serialize_placeholder_render(
                                render,
                                key.wire_placement_id.wire_value(),
                            ))
                        },
                    }
                }
                vte_output.push_str("\u{1b}[u");
                vte_output
            },
            KittyScenePlan::FullResetAndResend {
                explicit_chunks,
                placeholder_renders,
            } => self.serialize_kitty_full_reset(client_id, explicit_chunks, placeholder_renders),
        }
    }

    fn serialize_kitty_image_data(
        clients_with_kitty_file_output: &HashSet<ClientId>,
        kitty_output_media_cache: &Rc<RefCell<KittyOutputMediaCache>>,
        client_id: ClientId,
        image_id: u32,
        generation: u64,
        image_data: &crate::output::KittyImageData,
    ) -> String {
        if clients_with_kitty_file_output.contains(&client_id) {
            if let Ok(path) = kitty_output_media_cache
                .borrow_mut()
                .ensure_regular_file(image_id, generation, image_data)
            {
                return KittyImageState::serialize_image_data_from_file(
                    image_id, image_data, &path,
                );
            }
        }
        KittyImageState::serialize_image_data(image_id, image_data)
    }

    fn serialize_kitty_full_reset(
        &mut self,
        client_id: ClientId,
        chunks: &[KittyImageChunk],
        renders: &[KittyPlaceholderRender],
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
                let kitty_asset_store = self.kitty_asset_store.borrow();
                let asset = kitty_asset_store.asset(image_id);
                let Some(asset) = asset else {
                    continue;
                };
                raw_vte_output.push_str(&Self::serialize_kitty_image_data(
                    &self.clients_with_kitty_file_output,
                    &self.kitty_output_media_cache,
                    client_id,
                    image_id,
                    asset.generation,
                    &asset.image_data,
                ));
            }
        }

        let mut next_synthesized_placement_id = 1u32;
        for chunk in chunks {
            let placement_id = chunk.placement_id.unwrap_or_else(|| {
                let placement_id = next_synthesized_placement_id;
                next_synthesized_placement_id += 1;
                crate::output::PlacementId::Synthetic(placement_id)
            });
            raw_vte_output.push_str(&KittyImageState::serialize_explicit_placement(
                chunk,
                placement_id.wire_value(),
            ));
        }
        for render in renders {
            let placement_id = render.placement_id.unwrap_or_else(|| {
                let placement_id = next_synthesized_placement_id;
                next_synthesized_placement_id += 1;
                crate::output::PlacementId::Synthetic(placement_id)
            });
            raw_vte_output.push_str(&KittyImageState::serialize_placeholder_render(
                render,
                placement_id.wire_value(),
            ));
        }
        raw_vte_output.push_str("\u{1b}[u");
        raw_vte_output
    }

    fn visible_image_fragments_from_pane_output(
        &self,
        pane_image_output: &PaneImageRenderOutput,
        floating_panes_stack: Option<&FloatingPanesStack>,
        z_index: Option<usize>,
    ) -> Vec<ImageFragment> {
        let mut fragments = vec![];
        fragments.extend(
            pane_image_output
                .sixel_chunks
                .iter()
                .cloned()
                .map(ImageFragment::Sixel),
        );
        fragments.extend(
            pane_image_output
                .kitty_scene
                .explicit_chunks
                .iter()
                .cloned()
                .map(ImageFragment::KittyExplicit),
        );
        fragments.extend(
            pane_image_output
                .kitty_scene
                .placeholder_renders
                .iter()
                .cloned()
                .map(ImageFragment::KittyPlaceholder),
        );
        if let Some(floating_panes_stack) = floating_panes_stack {
            let character_cell_size = self.character_cell_size.borrow();
            visible_image_fragments(
                floating_panes_stack,
                fragments,
                z_index,
                character_cell_size.as_ref(),
            )
        } else {
            fragments
        }
    }

    fn kitty_scene_is_dirty(&self) -> bool {
        self.client_image_states.values().any(|client_state| {
            client_state.current.kitty_chunks
                != client_state.last_rendered_image_state().explicit_chunks
                || client_state.current.kitty_placeholder_renders
                    != client_state.last_rendered_image_state().placeholder_renders
                || client_state
                    .current
                    .kitty_chunks
                    .iter()
                    .map(|chunk| chunk.image_id)
                    .chain(
                        client_state
                            .current
                            .kitty_placeholder_renders
                            .iter()
                            .map(|render| render.image_id),
                    )
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .any(|image_id| {
                        let Some(current_image_data) =
                            self.kitty_asset_store.borrow().generation(image_id)
                        else {
                            return false;
                        };
                        client_state
                            .last_rendered_image_state()
                            .resident_asset_generations
                            .get(&image_id)
                            != Some(&current_image_data)
                    })
        })
    }

    fn set_last_rendered_image_state<T>(&mut self, client_id: ClientId, image_state: T)
    where
        T: IntoLastRenderedImageState,
    {
        let image_state = self.prepare_last_rendered_snapshot(image_state);
        if let Some(scene_state) = image_state.kitty_scene_state() {
            log::trace!(
                target: "zellij::kitty_images",
                "set last rendered kitty image state for client {client_id}: resident_assets={}, placements={}, explicit_chunks={}, placeholder_renders={}",
                scene_state.resident_asset_generations.len(),
                scene_state.placements.len(),
                image_state.rendered_image_state().explicit_chunks.len(),
                image_state.rendered_image_state().placeholder_renders.len(),
            );
        }
        self.client_image_state_mut(client_id)
            .set_last_rendered_state(image_state);
    }

    fn prepare_last_rendered_snapshot<T>(&self, image_state: T) -> Rc<LastRenderedImageState>
    where
        T: IntoLastRenderedImageState,
    {
        let image_state = image_state.into_last_rendered_image_state();
        if image_state.kitty_scene_state().is_some() || image_state.is_empty() {
            return image_state;
        }
        let normalized_state = {
            let kitty_asset_store = self.kitty_asset_store.borrow();
            Self::normalize_rendered_image_state(
                &kitty_asset_store,
                image_state.rendered_image_state().clone(),
            )
        };
        let scene_state = {
            let kitty_asset_store = self.kitty_asset_store.borrow();
            Self::kitty_scene_state_from_rendered(
                &normalized_state.resident_asset_generations,
                &normalized_state.explicit_chunks,
                &normalized_state.placeholder_renders,
                &kitty_asset_store,
            )
        };
        Rc::new(LastRenderedImageState::with_kitty_scene_state(
            normalized_state,
            scene_state,
        ))
    }

    fn normalize_rendered_image_state(
        kitty_asset_store: &KittyAssetStore,
        mut image_state: RenderedImageState,
    ) -> RenderedImageState {
        for image_id in image_state
            .explicit_chunks
            .iter()
            .map(|chunk| chunk.image_id)
            .chain(
                image_state
                    .placeholder_renders
                    .iter()
                    .map(|render| render.image_id),
            )
        {
            if let Some(generation) = kitty_asset_store.generation(image_id) {
                image_state
                    .resident_asset_generations
                    .entry(image_id)
                    .or_insert(generation);
            }
        }
        image_state
    }

    pub fn set_last_rendered_image_states<T>(
        &mut self,
        last_rendered_image_states: HashMap<ClientId, T>,
    ) where
        T: IntoLastRenderedImageState,
    {
        for (client_id, image_state) in last_rendered_image_states {
            self.set_last_rendered_image_state(client_id, image_state);
        }
    }

    pub fn set_last_rendered_image_state_for_client<T>(
        &mut self,
        client_id: ClientId,
        image_state: T,
    ) where
        T: IntoLastRenderedImageState,
    {
        self.set_last_rendered_image_state(client_id, image_state);
    }

    pub fn last_rendered_image_states(&self) -> HashMap<ClientId, Rc<LastRenderedImageState>> {
        let mut last_rendered_image_states = HashMap::new();
        for (client_id, client_state) in &self.client_image_states {
            if let Some(image_state) = client_state.clone_last_rendered_state() {
                last_rendered_image_states.insert(*client_id, image_state);
            }
        }
        last_rendered_image_states
    }

    pub fn last_rendered_image_state_for_client(
        &self,
        client_id: ClientId,
    ) -> Option<Rc<LastRenderedImageState>> {
        let client_state = self.client_image_states.get(&client_id)?;
        client_state.clone_last_rendered_state()
    }

    fn add_changed_rects_to_client(
        &mut self,
        client_id: ClientId,
        changed_rects: HashMap<usize, usize>,
    ) {
        let client_state = self.client_image_state_mut(client_id);
        client_state.add_changed_rects(changed_rects);
    }

    fn add_changed_rects_to_multiple_clients(
        &mut self,
        changed_rects: HashMap<usize, usize>,
        client_ids: impl Iterator<Item = ClientId>,
    ) {
        for client_id in client_ids {
            self.add_changed_rects_to_client(client_id, changed_rects.clone());
        }
    }

    pub fn add_pane_image_output_to_client(
        &mut self,
        client_id: ClientId,
        pane_image_output: PaneImageRenderOutput,
        floating_panes_stack: Option<&FloatingPanesStack>,
        z_index: Option<usize>,
    ) {
        let visible_fragments = self.visible_image_fragments_from_pane_output(
            &pane_image_output,
            floating_panes_stack,
            z_index,
        );
        let client_state = self.client_image_state_mut(client_id);
        for fragment in visible_fragments {
            client_state.push_fragment(fragment);
        }
        if pane_image_output.kitty_host_state_cleared {
            client_state.set_kitty_host_state_cleared();
        }
        self.add_changed_rects_to_client(client_id, pane_image_output.changed_rects);
    }

    pub fn add_pane_image_output_to_multiple_clients(
        &mut self,
        pane_image_output: PaneImageRenderOutput,
        client_ids: impl Iterator<Item = ClientId>,
        floating_panes_stack: Option<&FloatingPanesStack>,
        z_index: Option<usize>,
    ) {
        let visible_fragments = self.visible_image_fragments_from_pane_output(
            &pane_image_output,
            floating_panes_stack,
            z_index,
        );
        let client_ids: Vec<ClientId> = client_ids.collect();
        for client_id in client_ids.iter().copied() {
            let client_state = self.client_image_state_mut(client_id);
            for fragment in visible_fragments.iter().cloned() {
                client_state.push_fragment(fragment);
            }
            if pane_image_output.kitty_host_state_cleared {
                client_state.set_kitty_host_state_cleared();
            }
        }
        self.add_changed_rects_to_multiple_clients(
            pane_image_output.changed_rects,
            client_ids.iter().copied(),
        );
    }

    pub(super) fn prepare_render_body_for_client(
        &mut self,
        client_id: ClientId,
        pre_vte_clears_display: bool,
    ) -> PreparedImageOutput {
        let (
            sixel_chunks,
            current_kitty_chunks,
            current_kitty_placeholder_renders,
            changed_rects,
            kitty_host_state_cleared,
            last_rendered_state,
        ) = {
            let client_state = self.client_image_state_mut(client_id);
            let (
                sixel_chunks,
                current_kitty_chunks,
                current_kitty_placeholder_renders,
                changed_rects,
                kitty_host_state_cleared,
            ) = client_state.take_current_render_state();
            (
                sixel_chunks,
                current_kitty_chunks,
                current_kitty_placeholder_renders,
                changed_rects,
                kitty_host_state_cleared,
                Rc::clone(&client_state.last_rendered_state),
            )
        };

        let prepared_kitty_render_plan = {
            let kitty_asset_store = self.kitty_asset_store.borrow();
            PreparedKittyRenderPlan::new(
                &kitty_asset_store,
                &current_kitty_chunks,
                &current_kitty_placeholder_renders,
                changed_rects,
                last_rendered_state.rendered_image_state(),
                last_rendered_state.kitty_scene_state(),
                pre_vte_clears_display,
                kitty_host_state_cleared,
            )
        };

        let mut fragments = vec![];
        fragments.extend(sixel_chunks.iter().cloned().map(ImageFragment::Sixel));

        PreparedImageOutput {
            before_text_vte: prepared_kitty_render_plan.before_text_vte,
            after_text: PreparedAfterTextImages {
                client_id,
                fragments,
                kitty_plan: prepared_kitty_render_plan.kitty_plan,
                current_kitty_scene: prepared_kitty_render_plan.desired_scene,
                current_kitty_chunks,
                current_kitty_placeholder_renders,
            },
        }
    }

    pub(super) fn finish_render_body_for_client(
        &mut self,
        client_id: ClientId,
        mut current_kitty_scene: Option<KittySceneState>,
        current_kitty_chunks: Vec<KittyImageChunk>,
        current_kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
        kitty_plan: &KittyScenePlan,
    ) {
        let previous_resident_assets = self
            .client_image_states
            .get(&client_id)
            .map(|client_state| {
                client_state
                    .last_rendered_image_state()
                    .resident_asset_generations
                    .clone()
            })
            .unwrap_or_default();
        let next_resident_assets = {
            let kitty_asset_store = self.kitty_asset_store.borrow();
            Self::next_resident_assets_for_plan(
                &kitty_asset_store,
                previous_resident_assets,
                kitty_plan,
            )
        };
        if let Some(scene_state) = current_kitty_scene.as_mut() {
            scene_state.resident_asset_generations = next_resident_assets.clone();
        }
        if let Some(scene_state) = current_kitty_scene.as_ref() {
            log::trace!(
                target: "zellij::kitty_images",
                "finished kitty image render for client {client_id}: resident_assets={}, placements={}, explicit_chunks={}, placeholder_renders={}",
                scene_state.resident_asset_generations.len(),
                scene_state.placements.len(),
                current_kitty_chunks.len(),
                current_kitty_placeholder_renders.len(),
            );
        }
        let client_state = self.client_image_state_mut(client_id);
        client_state.set_last_rendered_state(Rc::new(
            LastRenderedImageState::with_kitty_scene_state(
                RenderedImageState {
                    explicit_chunks: current_kitty_chunks,
                    placeholder_renders: current_kitty_placeholder_renders,
                    resident_asset_generations: next_resident_assets,
                },
                current_kitty_scene,
            ),
        ));
    }

    pub fn is_dirty(&self) -> bool {
        self.client_image_states
            .values()
            .any(|client_state| client_state.pending_has_rendered_assets())
            || self.kitty_scene_is_dirty()
    }

    pub fn has_rendered_assets(&self) -> bool {
        self.client_image_states
            .values()
            .any(|client_state| client_state.pending_has_rendered_assets())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_kitty_chunk(
        image_id: u32,
        placement_id: u32,
        cell_y: usize,
        rows: usize,
    ) -> KittyImageChunk {
        KittyImageChunk {
            stable_render_id: image_id as u64,
            image_id,
            placement_id: Some(crate::output::PlacementId::Protocol(placement_id)),
            placement_mode: crate::output::KittyImagePlacementMode::Explicit,
            cell_x: 0,
            cell_y,
            columns: 1,
            rows,
            columns_specified: true,
            rows_specified: true,
            source_x: 0,
            source_y: 0,
            source_width: 10,
            source_height: 10,
            z_index: 0,
            x_offset: 0,
            y_offset: 0,
        }
    }

    #[test]
    fn client_image_render_state_round_trips_last_rendered_snapshot() {
        let mut client_state = ClientImageRenderState::default();
        let expected_state = RenderedImageState {
            resident_asset_generations: HashMap::from([(9, 42)]),
            ..Default::default()
        };
        let expected_snapshot = Rc::new(LastRenderedImageState::with_kitty_scene_state(
            expected_state.clone(),
            Some(KittySceneState::default()),
        ));

        client_state.set_last_rendered_state(Rc::clone(&expected_snapshot));

        assert!(Rc::ptr_eq(
            &client_state.clone_last_rendered_state().unwrap(),
            &expected_snapshot
        ));
        assert_eq!(client_state.last_rendered_image_state(), &expected_state,);
        assert!(client_state
            .last_rendered_state
            .kitty_scene_state()
            .is_some());
    }

    #[test]
    fn prepared_kitty_render_plan_resends_unchanged_placements_that_intersect_damage() {
        let mut kitty_asset_store = KittyAssetStore::default();
        for image_id in [11, 12] {
            kitty_asset_store.insert_asset(
                image_id,
                crate::output::KittyImageData::Png {
                    data: vec![image_id as u8],
                    width: 1,
                    height: 1,
                },
            );
        }
        let top_chunk = test_kitty_chunk(11, 11, 0, 1);
        let bottom_chunk = test_kitty_chunk(12, 12, 5, 1);
        let last_rendered_image_state = RenderedImageState {
            explicit_chunks: vec![top_chunk.clone(), bottom_chunk.clone()],
            resident_asset_generations: HashMap::from([(11, 1), (12, 1)]),
            ..Default::default()
        };

        let planned = PreparedKittyRenderPlan::new(
            &kitty_asset_store,
            &[top_chunk.clone(), bottom_chunk.clone()],
            &[],
            HashMap::from([(0, 1)]),
            &last_rendered_image_state,
            None,
            false,
            false,
        );

        assert!(planned.before_text_vte.is_none());
        match planned.kitty_plan {
            KittyScenePlan::Diff {
                asset_ops,
                placement_ops,
            } => {
                assert!(asset_ops.is_empty());
                assert_eq!(placement_ops.len(), 1);
                match &placement_ops[0] {
                    KittyPlacementOp::PlaceExplicit { key, chunk } => {
                        assert_eq!(key.image_id, top_chunk.image_id);
                        assert_eq!(*chunk, top_chunk);
                    },
                    other => panic!("expected explicit resend, got {other:?}"),
                }
            },
            other => panic!("expected diff kitty plan, got {other:?}"),
        }
    }
}
