#[derive(Clone, Copy, Default)]
pub(super) struct BlockListLayout {
    leading: bool,
}

impl BlockListLayout {
    pub(super) fn new(leading: bool) -> Self {
        Self { leading }
    }

    pub(super) fn item_for_block(self, block: usize) -> usize {
        block + usize::from(self.leading)
    }

    pub(super) fn block_for_item(self, item: usize) -> Option<usize> {
        item.checked_sub(usize::from(self.leading))
    }

    pub(super) fn item_count(self, blocks: usize) -> usize {
        self.item_for_block(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::BlockListLayout;

    #[test]
    fn no_header_keeps_block_indexes_and_empty_count() {
        let layout = BlockListLayout::default();
        assert_eq!(layout.item_count(0), 0);
        for block in 0..20 {
            assert_eq!(layout.item_for_block(block), block);
            assert_eq!(layout.block_for_item(block), Some(block));
        }
    }

    #[test]
    fn header_has_no_block_and_empty_document_has_one_item() {
        let layout = BlockListLayout::new(true);
        assert_eq!(layout.item_count(0), 1);
        assert_eq!(layout.block_for_item(0), None);
        for block in 0..20 {
            let item = layout.item_for_block(block);
            assert_eq!(item, block + 1);
            assert_eq!(layout.block_for_item(item), Some(block));
            assert_eq!(layout.item_count(block + 1), item + 1);
        }
    }

    #[test]
    fn append_ranges_leave_header_and_unchanged_blocks_alone() {
        for leading in [false, true] {
            let layout = BlockListLayout::new(leading);
            let old_count = layout.item_count(3);
            let unchanged = layout.item_for_block(2);
            assert_eq!(layout.block_for_item(unchanged), Some(2));
            assert_eq!(old_count - unchanged, 1);
            assert_eq!(layout.item_count(5) - old_count, 2);
        }
    }
}
