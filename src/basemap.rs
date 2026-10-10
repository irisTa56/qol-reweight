//! The base map: the pale map of the Geospatial Information Authority of
//! Japan (GSI), as raster tiles fetched while the map is looked at.
//!
//! Its terms are on GSI's list of tiles, at [`TILE_LIST`]: an application may
//! load the tiles on stating their source with a link to that list.

use walkers::TileId;
use walkers::sources::{Attribution, TileSource};

use crate::asset;

/// What the screen calls the tiles' source.
pub(crate) const SOURCE: &str = asset::text!("base-map-source.txt");

/// GSI's list of its tiles, which the statement of the source links to.
pub(crate) const TILE_LIST: &str = "https://maps.gsi.go.jp/development/ichiran.html";

/// The credit the pale map's entry on that list asks for beside the source,
/// for its tiles at zoom level 8 and below.
pub(crate) const SHORELINE_CREDIT: &str = "Shoreline data is derived from: United States. \
     National Imagery and Mapping Agency. \"Vector Map Level 0 (VMAP0).\" \
     Bethesda, MD: Denver, CO: The Agency; USGS Information Services, 1997.";

/// The deepest zoom level the pale map has tiles for.
const DEEPEST_ZOOM: u8 = 18;

/// GSI's pale map.
pub(crate) struct PaleMap;

impl TileSource for PaleMap {
    fn tile_url(&self, tile: TileId) -> String {
        format!(
            "https://cyberjapandata.gsi.go.jp/xyz/pale/{}/{}/{}.png",
            tile.zoom, tile.x, tile.y
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: SOURCE,
            url: TILE_LIST,
            logo_light: None,
            logo_dark: None,
        }
    }

    fn max_zoom(&self) -> u8 {
        DEEPEST_ZOOM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The address GSI's list gives for the pale map, with the zoom level
    /// before the column and the row.
    #[test]
    fn a_tile_is_asked_of_gsi_by_its_zoom_level_column_and_row() {
        let tile = TileId {
            x: 3,
            y: 5,
            zoom: 7,
        };
        assert_eq!(
            PaleMap.tile_url(tile),
            "https://cyberjapandata.gsi.go.jp/xyz/pale/7/3/5.png"
        );
    }

    #[test]
    fn the_map_is_given_the_source_and_the_list_it_links_to() {
        let attribution = PaleMap.attribution();
        assert_eq!(attribution.text, SOURCE);
        assert_eq!(attribution.url, TILE_LIST);
    }

    /// GSI's list gives the pale map tiles down to zoom level 18.
    #[test]
    fn no_tile_is_asked_for_past_the_deepest_level_the_map_has() {
        assert_eq!(PaleMap.max_zoom(), 18);
    }
}
