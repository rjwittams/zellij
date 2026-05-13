use super::{KittyImageChunk, PlacementId};
use crate::panes::pane_image_scene::LogicalPlacementId;
use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) const REAL_PLACEMENT_ID_MIN: u32 = 1;
pub(crate) const REAL_PLACEMENT_ID_MAX: u32 = 0x00ff_ffff;
pub(crate) const EXPLICIT_FRAGMENT_PLACEMENT_ID_MIN: u32 = 0x0100_0000;
pub(crate) const EXPLICIT_FRAGMENT_PLACEMENT_ID_SPAN: u32 = 0xff00_0000;

pub(crate) trait PlacementIdAllocator {
    fn allocate_placeholder_placement_id(&self) -> LogicalPlacementId;
    fn allocate_explicit_placement_id(&self) -> LogicalPlacementId;
    fn explicit_fragment_placement_id(
        &self,
        base_placement_id: LogicalPlacementId,
        fragment: &KittyImageChunk,
    ) -> PlacementId;
}

pub(crate) struct AtomicPlacementIdAllocator {
    next_real_placement_id: AtomicU32,
}

impl AtomicPlacementIdAllocator {
    pub(crate) const fn new() -> Self {
        Self {
            next_real_placement_id: AtomicU32::new(REAL_PLACEMENT_ID_MIN),
        }
    }

    fn allocate_real_placement_id(&self) -> LogicalPlacementId {
        let raw = self.next_real_placement_id.fetch_add(1, Ordering::Relaxed);
        let placement_id = raw.saturating_sub(1) % REAL_PLACEMENT_ID_MAX + REAL_PLACEMENT_ID_MIN;
        LogicalPlacementId(placement_id as u64)
    }
}

impl PlacementIdAllocator for AtomicPlacementIdAllocator {
    fn allocate_placeholder_placement_id(&self) -> LogicalPlacementId {
        self.allocate_real_placement_id()
    }

    fn allocate_explicit_placement_id(&self) -> LogicalPlacementId {
        self.allocate_real_placement_id()
    }

    fn explicit_fragment_placement_id(
        &self,
        base_placement_id: LogicalPlacementId,
        fragment: &KittyImageChunk,
    ) -> PlacementId {
        let mut hash = base_placement_id.0 ^ 0x9e37_79b9_7f4a_7c15;
        let fields = [
            fragment.cell_x as u64,
            fragment.cell_y as u64,
            fragment.columns as u64,
            fragment.rows as u64,
            fragment.source_x as u64,
            fragment.source_y as u64,
            fragment.source_width as u64,
            fragment.source_height as u64,
            fragment.x_offset as u64,
            fragment.y_offset as u64,
        ];
        for field in fields {
            hash ^= field
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(hash << 6)
                .wrapping_add(hash >> 2);
        }
        let placement_id = EXPLICIT_FRAGMENT_PLACEMENT_ID_MIN
            + (hash as u32 % EXPLICIT_FRAGMENT_PLACEMENT_ID_SPAN);
        PlacementId::Synthetic(placement_id)
    }
}

static GLOBAL_PLACEMENT_ID_ALLOCATOR: AtomicPlacementIdAllocator =
    AtomicPlacementIdAllocator::new();

pub(crate) fn placement_id_allocator() -> &'static AtomicPlacementIdAllocator {
    &GLOBAL_PLACEMENT_ID_ALLOCATOR
}
