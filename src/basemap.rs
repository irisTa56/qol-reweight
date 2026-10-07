//! The base map: the pale map of the Geospatial Information Authority of
//! Japan (GSI), as raster tiles fetched while the map is looked at.
//!
//! Its terms are on GSI's list of tiles, at [`TILE_LIST`]: an application may
//! load the tiles on stating their source with a link to that list.

use walkers::TileId;
use walkers::sources::{Attribution, TileSource};

/// What the screen calls the tiles' source.
pub(crate) const SOURCE: &str = include_str!("../assets/base-map-source.txt").trim_ascii_end();

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
