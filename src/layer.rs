//! The meshes as the map draws them: a filled square each, over the base map.

use eframe::egui::{Color32, Rect, Shape};
use eframe::epaint::Mesh as Triangles;
use walkers::{Projector, lat_lon};

use crate::dataset::{Mesh, Series};
use crate::mesh::HalfMesh;
use crate::paint::Paint;

/// Each mesh's square, and the colour it is filled with.
pub(crate) struct MeshLayer {
    squares: Vec<(HalfMesh, Color32)>,
    /// What the colours were taken from.
    paint: Paint,
}

impl MeshLayer {
    /// The meshes coloured by `series`, which has a value for each of them in
    /// their order, on the scale that fits those values, each colour
    /// covering `opacity` of 255 of the base map.
    pub(crate) fn showing(meshes: &[Mesh], series: &Series, opacity: u8) -> Self {
        let paint = Paint::fitting(series.values(), opacity);
        let squares = meshes
            .iter()
            .zip(series.values())
            .map(|(mesh, &value)| (mesh.square(), paint.of(value)))
            .collect();
        Self { squares, paint }
    }

    /// How the squares are painted, which a legend shows.
    pub(crate) fn paint(&self) -> Paint {
        self.paint
    }

    /// The squares as `projector` places them on screen, four corners each in
    /// the order of the meshes.
    pub(crate) fn shape(&self, projector: &Projector) -> Shape {
        let mut triangles = Triangles::default();
        for &(square, colour) in &self.squares {
            let south_west = projector.project(lat_lon(square.south(), square.west()));
            let north_east = projector.project(lat_lon(square.north(), square.east()));
            let corners = Rect::from_two_pos(south_west.to_pos2(), north_east.to_pos2());
            triangles.add_colored_rect(corners, colour);
        }
        Shape::mesh(triangles)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use eframe::egui::{Pos2, pos2, vec2};
    use walkers::MapMemory;

    use super::*;
    use crate::paint::OPENING_OPACITY;
    use crate::test_support::dataset_of_totals;

    /// The map's place in the window.
    const MAP: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));

    /// A map at `zoom`, centred on the middle of the square `code` names.
    fn centred_on(code: &str, zoom: f64) -> Projector {
        let square = HalfMesh::from_code(code).unwrap();
        let latitude = (square.south() + square.north()) / 2.0;
        let longitude = (square.west() + square.east()) / 2.0;
        let mut memory = MapMemory::default();
        memory.set_zoom(zoom).unwrap();
        Projector::new(MAP, &memory, lat_lon(latitude, longitude))
    }

    /// Each square the layer draws: its rectangle on screen and its colour.
    fn drawn(totals: &[(&str, f64)], projector: &Projector) -> Vec<(Rect, Color32)> {
        let dataset = dataset_of_totals(totals);
        let layer = MeshLayer::showing(dataset.meshes(), &dataset.series()[0], OPENING_OPACITY);
        let Shape::Mesh(triangles) = layer.shape(projector) else {
            panic!("the layer is not drawn as triangles");
        };
        triangles
            .vertices
            .chunks(4)
            .map(|corners| {
                let places: Vec<Pos2> = corners.iter().map(|corner| corner.pos).collect();
                (Rect::from_points(&places), corners[0].color)
            })
            .collect()
    }

    /// The square is as wide as its 22.5" of longitude is on a Web Mercator
    /// map, 256 pixels around the world at zoom 0 and twice that each level,
    /// and as high as its 15" of latitude, which that map stretches by one
    /// over the cosine of the latitude.
    #[test]
    fn a_square_is_drawn_at_its_place_and_size() {
        let zoom = 12.0;
        let drawn = drawn(&[("543823431", 1.0)], &centred_on("543823431", zoom));
        let [(square, _)] = drawn.as_slice() else {
            panic!("one mesh is not one square: {drawn:?}");
        };
        let world = 256.0 * 2f64.powf(zoom);
        let latitude = HalfMesh::from_code("543823431").unwrap().south();
        let wide = world * (22.5 / 3600.0) / 360.0;
        let high =
            world * (15.0f64 / 3600.0).to_radians() / (2.0 * PI) / latitude.to_radians().cos();
        assert!(
            (square.center() - MAP.center()).length() < 0.01,
            "{square:?}"
        );
        assert!(
            (f64::from(square.width()) - wide).abs() < 0.01,
            "{square:?}"
        );
        assert!(
            (f64::from(square.height()) - high).abs() < 0.01,
            "{square:?}"
        );
    }

    /// The four quarters of one 1 km mesh: 1 is the south-west one, 2 the
    /// south-east, 3 the north-west, and 4 the north-east. North is up.
    #[test]
    fn squares_that_share_an_edge_are_drawn_with_no_gap_and_no_overlap() {
        let quarters = [
            ("543823431", 1.0),
            ("543823432", 1.0),
            ("543823433", 1.0),
            ("543823434", 1.0),
        ];
        let drawn = drawn(&quarters, &centred_on("543823431", 14.0));
        let [south_west, south_east, north_west, north_east] =
            [0, 1, 2, 3].map(|quarter: usize| drawn[quarter].0);
        assert_eq!(south_west.right(), south_east.left());
        assert_eq!(north_west.right(), north_east.left());
        assert_eq!(south_west.top(), north_west.bottom());
        assert_eq!(south_east.top(), north_east.bottom());
        assert!(south_west.left() < south_west.right());
        assert!(north_west.top() < north_west.bottom());
    }

    #[test]
    fn a_square_has_the_colour_its_value_is_painted_in() {
        let totals = [("543823431", -2.0), ("543823432", 0.0), ("543823433", 1.0)];
        let drawn = drawn(&totals, &centred_on("543823431", 14.0));
        let paint = Paint::fitting(&[-2.0, 0.0, 1.0], OPENING_OPACITY);
        for ((_, colour), (_, value)) in drawn.iter().zip(totals) {
            assert_eq!(*colour, paint.of(value), "for {value}");
        }
        assert_ne!(drawn[0].1, drawn[1].1);
        assert_ne!(drawn[1].1, drawn[2].1);
    }

    /// Panning the map moves every square with it.
    #[test]
    fn the_squares_follow_the_map() {
        let totals = [("543823431", 1.0)];
        let here = drawn(&totals, &centred_on("543823431", 12.0))[0].0;
        let there = drawn(&totals, &centred_on("543823432", 12.0))[0].0;
        let moved = here.center() - there.center();
        assert!(
            (moved - vec2(here.width(), 0.0)).length() < 0.01,
            "{moved:?}"
        );
    }
}
