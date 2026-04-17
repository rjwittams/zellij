use super::{
    FloatingPanesStack, ImageRenderBundle, KittyImageChunk, KittyPlaceholderRender, SixelImageChunk,
};
use crate::{panes::pane_image_scene::KittyRenderBundle, ClientId};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};
use zellij_utils::pane_size::SizeInPixels;

use crate::panes::sixel::SixelImageStore;

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

pub(crate) struct ClientImageOutput {
    pub sixel_chunks: Vec<SixelImageChunk>,
    pub kitty_chunks_to_serialize: Vec<KittyImageChunk>,
    pub kitty_placeholder_renders_to_serialize: Vec<KittyPlaceholderRender>,
    pub current_kitty_chunks: Vec<KittyImageChunk>,
    pub current_kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
    pub kitty_scene_changed: bool,
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

    pub fn add_sixel_image_chunks_to_client(
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

    pub fn add_sixel_image_chunks_to_multiple_clients(
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

    pub fn add_kitty_image_chunks_to_client(
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

    pub fn add_kitty_image_chunks_to_multiple_clients(
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

    pub fn add_kitty_placeholder_renders_to_client(
        &mut self,
        client_id: ClientId,
        kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
    ) {
        let entry = self.kitty_placeholder_renders.entry(client_id).or_default();
        entry.extend(kitty_placeholder_renders);
    }

    pub fn add_kitty_placeholder_renders_to_multiple_clients(
        &mut self,
        kitty_placeholder_renders: Vec<KittyPlaceholderRender>,
        client_ids: impl Iterator<Item = ClientId>,
    ) {
        for client_id in client_ids {
            let entry = self.kitty_placeholder_renders.entry(client_id).or_default();
            entry.extend(kitty_placeholder_renders.clone());
        }
    }

    pub fn add_kitty_render_bundle_to_client(
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

    pub fn add_kitty_render_bundle_to_multiple_clients(
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

    pub fn take_client_output_for_serialization(
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

    pub fn finish_client_frame(
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

    pub fn with_sixel_image_store<T>(&mut self, f: impl FnOnce(&mut SixelImageStore) -> T) -> T {
        f(&mut self.sixel_image_store.borrow_mut())
    }
}
