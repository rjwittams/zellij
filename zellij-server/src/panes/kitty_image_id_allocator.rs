#[derive(Clone, Debug)]
pub(crate) struct KittyHostImageIdAllocator {
    next_image_id: u32,
}

impl Default for KittyHostImageIdAllocator {
    fn default() -> Self {
        Self { next_image_id: 1 }
    }
}

impl KittyHostImageIdAllocator {
    pub(crate) fn new(next_image_id: u32) -> Self {
        Self {
            next_image_id: next_image_id.max(1),
        }
    }

    pub(crate) fn next(&mut self) -> u32 {
        let image_id = self.next_image_id;
        self.next_image_id = self.next_image_id.wrapping_add(1).max(1);
        image_id
    }
}

#[cfg(test)]
mod tests {
    use super::KittyHostImageIdAllocator;

    #[test]
    fn host_image_id_allocator_returns_non_zero_ids() {
        let mut allocator = KittyHostImageIdAllocator::new(0);

        assert_eq!(allocator.next(), 1);
    }

    #[test]
    fn host_image_id_allocator_returns_monotonic_ids() {
        let mut allocator = KittyHostImageIdAllocator::new(1);

        assert_eq!(allocator.next(), 1);
        assert_eq!(allocator.next(), 2);
        assert_eq!(allocator.next(), 3);
    }

    #[test]
    fn host_image_id_allocator_wraps_without_returning_zero() {
        let mut allocator = KittyHostImageIdAllocator::new(u32::MAX);

        assert_eq!(allocator.next(), u32::MAX);
        assert_eq!(allocator.next(), 1);
    }
}
