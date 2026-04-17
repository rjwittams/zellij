use super::{KittyImageChunk, KittyImageData, KittyPlaceholderRender};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct KittyPlacementKey {
    pub image_id: u32,
    pub placement_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlannedKittyPlacement {
    Explicit {
        key: KittyPlacementKey,
        chunk: KittyImageChunk,
    },
    Placeholder {
        key: KittyPlacementKey,
        render: KittyPlaceholderRender,
    },
}

impl PlannedKittyPlacement {
    pub(crate) fn key(&self) -> KittyPlacementKey {
        match self {
            PlannedKittyPlacement::Explicit { key, .. } => *key,
            PlannedKittyPlacement::Placeholder { key, .. } => *key,
        }
    }

    pub(crate) fn image_id(&self) -> u32 {
        self.key().image_id
    }

    pub(crate) fn image_data(&self) -> &KittyImageData {
        match self {
            PlannedKittyPlacement::Explicit { chunk, .. } => &chunk.image_data,
            PlannedKittyPlacement::Placeholder { render, .. } => &render.image_data,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct KittySceneState {
    pub resident_assets: BTreeMap<u32, KittyImageData>,
    pub placements: BTreeMap<KittyPlacementKey, PlannedKittyPlacement>,
}

impl KittySceneState {
    pub(crate) fn insert_placement(&mut self, placement: PlannedKittyPlacement) {
        self.resident_assets
            .insert(placement.image_id(), placement.image_data().clone());
        self.placements.insert(placement.key(), placement);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum KittyAssetOp {
    EnsureResident {
        image_id: u32,
        image_data: KittyImageData,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum KittyPlacementOp {
    Delete {
        key: KittyPlacementKey,
    },
    PlaceExplicit {
        key: KittyPlacementKey,
        chunk: KittyImageChunk,
    },
    PlacePlaceholder {
        key: KittyPlacementKey,
        render: KittyPlaceholderRender,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum KittyScenePlan {
    Diff {
        asset_ops: Vec<KittyAssetOp>,
        placement_ops: Vec<KittyPlacementOp>,
    },
    FullResetAndResend {
        desired: KittySceneState,
    },
}

pub(crate) fn plan_kitty_scene(
    assumed: &KittySceneState,
    desired: &KittySceneState,
) -> KittyScenePlan {
    let changed_asset_ids: BTreeSet<u32> = desired
        .resident_assets
        .iter()
        .filter_map(|(&image_id, desired_data)| {
            match assumed.resident_assets.get(&image_id) {
                Some(existing_data) if existing_data == desired_data => None,
                _ => Some(image_id),
            }
        })
        .collect();

    let mut asset_ops = vec![];
    for image_id in &changed_asset_ids {
        let image_data = desired
            .resident_assets
            .get(image_id)
            .expect("changed asset must exist in desired scene")
            .clone();
        asset_ops.push(KittyAssetOp::EnsureResident {
            image_id: *image_id,
            image_data,
        });
    }

    let mut placement_ops = vec![];
    for key in assumed.placements.keys() {
        if !desired.placements.contains_key(key) {
            placement_ops.push(KittyPlacementOp::Delete { key: *key });
        }
    }

    let mut replacement_deletes = vec![];
    let mut placement_creates = vec![];
    for (key, desired_placement) in &desired.placements {
        let asset_changed = changed_asset_ids.contains(&key.image_id);
        match assumed.placements.get(key) {
            Some(existing_placement) if !asset_changed && existing_placement == desired_placement => {
            },
            Some(_) => {
                replacement_deletes.push(KittyPlacementOp::Delete { key: *key });
                placement_creates.push(place_placement_op(desired_placement));
            },
            None => {
                placement_creates.push(place_placement_op(desired_placement));
            },
        }
    }
    placement_ops.extend(replacement_deletes);
    placement_ops.extend(placement_creates);

    KittyScenePlan::Diff {
        asset_ops,
        placement_ops,
    }
}

fn place_placement_op(placement: &PlannedKittyPlacement) -> KittyPlacementOp {
    match placement {
        PlannedKittyPlacement::Explicit { key, chunk } => KittyPlacementOp::PlaceExplicit {
            key: *key,
            chunk: chunk.clone(),
        },
        PlannedKittyPlacement::Placeholder { key, render } => KittyPlacementOp::PlacePlaceholder {
            key: *key,
            render: render.clone(),
        },
    }
}
