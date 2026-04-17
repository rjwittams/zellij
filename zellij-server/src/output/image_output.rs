use super::{
    image_fragment::{
        visible_image_fragments, ImageFragment, KittyExplicitFragment, KittyPlaceholderFragment,
        PreparedAfterTextImages, PreparedImageOutput, SixelFragment,
    },
    kitty_diff::{
        plan_kitty_scene, KittyAssetOp, KittyPlacementKey, KittyPlacementOp, KittyScenePlan,
        KittySceneState, PlannedKittyPlacement,
    },
    vte_goto_instruction, FloatingPanesStack, KittyImageChunk, KittyPlaceholderRender,
    PaneImageRenderOutput, SixelImageChunk,
};
use crate::panes::kitty::KittyImageState;
use crate::panes::kitty_asset_store::KittyAssetStore;
use crate::panes::pane_image_scene::KittyDamageRedraw;
use crate::panes::sixel::SixelImageStore;
use crate::ClientId;
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
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
}

#[derive(Clone, Debug, Default)]
struct LastRenderedKittyScene {
    chunks: Vec<KittyImageChunk>,
    placeholder_renders: Vec<KittyPlaceholderRender>,
}

#[derive(Clone, Debug, Default)]
struct ClientImageState {
    current: CurrentImageState,
    last_rendered_kitty: LastRenderedKittyScene,
    resident_kitty_assets: HashMap<u32, crate::output::KittyImageData>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ImageOutput {
    client_image_states: HashMap<ClientId, ClientImageState>,
    pub(crate) sixel_image_store: Rc<RefCell<SixelImageStore>>,
    pub(crate) kitty_asset_store: Rc<RefCell<KittyAssetStore>>,
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
            current_kitty_chunks,
            current_kitty_placeholder_renders,
        } = self;
        let err_context = || "failed to serialize image chunks".to_string();
        let mut sixel_vte: Option<String> = None;
        for fragment in fragments {
            match fragment {
                ImageFragment::Sixel(sixel_fragment) => {
                    let sixel_chunk = sixel_fragment.chunk;
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

        vte_output.push_str(&image_output.serialize_kitty_plan(&kitty_plan));

        image_output.finish_render_body_for_client(
            client_id,
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
        character_cell_size: Rc<RefCell<Option<SizeInPixels>>>,
    ) -> Self {
        Self {
            sixel_image_store,
            kitty_asset_store,
            character_cell_size,
            ..Default::default()
        }
    }

    fn client_image_state_mut(&mut self, client_id: ClientId) -> &mut ClientImageState {
        self.client_image_states.entry(client_id).or_default()
    }

    fn kitty_scene_state_from_rendered(
        resident_kitty_assets: &HashMap<u32, crate::output::KittyImageData>,
        chunks: &[KittyImageChunk],
        placeholder_renders: &[KittyPlaceholderRender],
        kitty_asset_store: &KittyAssetStore,
    ) -> Option<KittySceneState> {
        let mut scene = KittySceneState {
            resident_assets: resident_kitty_assets
                .iter()
                .map(|(image_id, image_data)| (*image_id, image_data.clone()))
                .collect(),
            ..Default::default()
        };
        let mut referenced_asset_ids: BTreeSet<u32> = BTreeSet::new();
        for chunk in chunks {
            let placement_id = chunk.placement_id?;
            referenced_asset_ids.insert(chunk.image_id);
            scene.insert_placement(PlannedKittyPlacement::Explicit {
                key: KittyPlacementKey {
                    image_id: chunk.image_id,
                    placement_id,
                },
                chunk: chunk.clone(),
            });
        }
        for render in placeholder_renders {
            let placement_id = render.placement_id?;
            referenced_asset_ids.insert(render.image_id);
            scene.insert_placement(PlannedKittyPlacement::Placeholder {
                key: KittyPlacementKey {
                    image_id: render.image_id,
                    placement_id,
                },
                render: render.clone(),
            });
        }
        for image_id in referenced_asset_ids {
            let image_data = kitty_asset_store.image_data(image_id)?;
            scene.insert_asset(image_id, image_data);
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
                    key.placement_id,
                ));
            }
        }
        (!vte_output.is_empty()).then_some(vte_output)
    }

