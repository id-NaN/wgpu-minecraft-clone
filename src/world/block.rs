use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
pub struct Block {
    block_type: u16,
}

impl Block {
    pub fn new(block_type: u16) -> Self {
        Self { block_type }
    }

    pub fn air() -> Self {
        Self { block_type: 0 }
    }

    pub fn block_type(&self) -> u16 {
        self.block_type
    }
}
