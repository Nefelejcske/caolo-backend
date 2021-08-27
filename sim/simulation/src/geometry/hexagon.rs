use super::Axial;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Hexagon {
    pub center: Axial,
    pub radius: i32,
}

impl Hexagon {
    pub fn new(center: Axial, radius: i32) -> Self {
        Self { center, radius }
    }

    pub fn from_radius(radius: i32) -> Self {
        debug_assert!(radius >= 0);
        Self {
            radius,
            center: Axial::new(radius, radius),
        }
    }

    pub fn contains(self, point: Axial) -> bool {
        let point = point - self.center;
        let [x, y, z] = point.hex_axial_to_cube();
        let r = self.radius.abs();
        x.abs() <= r && y.abs() <= r && z.abs() <= r
    }

    pub fn iter_edge(self) -> impl Iterator<Item = Axial> {
        debug_assert!(
            self.radius > 0,
            "not-positive radius will not work as expected"
        );

        const STARTS: [Axial; 6] = [
            Axial::new(0, -1),
            Axial::new(1, -1),
            Axial::new(1, 0),
            Axial::new(0, 1),
            Axial::new(-1, 1),
            Axial::new(-1, 0),
        ];
        const DELTAS: [Axial; 6] = [
            Axial::new(1, 0),
            Axial::new(0, 1),
            Axial::new(-1, 1),
            Axial::new(-1, 0),
            Axial::new(0, -1),
            Axial::new(1, -1),
        ];
        let radius = self.radius;
        let center = self.center;
        (0..6).flat_map(move |di| {
            // iterating over `deltas` is a compile error because they're freed at the end of this
            // funciton...
            let delta = DELTAS[di];
            let pos = center + STARTS[di] * radius;
            (0..radius).map(move |j| pos + delta * j)
        })
    }

    pub fn area(self) -> usize {
        debug_assert!(self.radius >= 0);
        (1 + 3 * self.radius * (self.radius + 1)) as usize
    }

    /// points will spiral out from the center
    pub fn iter_points(self) -> impl Iterator<Item = Axial> {
        let center: Axial = self.center;
        // radius =0 doesn't yield any points
        Some(center)
            .into_iter()
            .chain((1..=self.radius).flat_map(move |r| Hexagon::new(center, r).iter_edge()))
    }

    pub fn with_center(mut self, center: Axial) -> Self {
        self.center = center;
        self
    }

    pub fn with_offset(mut self, offset: Axial) -> Self {
        self.center += offset;
        self
    }

    pub fn with_radius(mut self, radius: i32) -> Self {
        self.radius = radius;
        self
    }
}

/// Rounds the given Axial coorinates to the nearest hex
pub fn hex_round(point: [f32; 2]) -> Axial {
    let x = point[0];
    let z = point[1];
    let y = -x - z;

    let cube = cube_round([x, y, z]);
    Axial::hex_cube_to_axial(cube)
}

/// Rounds the Cube representation of a hexagon to the nearest hex
pub fn cube_round(point: [f32; 3]) -> [i32; 3] {
    let mut rx = point[0].round();
    let mut ry = point[1].round();
    let mut rz = point[2].round();

    let dx = (rx - point[0]).abs();
    let dy = (ry - point[1]).abs();
    let dz = (rz - point[2]).abs();

    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy > dz {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }

    [rx as i32, ry as i32, rz as i32]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_iter_points_are_inside_itself() {
        let hex = Hexagon::from_radius(3).with_center(Axial::default());

        dbg!(hex
            .with_center(Axial::default())
            .with_radius(2)
            .iter_points()
            .collect::<Vec<_>>());

        for (i, p) in hex.iter_points().enumerate() {
            dbg!(p);
            assert!(hex.contains(p), "{} {:?} {:?}", i, p, hex);
        }
    }

    #[test]
    fn test_iter_edge() {
        let pos = Axial::new(0, 0);
        let radius = 4;
        let hex = Hexagon::new(pos, radius);

        let edge: Vec<_> = hex.iter_edge().collect();

        dbg!(hex, &edge);

        assert_eq!(edge.len(), 6 * radius as usize);

        for (i, p) in edge.iter().copied().enumerate() {
            assert_eq!(
                p.hex_distance(pos),
                radius as u32,
                "Hex #{} {:?} is out of range",
                i,
                p
            );
        }
    }
}
