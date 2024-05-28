use std::f64::consts::PI;

/// Point Primitives

/// 2D Point with Identifier
/// The identified is used to identify points between data structures
/// (the points list and the kd-tree)
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// x coordinate
    pub x: f64,
    /// y coordinate
    pub y: f64,
    /// identifier
    pub id: u64,
}

impl Point {
    /// constructor for python bindings
    pub fn new(x: f64, y: f64, id: u64) -> Self {
        Point { x, y, id }
    }

    pub fn angle(&self, b: &Point) -> f64 {
        let angle = -((b.y - self.y).atan2(b.x - self.x));
        normalise_angle(angle)
    }
}

/// Point Value -- Neighbor Information
/// Point value captures a point, with a distance and angle quantity with
/// respect to another point
pub struct PointValue {
    /// identified point
    pub point: Point,
    /// distance to other
    pub distance: f64,
    /// angle from other
    pub angle: f64,
}

pub fn normalise_angle(radians: f64) -> f64 {
    if radians < 0.0 {
        radians + PI + PI
    } else {
        radians
    }
}

pub fn intersects(a: (&Point, &Point), b: (&Point, &Point)) -> bool {
    let ax1 = a.0.x;
    let ay1 = a.0.y;
    let ax2 = a.1.x;
    let ay2 = a.1.y;
    let bx1 = b.0.x;
    let by1 = b.0.y;
    let bx2 = b.1.x;
    let by2 = b.1.y;

    let a1 = ay2 - ay1;
    let b1 = ax1 - ax2;
    let c1 = a1 * ax1 + b1 * ay1;
    let a2 = by2 - by1;
    let b2 = bx1 - bx2;
    let c2 = a2 * bx1 + b2 * by1;
    let det = a1 * b2 - a2 * b1;

    if det.abs() < 1E-10 {
        false
    } else {
        let x = (b2 * c1 - b1 * c2) / det;
        let y = (a1 * c2 - a2 * c1) / det;

        ax1.min(ax2) <= x
            && (x <= ax1.max(ax2))
            && (ay1.min(ay2) <= y)
            && (y <= ay1.max(ay2))
            && (bx1.min(bx2) <= x)
            && (x <= bx1.max(bx2))
            && (by1.min(by2) <= y)
            && (y <= by1.max(by2))
    }
}
