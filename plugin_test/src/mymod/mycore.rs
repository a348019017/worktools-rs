use std::fmt::Debug;

pub mod mycore {

    /// Vertex position bounding box.
    pub type BoundingBox = Bounds<Vec3>;

    /// The minimum and maximum values for a generic accessor.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Bounds<T> {
        /// Minimum value.
        pub min: T,

        /// Maximum value.
        pub max: T,
    }

    #[derive(Clone, Debug, Copy, PartialEq)]
    pub struct Vec3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    impl Vec3 {
        pub const fn new(x: f32, y: f32, z: f32) -> Self {
            Self { x, y, z }
        }
    }

    fn point_min(p: &[Vec3]) -> Vec3 {
        p.iter().fold(
            Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            |mut min, current| {
                min.x = min.x.min(current.x);
                min.y = min.y.min(current.y);
                min.z = min.z.min(current.z);
                min
            },
        )
    }

    fn point_max(p: &[Vec3]) -> Vec3 {
        p.iter().fold(
            Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            |mut max, current| {
                max.x = max.x.max(current.x);
                max.y = max.y.max(current.y);
                max.z = max.z.max(current.z);
                max
            },
        )
    }

    impl BoundingBox {
        pub fn union(&self, other: &BoundingBox) -> BoundingBox {
            BoundingBox {
                min: point_min(&[self.min, other.min]),
                max: point_max(&[self.max, other.max]),
            }
        }
        pub fn center(&self) -> Vec3 {
            let centerx = (self.min.x + self.max.x) / 2.0;
            let centery = (self.min.y + self.max.y) / 2.0;
            let centerz = (self.min.z + self.max.z) / 2.0;
            Vec3::new(centerx, centery, centerz)
        }
    }
}
