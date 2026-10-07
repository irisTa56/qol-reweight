//! What of the map is in view when the window opens.

use std::f64::consts::PI;

use eframe::egui::Vec2;
use walkers::{Position, lat_lon};

use crate::basemap::DEEPEST_ZOOM;
use crate::mesh::Extent;

/// How many pixels wide the whole world is at zoom level 0: one tile.
const WORLD_AT_ZOOM_0: f64 = 256.0;

/// The share of the map's width and height an area may take up, which leaves
/// a margin of the base map around it.
const FILL: f64 = 0.85;

/// A place the map is centred on, and a zoom level.
#[derive(Clone, Copy, Debug)]
pub(crate) struct View {
    centre: Position,
    zoom: f64,
}

impl View {
    /// The view centred on `extent` at the deepest zoom that shows all of it
    /// on a map of `size`, in points, and that the base map has tiles for.
    pub(crate) fn fitting(extent: Extent, size: Vec2) -> Self {
        // The base map is in the Web Mercator projection: at zoom level 0 the
        // world is one tile wide and high, and each level doubles both.
        let wide = (extent.east() - extent.west()) / 360.0;
        let high = (northing(extent.north()) - northing(extent.south())) / (2.0 * PI);
        let across = f64::from(size.x) * FILL / (wide * WORLD_AT_ZOOM_0);
        let down = f64::from(size.y) * FILL / (high * WORLD_AT_ZOOM_0);
        // Midway between the edges as the map draws them, which for the
        // northern and southern ones is not midway between their latitudes.
        let latitude = latitude_at((northing(extent.north()) + northing(extent.south())) / 2.0);
        let longitude = (extent.west() + extent.east()) / 2.0;
        Self {
            centre: lat_lon(latitude, longitude),
            zoom: across.min(down).log2().clamp(0.0, f64::from(DEEPEST_ZOOM)),
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
    use eframe::egui::{Rect, pos2, vec2};
    use walkers::{MapMemory, Projector};

    use super::*;
    use crate::mesh::HalfMesh;

    const SIZE: Vec2 = vec2(800.0, 600.0);

    fn extent(codes: &[&str]) -> Extent {
        Extent::of(codes.iter().map(|code| HalfMesh::from_code(code).unwrap())).unwrap()
    }

    /// Where the map widget puts the corners of `extent` in `view`, on a map
    /// of [`SIZE`].
    fn on_screen(extent: Extent, view: View) -> Rect {
        let map = Rect::from_min_size(pos2(0.0, 0.0), SIZE);
        let mut memory = MapMemory::default();
        memory.set_zoom(view.zoom()).unwrap();
        let projector = Projector::new(map, &memory, view.centre());
        let south_west = projector.project(lat_lon(extent.south(), extent.west()));
        let north_east = projector.project(lat_lon(extent.north(), extent.east()));
        Rect::from_two_pos(south_west.to_pos2(), north_east.to_pos2())
    }

    /// The area is in the middle of the map and takes up the share of it
    /// that [`FILL`] allows one way, and no more than that the other way.
    fn assert_fits(extent: Extent) -> Rect {
        let area = on_screen(extent, View::fitting(extent, SIZE));
        let allowed = SIZE * FILL as f32;
        assert!(
            (area.center() - (SIZE / 2.0).to_pos2()).length() < 0.5,
            "{area:?}"
        );
        assert!(area.width() <= allowed.x + 0.5, "{area:?}");
        assert!(area.height() <= allowed.y + 0.5, "{area:?}");
        let full_width = (area.width() - allowed.x).abs() < 0.5;
        let full_height = (area.height() - allowed.y).abs() < 0.5;
        assert!(full_width || full_height, "{area:?}");
        area
    }

    /// Two squares a degree of longitude apart on one parallel.
    #[test]
    fn a_wide_area_fills_the_width() {
        let area = assert_fits(extent(&["533900001", "534000001"]));
        assert!(area.height() < 100.0, "{area:?}");
    }

    /// Two squares 40' of latitude apart on one meridian.
    #[test]
    fn a_tall_area_fills_the_height() {
        let area = assert_fits(extent(&["533900001", "543900001"]));
        assert!(area.width() < 100.0, "{area:?}");
    }

    /// The four quarters of one 1 km mesh, on which a fraction of a mesh is
    /// many pixels.
    #[test]
    fn a_small_area_is_fitted_as_closely() {
        let quarters = ["543823431", "543823432", "543823433", "543823434"];
        assert_fits(extent(&quarters));
    }

    /// One square fills a map this large only past the deepest level the
    /// base map has tiles for.
    #[test]
    fn a_small_area_stops_at_the_deepest_zoom_of_the_base_map() {
        let view = View::fitting(extent(&["543823431"]), SIZE * 10.0);
        assert_eq!(view.zoom(), f64::from(DEEPEST_ZOOM));
    }
}
