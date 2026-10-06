//! A file of Urban QOL data once it is in memory: its meshes, and what it
//! publishes for each of them. Reading one is [`file`]'s.

use crate::mesh::HalfMesh;

mod file;

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

    /// The published total, then each indicator of the file.
    pub(crate) fn series(&self) -> &[Series] {
        &self.series
    }
}
