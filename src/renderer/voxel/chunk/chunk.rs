use std::mem::size_of;

use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferUsages, Device};

use crate::renderer::voxel::octree::{point::Point3D, tree::Tree};

pub struct Chunk<const SIZE: usize> {
    /// The position of the chunk in the world.
    pos: Point3D,
    /// The octree of the chunk.
    tree: Tree,
    /// The voxels of the chunk.
    voxels: Vec<u16>,
}

impl<const CHUNK_SIZE: usize> Chunk<CHUNK_SIZE> {
    /// Create a new [Chunk].
    pub fn new<P: Into<Point3D>>(pos: P) -> Self {
        let pos: Point3D = pos.into();

        let voxels = vec![0u16; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

        Self {
            pos,
            tree: Tree::new(CHUNK_SIZE as u32),
            voxels,
        }
    }

    pub fn get_tree(&self) -> &Tree {
        &self.tree
    }

    pub fn get_voxels(&self) -> &[u16] {
        &self.voxels
    }

    /// Set the type of a voxel at a given position
    /// in the current chunk.
    ///
    /// # Arguments
    ///
    /// * `ty` - The voxel type.
    /// * `x` - The position of the voxel in `x` axis.
    /// * `y` - The position of the voxel in `y` axis.
    /// * `z` - The position of the voxel in `z` axis.
    ///
    pub fn set_voxel(&mut self, ty: u16, x: u32, y: u32, z: u32) {
        self.voxels[0];
    }

    pub fn get_raw_voxels(&self) -> &[u8] {
        bytemuck::cast_slice(self.voxels.as_slice())
    }

    pub fn get_pos(&self) -> &Point3D {
        &self.pos
    }

    pub fn add_block<P: Into<Point3D>>(&mut self, at: P) {
        self.tree.set_block_state(at, true, Default::default());
    }

    pub fn rem_block<P: Into<Point3D>>(&mut self, at: P) {
        self.tree.set_block_state(at, false, Default::default());
    }
}
