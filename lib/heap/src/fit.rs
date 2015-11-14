//! An implementation of a first fit allocator.
//!
//! The allocator uses a list of free blocks that can be allocated. When an
//! allocation is performed and the block is bigger than the requested size,
//! the block is splitted in 2 blocks: one used to fulfill the allocation, the
//! other is free.
//!
//! When a block is deallocated, the allocator tries to merge it with adjacent
//! blocks. The header of a block contains the necessary information to
//! retrieve the previous and next block if they exist. In order to access the
//! previous block, each block contains a footer that indicates the offset to
//! get the header.
