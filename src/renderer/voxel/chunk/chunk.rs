use crate::renderer::voxel::octree::{point::Point3D, tree::Tree};

pub struct Voxel;

impl Voxel {
    pub fn new_color(r: u8, g: u8, b: u8) -> u16 {
        let r = (r & 0x1F) as u16;
        let g = (g & 0x3F) as u16;
        let b = (b & 0x1F) as u16;
        println!("{r}, {g}, {b}");
        ((r & 0x1F) | ((g & 0x3F) << 5) | ((b & 0x1F) << 11)) as u16
    }
}

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
    pub fn set_voxel(&mut self, ty: u16, x: usize, y: usize, z: usize) {
        let index = x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE;
        self.voxels[index] = ty;
    }

    pub fn get_raw_voxels(&self) -> &[u8] {
        bytemuck::cast_slice(self.voxels.as_slice())
    }
}
