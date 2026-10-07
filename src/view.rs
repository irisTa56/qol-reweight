//! What of the map is in view when the window opens.

use std::f64::consts::PI;

use walkers::{Position, lat_lon};

use crate::mesh::Extent;

/// The zoom level the map opens at, which shows a city and what lies around
/// it. Its user zooms in from there to the scale a neighbourhood shows at,
/// which no one level gives for every file.
const OPENING_ZOOM: f64 = 10.0;

/// A place the map is centred on, and a zoom level.
#[derive(Clone, Copy, Debug)]
pub(crate) struct View {
    centre: Position,
    zoom: f64,
}

impl View {
    /// The view the map opens with: centred on `extent`.
    pub(crate) fn of(extent: Extent) -> Self {
        // Midway between the edges as the map draws them, which for the
        // northern and southern ones is not midway between their latitudes:
        // the base map is in the Web Mercator projection.
        let latitude = latitude_at((northing(extent.north()) + northing(extent.south())) / 2.0);
        let longitude = (extent.west() + extent.east()) / 2.0;
        Self {
            centre: lat_lon(latitude, longitude),
            zoom: OPENING_ZOOM,
        }
    }

    pub(crate) fn centre(self) -> Position {
        self.centre
    }

    pub(crate) fn zoom(self) -> f64 {
        self.zoom
    }
}

/// How far north of the equator a latitude, in degrees, lies on a Web
/// Mercator map whose equator is 2 pi long.
fn northing(latitude: f64) -> f64 {
    (PI / 4.0 + latitude.to_radians() / 2.0).tan().ln()
}

/// The latitude, in degrees, that lies `northing` north of the equator on
/// that map.
fn latitude_at(northing: f64) -> f64 {
    (2.0 * northing.exp().atan() - PI / 2.0).to_degrees()
}

#[cfg(test)]
mod tests {
    use eframe::egui::{Rect, Vec2, pos2, vec2};
    use walkers::{MapMemory, Projector};

    use super::*;
    use crate::mesh::HalfMesh;

    const SIZE: Vec2 = vec2(800.0, 600.0);

    fn extent(codes: &[&str]) -> Extent {
        Extent::of(codes.iter().map(|code| HalfMesh::from_code(code).unwrap())).unwrap()
    }

    /// Where the map widget puts the corners of `extent` in its opening
    /// view, on a map of [`SIZE`], is around the middle of that map.
    fn assert_centred(extent: Extent) {
        let view = View::of(extent);
        let map = Rect::from_min_size(pos2(0.0, 0.0), SIZE);
        let mut memory = MapMemory::default();
        memory.set_zoom(view.zoom()).unwrap();
        let projector = Projector::new(map, &memory, view.centre());
        let south_west = projector.project(lat_lon(extent.south(), extent.west()));
        let north_east = projector.project(lat_lon(extent.north(), extent.east()));
        let area = Rect::from_two_pos(south_west.to_pos2(), north_east.to_pos2());
        assert!((area.center() - map.center()).length() < 0.01, "{area:?}");
    }

    /// Two squares a degree of longitude apart on one parallel.
    #[test]
    fn a_wide_area_is_in_the_middle_of_the_map() {
        assert_centred(extent(&["533900001", "534000001"]));
    }

    /// Two squares 2 degrees of latitude apart on one meridian, far enough
    /// that midway between their latitudes is pixels off the middle.
    #[test]
    fn a_tall_area_is_in_the_middle_of_the_map() {
        assert_centred(extent(&["523900001", "553900001"]));
    }

    #[test]
    fn one_square_is_in_the_middle_of_the_map() {
        assert_centred(extent(&["543823431"]));
    }
}
