//! A file of Urban QOL data once it is in memory: its meshes, and what it
//! publishes for each of them. Reading one is [`file`]'s.

use crate::mesh::{Extent, HalfMesh};

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
    code: String,
    square: HalfMesh,
    /// The name of the municipality the file puts the mesh in.
    city: String,
}

/// What the file publishes under one `IndicatorCode`: the total, or an
/// indicator.
#[derive(Debug, PartialEq)]
pub(crate) struct Series {
    code: String,
    name: String,
    /// One value for each mesh, in the order of [`Dataset::meshes`].
    values: Vec<f64>,
}

impl Mesh {
    /// The square the mesh's code names.
    pub(crate) fn square(&self) -> HalfMesh {
        self.square
    }
}

impl Series {
    /// What the file calls the total or the indicator.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// One value for each mesh, in the order of [`Dataset::meshes`].
    pub(crate) fn values(&self) -> &[f64] {
        &self.values
    }
}

impl Dataset {
    /// The file's meshes, in the order it first names them.
    pub(crate) fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    /// The area the meshes cover.
    pub(crate) fn extent(&self) -> Extent {
        Extent::of(self.meshes.iter().map(|mesh| mesh.square))
            .expect("a file that reads has a mesh")
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
    fn the_extent_is_the_area_all_the_meshes_cover() {
        let codes = ["533900001", "543823434"];
        let squares = codes.map(|code| HalfMesh::from_code(code).unwrap());
        assert_eq!(dataset(&codes).extent(), Extent::of(squares).unwrap());
    }
}