    fn serialize_kitty_plan(&mut self, kitty_plan: &KittyScenePlan) -> String {
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
                            image_data,
                        } => vte_output.push_str(&KittyImageState::serialize_image_data(
                            *image_id, image_data,
                        )),
                    }
                }
                for placement_op in placement_ops {
                    match placement_op {
                        KittyPlacementOp::Delete { .. } => {},
                        KittyPlacementOp::PlaceExplicit { key, chunk } => vte_output.push_str(
                            &KittyImageState::serialize_explicit_placement(chunk, key.placement_id),
                        ),
                        KittyPlacementOp::PlacePlaceholder { key, render } => {
                            vte_output.push_str(&KittyImageState::serialize_placeholder_render(
                                render,
                                key.placement_id,
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
            } => {
                let mut vte_output = String::new();
                let kitty_asset_store = self.kitty_asset_store.borrow();
                vte_output.push_str(&KittyImageState::serialize_chunks_with_asset_store(
                    explicit_chunks,
                    &kitty_asset_store,
                ));
                vte_output.push_str(
                    &KittyImageState::serialize_placeholder_renders_with_asset_store(
                        placeholder_renders,
                        &kitty_asset_store,
                    ),
                );
                vte_output
            },
        }
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
                .map(|chunk| ImageFragment::Sixel(SixelFragment { chunk })),
        );
        fragments.extend(
            pane_image_output
                .kitty_scene
                .explicit_chunks
                .iter()
                .cloned()
                .map(|chunk| ImageFragment::KittyExplicit(KittyExplicitFragment { chunk })),
        );
        fragments.extend(
            pane_image_output
                .kitty_scene
                .placeholder_renders
                .iter()
                .cloned()
                .map(|render| ImageFragment::KittyPlaceholder(KittyPlaceholderFragment { render })),
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
            client_state.current.kitty_chunks != client_state.last_rendered_kitty.chunks
                || client_state.current.kitty_placeholder_renders
                    != client_state.last_rendered_kitty.placeholder_renders
        })
    }

    pub fn set_last_rendered_kitty_chunks(
        &mut self,
        last_rendered_kitty_chunks: HashMap<ClientId, Vec<KittyImageChunk>>,
        last_rendered_kitty_placeholder_renders: HashMap<ClientId, Vec<KittyPlaceholderRender>>,
    ) {
        let kitty_asset_store = self.kitty_asset_store.clone();
        for (client_id, chunks) in last_rendered_kitty_chunks {
            let resident_assets: Vec<_> = chunks
                .iter()
                .filter_map(|chunk| {
                    kitty_asset_store
                        .borrow()
                        .image_data(chunk.image_id)
                        .map(|image_data| (chunk.image_id, image_data))
                })
                .collect();
            let client_state = self.client_image_state_mut(client_id);
            client_state.last_rendered_kitty.chunks = chunks.clone();
            for (image_id, image_data) in resident_assets {
                client_state.resident_kitty_assets.insert(image_id, image_data);
            }
        }
        for (client_id, placeholder_renders) in last_rendered_kitty_placeholder_renders {
            let resident_assets: Vec<_> = placeholder_renders
                .iter()
                .filter_map(|render| {
                    kitty_asset_store
                        .borrow()
                        .image_data(render.image_id)
                        .map(|image_data| (render.image_id, image_data))
                })
                .collect();
            let client_state = self.client_image_state_mut(client_id);
            client_state.last_rendered_kitty.placeholder_renders = placeholder_renders.clone();
            for (image_id, image_data) in resident_assets {
                client_state.resident_kitty_assets.insert(image_id, image_data);
            }
        }
    }

    pub fn take_last_rendered_kitty_chunks(
        &mut self,
    ) -> (
        HashMap<ClientId, Vec<KittyImageChunk>>,
        HashMap<ClientId, Vec<KittyPlaceholderRender>>,
    ) {
        let mut last_rendered_kitty_chunks = HashMap::new();
        let mut last_rendered_kitty_placeholder_renders = HashMap::new();
        for (client_id, client_state) in &mut self.client_image_states {
            if !client_state.last_rendered_kitty.chunks.is_empty() {
                last_rendered_kitty_chunks.insert(
                    *client_id,
                    std::mem::take(&mut client_state.last_rendered_kitty.chunks),
                );
            }
            if !client_state
                .last_rendered_kitty
                .placeholder_renders
                .is_empty()
            {
                last_rendered_kitty_placeholder_renders.insert(
                    *client_id,
                    std::mem::take(&mut client_state.last_rendered_kitty.placeholder_renders),
                );
            }
            client_state.resident_kitty_assets.clear();
        }
        (
            last_rendered_kitty_chunks,
            last_rendered_kitty_placeholder_renders,
        )
    }

    fn add_changed_rects_to_client(
        &mut self,
        client_id: ClientId,
        changed_rects: HashMap<usize, usize>,
    ) {
        let client_state = self.client_image_state_mut(client_id);
        for (start_row, line_count) in changed_rects {
            client_state
                .current
                .changed_rects
                .entry(start_row)
                .and_modify(|current_line_count| {
                    *current_line_count = (*current_line_count).max(line_count);
                })
                .or_insert(line_count);
        }
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
            match fragment {
                ImageFragment::Sixel(sixel_fragment) => {
                    client_state.current.sixel_chunks.push(sixel_fragment.chunk)
                },
                ImageFragment::KittyExplicit(kitty_explicit_fragment) => client_state
                    .current
                    .kitty_chunks
                    .push(kitty_explicit_fragment.chunk),
                ImageFragment::KittyPlaceholder(kitty_placeholder_fragment) => client_state
                    .current
                    .kitty_placeholder_renders
                    .push(kitty_placeholder_fragment.render),
            }
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
                match fragment {
                    ImageFragment::Sixel(sixel_fragment) => {
                        client_state.current.sixel_chunks.push(sixel_fragment.chunk)
                    },
                    ImageFragment::KittyExplicit(kitty_explicit_fragment) => client_state
                        .current
                        .kitty_chunks
                        .push(kitty_explicit_fragment.chunk),
                    ImageFragment::KittyPlaceholder(kitty_placeholder_fragment) => client_state
                        .current
                        .kitty_placeholder_renders
                        .push(kitty_placeholder_fragment.render),
                }
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
            resident_kitty_assets,
            last_rendered_kitty_chunks,
            last_rendered_kitty_placeholder_renders,
        ) = {
            let client_state = self.client_image_state_mut(client_id);
            (
                std::mem::take(&mut client_state.current.sixel_chunks),
                std::mem::take(&mut client_state.current.kitty_chunks),
                std::mem::take(&mut client_state.current.kitty_placeholder_renders),
                std::mem::take(&mut client_state.current.changed_rects),
                client_state.resident_kitty_assets.clone(),
                client_state.last_rendered_kitty.chunks.clone(),
                client_state.last_rendered_kitty.placeholder_renders.clone(),
            )
        };

        let kitty_damage_redraw = KittyDamageRedraw::from_changed_rects(changed_rects);
        let empty_resident_assets = HashMap::new();
        let kitty_asset_store = self.kitty_asset_store.borrow();
        let desired_kitty_scene = Self::kitty_scene_state_from_rendered(
            &empty_resident_assets,
            &current_kitty_chunks,
            &current_kitty_placeholder_renders,
            &kitty_asset_store,
        );
        let assumed_kitty_scene = if pre_vte_clears_display {
            Some(KittySceneState {
                resident_assets: resident_kitty_assets
                    .iter()
                    .map(|(image_id, image_data)| (*image_id, image_data.clone()))
                    .collect(),
                ..Default::default()
            })
        } else {
            Self::kitty_scene_state_from_rendered(
                &resident_kitty_assets,
                &last_rendered_kitty_chunks,
                &last_rendered_kitty_placeholder_renders,
                &kitty_asset_store,
            )
        };
        let kitty_plan = match (assumed_kitty_scene, desired_kitty_scene) {
            (Some(assumed_kitty_scene), Some(desired_kitty_scene)) => {
                let mut kitty_plan = plan_kitty_scene(&assumed_kitty_scene, &desired_kitty_scene);
                if let KittyScenePlan::Diff {
                    asset_ops: _,
                    placement_ops,
                } = &mut kitty_plan
                {
                    let place_keys: BTreeSet<KittyPlacementKey> = placement_ops
                        .iter()
                        .filter_map(|placement_op| match placement_op {
                            KittyPlacementOp::PlaceExplicit { key, .. }
                            | KittyPlacementOp::PlacePlaceholder { key, .. } => Some(*key),
                            KittyPlacementOp::Delete { .. } => None,
                        })
                        .collect();
                    for (key, placement) in desired_kitty_scene.placements.iter() {
                        if place_keys.contains(key)
                            || !Self::placement_intersects_damage(placement, &kitty_damage_redraw)
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
            _ => Self::render_full_reset_plan(
                &current_kitty_chunks,
                &current_kitty_placeholder_renders,
            ),
        };

        let mut fragments = vec![];
        fragments.extend(
            sixel_chunks
                .iter()
                .cloned()
                .map(|chunk| ImageFragment::Sixel(SixelFragment { chunk })),
        );
        let before_text_vte = match &kitty_plan {
            KittyScenePlan::Diff {
                asset_ops: _,
                placement_ops,
            } => Self::serialize_kitty_delete_ops(placement_ops),
            KittyScenePlan::FullResetAndResend { .. } => {
                (!pre_vte_clears_display).then(kitty_clear_before_text_vte)
            },
        };

        PreparedImageOutput {
            before_text_vte,
            after_text: PreparedAfterTextImages {
                client_id,
                fragments,
                kitty_plan,
                current_kitty_chunks,
                current_kitty_placeholder_renders,
            },
        }
    }

    pub(super) fn finish_render_body_for_client(
        &mut self,
        client_id: ClientId,
        current_kitty_chunks: Vec<KittyImageChunk>,
        current_kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
        kitty_plan: &KittyScenePlan,
    ) {
        let previous_resident_assets = self
            .client_image_states
            .get(&client_id)
            .map(|client_state| client_state.resident_kitty_assets.clone())
            .unwrap_or_default();
        let mut next_resident_assets = HashMap::new();
        match kitty_plan {
            KittyScenePlan::Diff {
                asset_ops,
                placement_ops: _,
            } => {
                next_resident_assets = previous_resident_assets;
                for asset_op in asset_ops {
                    match asset_op {
                        KittyAssetOp::EnsureResident {
                            image_id,
                            image_data,
                        } => {
                            next_resident_assets.insert(*image_id, image_data.clone());
                        },
                    }
                }
            },
            KittyScenePlan::FullResetAndResend {
                explicit_chunks,
                placeholder_renders,
            } => {
                let kitty_asset_store = self.kitty_asset_store.borrow();
                for chunk in explicit_chunks {
                    if let Some(image_data) = kitty_asset_store.image_data(chunk.image_id) {
                        next_resident_assets.insert(chunk.image_id, image_data);
                    }
                }
                for render in placeholder_renders {
                    if let Some(image_data) = kitty_asset_store.image_data(render.image_id) {
                        next_resident_assets.insert(render.image_id, image_data);
                    }
                }
            },
        }
        let client_state = self.client_image_state_mut(client_id);
        client_state.resident_kitty_assets = next_resident_assets;
        client_state.last_rendered_kitty.chunks = current_kitty_chunks;
        client_state.last_rendered_kitty.placeholder_renders = current_kitty_placeholder_renders;
    }

    pub fn is_dirty(&self) -> bool {
        self.client_image_states.values().any(|client_state| {
            !client_state.current.sixel_chunks.is_empty()
                || !client_state.current.kitty_chunks.is_empty()
                || !client_state.current.kitty_placeholder_renders.is_empty()
        }) || self.kitty_scene_is_dirty()
    }

    pub fn has_rendered_assets(&self) -> bool {
        self.client_image_states.values().any(|client_state| {
            !client_state.current.sixel_chunks.is_empty()
                || !client_state.current.kitty_chunks.is_empty()
                || !client_state.current.kitty_placeholder_renders.is_empty()
        })
    }
}
