//! A file of Urban QOL data once it is in memory: its meshes, and what it
//! publishes for each of them. Reading one is [`file`]'s.

use crate::mesh::{HalfMesh, Point};

mod file;

/// The statement of the data's source that its provider asks for, as the
/// screen shows it.
pub(crate) const SOURCE: &str = include_str!("../assets/data-source.txt").trim_ascii_end();

/// A file's meshes, and what it publishes for each of them.
#[derive(Debug, PartialEq)]
pub(crate) struct Dataset {
    meshes: Vec<Mesh>,
    series: Vec<Series>,
}

/// A mesh of the file.
#[derive(Debug, PartialEq)]
pub(crate) struct Mesh {
    pub(crate) code: String,
    pub(crate) square: HalfMesh,
    /// The name of the municipality the file puts the mesh in.
    pub(crate) city: String,
}

/// What the file publishes under one `IndicatorCode`: the total, or an
/// indicator.
#[derive(Debug, PartialEq)]
pub(crate) struct Series {
    pub(crate) code: String,
    pub(crate) name: String,
    /// One value for each mesh, in the order of [`Dataset::meshes`].
    pub(crate) values: Vec<f64>,
}

impl Dataset {
    /// The file's meshes, in the order it first names them.
    pub(crate) fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    /// The middle of the area the meshes cover. A file that reads has a mesh.
    pub(crate) fn centre(&self) -> Point {
        let (south, north, west, east) = self.meshes.iter().map(|mesh| mesh.square).fold(
            (
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ),
            |(south, north, west, east), square| {
                (
                    south.min(square.south()),
                    north.max(square.north()),
                    west.min(square.west()),
                    east.max(square.east()),
                )
            },
        );
        Point {
            latitude: (south + north) / 2.0,
            longitude: (west + east) / 2.0,
        }
    }

    /// The published total, then each indicator of the file.
    pub(crate) fn series(&self) -> &[Series] {
        &self.series
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dataset(codes: &[&str]) -> Dataset {
        let meshes = codes
            .iter()
            .map(|code| Mesh {
                code: (*code).to_owned(),
                square: HalfMesh::from_code(code).unwrap(),
                city: String::new(),
            })
            .collect();
        Dataset {
            meshes,
            series: Vec::new(),
        }
    }

    #[test]
    fn the_centre_of_one_mesh_is_its_middle() {
        let square = HalfMesh::from_code("543823431").unwrap();
        let centre = dataset(&["543823431"]).centre();
        assert_eq!(centre.latitude, (square.south() + square.north()) / 2.0);
        assert_eq!(centre.longitude, (square.west() + square.east()) / 2.0);
    }

    /// The first mesh is the one further south and east, so each edge of the
    /// area comes from the mesh that reaches furthest that way.
    #[test]
    fn the_centre_is_midway_between_the_outermost_edges() {
        let south_east = HalfMesh::from_code("533900001").unwrap();
        let north_west = HalfMesh::from_code("543823434").unwrap();
        let centre = dataset(&["533900001", "543823434"]).centre();
        assert_eq!(
            centre.latitude,
            (south_east.south() + north_west.north()) / 2.0
        );
        assert_eq!(
            centre.longitude,
            (north_west.west() + south_east.east()) / 2.0
        );
    }
}
