use super::point::Point3D;

/// Represent a cell of an octree.
#[derive(Default, Debug, Clone, Copy)]
pub struct Cell {
    /// The size of the [cell](Cell) from the center.
    pub extend: Point3D,
    /// The position of the bottom left edge of the [cell](Cell).
    pub position: Point3D,
}

impl Cell {
    /// Create a new [cell](Cell)
    ///
    /// # Arguments
    ///
    /// * `center` - The position of the bottom left edge of the [cell](Cell).
    /// * `extend` - The size of each positive side of the [cell](Cell) from the bottom left edge.
    ///
    pub fn new<P: Into<Point3D>, E: Into<Point3D>>(position: P, extend: E) -> Self {
        Self {
            position: position.into(),
            extend: extend.into(),
        }
    }

    /// Subdivide the current [cell](Cell) into 8 [cells](Cell)
    /// with equal size.
    ///
    pub fn subdivide(&self) -> Option<[Self; 8]> {
        if self.extend == 1 {
            return None;
        }

        let mut childs = [Self::default(); 8];

        let new_extend = self.extend / 2;
        let new_position: [Point3D; 8] = [
            self.position,
            self.position + Into::<Point3D>::into((0, 0, new_extend.z)),
            self.position + Into::<Point3D>::into((0, new_extend.y, 0)),
            self.position + Into::<Point3D>::into((0, new_extend.y, new_extend.z)),
            self.position + Into::<Point3D>::into((new_extend.x, 0, 0)),
            self.position + Into::<Point3D>::into((new_extend.x, 0, new_extend.z)),
            self.position + Into::<Point3D>::into((new_extend.x, new_extend.y, 0)),
            self.position + Into::<Point3D>::into((new_extend.x, new_extend.y, new_extend.z)),
        ];

        for i in 0..8 {
            childs[i].position = new_position[i];
            childs[i].extend = new_extend;
        }

        Some(childs)
    }

    /// Check if the current [cell](Cell) contains
    /// the given [point](Point3D).
    ///
    pub fn contains<P: Into<Point3D>>(&self, point: P) -> bool {
        let point: Point3D = point.into();
        point.x >= self.position.x
            && point.y >= self.position.y
            && point.z >= self.position.z
            && point.x < self.position.x + self.extend.x
            && point.y < self.position.y + self.extend.y
            && point.z < self.position.z + self.extend.z
    }
}
