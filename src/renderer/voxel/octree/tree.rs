use super::{cell::Cell, point::Point3D};

#[derive(Debug, Default)]
pub enum SearchParameter {
    /// No custom search
    #[default]
    None,
    /// Use a custom search
    Custom {
        /// The cell at where the search need to start from.
        cell: Cell,
        /// The offset
        offset: usize,
    },
}

/// Represent an Octree.
pub struct Tree {
    /// The raw [tree](Tree) data.
    data: Vec<u8>,
    /// The amount of block in each axis
    size: i32,
}

impl Tree {
    pub fn estimated_size(size: u32) -> usize {
        // Calculate the depth of the current tree
        let depth = (size as f32).log2() as u32;

        // Calculate the amount of data for the given size
        let len = ((Self::pow8(depth + 1) - 1) / 7 - 1) / 8;

        len as usize
    }

    pub fn estimated_size_aligned(size: u32, alignement: usize) -> usize {
        (Self::estimated_size(size) + (alignement - 1)) & (!(alignement - 1))
    }

    /// Create a new [tree](Tree).
    ///
    /// # Arguments
    ///
    /// * `size` - The amount of block in each axis.
    ///
    pub fn new(size: u32) -> Self {
        let len = Self::estimated_size_aligned(size, 256);

        Self {
            size: size as i32,
            data: vec![0u8; len as usize],
        }
    }

    /// Very fast `8` power of `x` function.
    ///
    /// # Arguments
    ///
    /// * `exp` - The power exponent.
    ///
    #[inline]
    fn pow8(exp: u32) -> u32 {
        1 << (3 * exp)
    }

    /// Set the state of a block at a given position.
    ///
    /// # Arguments
    ///
    /// * `at` - The position of the block
    /// * `state` - The new state
    ///
    pub fn set_block_state<P: Into<Point3D>>(
        &mut self,
        at: P,
        state: bool,
        search: SearchParameter,
    ) {
        let (cells, offset) = match search {
            SearchParameter::None => (Cell::new(0, self.size).subdivide(), 0),
            SearchParameter::Custom { cell, offset } => (cell.subdivide(), offset),
        };

        if let Some(cells) = cells {
            let point = at.into();

            for (cell_index, cell) in cells.iter().enumerate() {
                if cells[cell_index].contains(point) {
                    #[cfg(test)]
                    println!(
                        "Set point at offset {} at pos [{:?}] extend [{:?}]",
                        offset, cells[cell_index].position, cells[cell_index].extend
                    );

                    let next_offset = offset * 8 + (cell_index + 1);
                    let mask = 1 << cell_index;

                    self.data[offset] |= mask;

                    if next_offset >= self.data.len() {
                        self.data[offset] =
                            (self.data[offset] & !mask) | ((state as u8) << cell_index);
                    }

                    return self.set_block_state(
                        point,
                        state,
                        SearchParameter::Custom {
                            cell: *cell,
                            offset: next_offset,
                        },
                    );
                }
            }
        }
    }

    /// Get the state of a block at a given position.
    ///
    /// # Arguments
    ///
    /// * `at` - The position of the block
    /// * `search` - The search parameter.
    ///
    /// # Return
    ///
    /// The state of the block a the given position `at`.
    ///
    pub fn get_block_state<P: Into<Point3D>>(&mut self, at: P, search: SearchParameter) -> bool {
        let (cells, offset) = match search {
            SearchParameter::None => (Cell::new(0, self.size).subdivide(), 0),
            SearchParameter::Custom { cell, offset } => (cell.subdivide(), offset),
        };

        if let Some(cells) = cells {
            let point = at.into();

            for (cell_index, cell) in cells.iter().enumerate() {
                if cells[cell_index].contains(point) {
                    let mask = 1 << cell_index;

                    if self.data[offset] & mask != mask {
                        return false;
                    }

                    return self.get_block_state(
                        point,
                        SearchParameter::Custom {
                            cell: *cell,
                            offset: offset * 8 + (cell_index + 1),
                        },
                    );
                }
            }
        }

        // At here we can retrive the index of the block
        // with: offset - self.data.len()
        true
    }

    /// Retrieve the raw data of the current [tree](Tree).
    pub fn raw_data(&self) -> &[u8] {
        &self.data
    }

    /// Retrieve the size of the tree.
    pub fn get_size(&self) -> u32 {
        self.size as u32
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_octree() {
        let mut tree = Tree::new(8);
        // for i in 0..16 {
        //     for j in 0..16 {
        tree.set_block_state((0, 0, 0), true, Default::default());
        tree.set_block_state((0, 6, 0), true, Default::default());
        //     }
        // }

        let raw_data = tree.raw_data();
        println!("{:?}", raw_data);

        let data: &[u32] = bytemuck::cast_slice(tree.raw_data());

        let mut raw = Vec::<u8>::new();

        for i in 0..(data.len() * 4) {
            let value = (data[i / 4] >> ((i % 4) * 8)) & 0xFF;
            raw.push(value as u8);
        }

        assert_eq!(raw_data, raw);
        println!("Len is : {}", raw_data.len());

        let mut offset = 0;

        for _ in 0..3 {
            println!("Offset: {:?}", offset);

            // Retrieve the value of the current cell.
            // This value contains 8 bits that represent
            // the 8 subdivision of the current cell. If
            // a bit is equal to 1 it mean that he have
            // childs otherwise it dosen't have child.
            let value = (data[offset / 4] >> ((offset % 4) * 8)) & 0xFF;

            let index = 2;

            // Check if the current cell have child for the
            // index 0...
            if ((value >> index) & 0x1) != 1 {
                println!("Don't have childs !");
                break;
            }

            // Go to the next octree child...
            offset = offset * 8 + (index + 1);
        }
    }
}
