use nalgebra::Vector2;

use crate::{
    tilemap::{get_tilemap_index, TileId, Tilemap},
    EmeraldError, TextureKey,
};

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum AutoTileRulesetValue {
    None,
    Tile,
    Any,
}

const AUTOTILE_RULESET_GRID_SIZE: usize = 5;

pub struct AutoTileRuleset {
    pub tile_id: TileId,

    /// A grid determining the ruleset that displays this tile.
    /// Most grids will only need to cover a 3x3 area around the center tile,
    /// however we offer a 5x5 to cover larger rulesets. If you don't care to use all
    /// all of the 5x5 grid, place the outer rings with AutoTile::Any.
    ///
    /// These grids are rotated 90* clockwise visually.
    /// Ex 1.
    /// [
    ///     [Any, Any, Any, Any, Any]
    ///     [Any, None, None, None, Any]
    ///     [Any, None, Tile, None, Any]
    ///     [Any, None, None, None, Any]
    ///     [Any, Any, Any, Any, Any]
    /// ]
    /// The above grid displays the tile when it is completely alone, and not surrounded by any tiles.
    /// The value in the center of the ruleset grid is ignored, as this space is reserved for the AutoTile.
    ///
    /// Ex 2.
    /// [
    ///     [Any, Any, Any, Any, Any]
    ///     [Any, None, Any, None, Any]
    ///     [Any, None, Tile, None, Any]
    ///     [Any, None, Tile, None, Any]
    ///     [Any, Any, Any, Any, Any]
    /// ]
    /// This example will display the given tile, when there is a tile right of the center and
    /// does not care about the tile left of center.
    pub grid: [[AutoTileRulesetValue; AUTOTILE_RULESET_GRID_SIZE]; AUTOTILE_RULESET_GRID_SIZE],
}
impl AutoTileRuleset {
    /// Tests a 5x5 area centering on the given x, y values and determines if it's a match.
    pub(crate) fn matches(
        &self,
        autotiles: &Vec<AutoTile>,
        autotilemap_width: usize,
        autotilemap_height: usize,
        x: usize,
        y: usize,
    ) -> bool {
        match get_tilemap_index(x, y, autotilemap_width, autotilemap_height) {
            Err(_) => {
                return false;
            }
            Ok(index) => {
                if autotiles[index] != AutoTile::Tile {
                    return false;
                }
            }
        }

        for ruleset_x in 0..AUTOTILE_RULESET_GRID_SIZE {
            for ruleset_y in 0..AUTOTILE_RULESET_GRID_SIZE {
                // If center tile or any, skip
                if (ruleset_x == (AUTOTILE_RULESET_GRID_SIZE / 2)
                    && ruleset_y == (AUTOTILE_RULESET_GRID_SIZE / 2))
                    || self.grid[ruleset_x][ruleset_y] == AutoTileRulesetValue::Any
                {
                    continue;
                }

                let autotile_ruleset_value = self.get_autotile_ruleset_value(
                    autotiles,
                    autotilemap_width,
                    autotilemap_height,
                    x as isize - (AUTOTILE_RULESET_GRID_SIZE / 2) as isize + ruleset_x as isize,
                    y as isize - (AUTOTILE_RULESET_GRID_SIZE / 2) as isize + ruleset_y as isize,
                );

                if self.grid[ruleset_x][ruleset_y] != autotile_ruleset_value {
                    return false;
                }
            }
        }

        true
    }

    fn get_autotile_ruleset_value(
        &self,
        autotiles: &Vec<AutoTile>,
        autotilemap_width: usize,
        autotilemap_height: usize,
        x: isize,
        y: isize,
    ) -> AutoTileRulesetValue {
        // Treat tiles outside the boundaries of the map as Any.
        if x < 0 || y < 0 {
            return AutoTileRulesetValue::Any;
        }

        match get_tilemap_index(
            x as usize,
            y as usize,
            autotilemap_width,
            autotilemap_height,
        ) {
            Ok(index) => match autotiles[index] {
                AutoTile::None => AutoTileRulesetValue::None,
                AutoTile::Tile => AutoTileRulesetValue::Tile,
            },
            // Out of bounds, so we assume any
            Err(_) => AutoTileRulesetValue::Any,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum AutoTile {
    None = 0,
    Tile = 1,
}

pub struct AutoTilemap {
    pub(crate) tilemap: Tilemap,
    rulesets: Vec<AutoTileRuleset>,
    autotiles: Vec<AutoTile>,
}
impl AutoTilemap {
    pub fn new(
        tilesheet: TextureKey,
        // Size of a tile in the grid, in pixels
        tile_size: Vector2<usize>,
        map_width: usize,
        map_height: usize,
        rulesets: Vec<AutoTileRuleset>,
    ) -> Self {
        let tilemap = Tilemap::new(tilesheet, tile_size, map_width, map_height);

        let tile_count = map_width * map_height;
        let mut autotiles = Vec::with_capacity(tile_count);
        for _ in 0..tile_count {
            autotiles.push(AutoTile::None);
        }

        Self {
            tilemap,
            rulesets,
            autotiles,
        }
    }

    /// Bakes the inner tileset in accordance to the Autotilemap
    /// TODO: feature(physics): Additionally bakes the colliders
    pub fn bake(&mut self) -> Result<(), EmeraldError> {
        for x in 0..self.width() {
            for y in 0..self.height() {
                self.tilemap.set_tile(x, y, self.compute_tile_id(x, y)?)?;
            }
        }

        Ok(())
    }

    pub fn width(&self) -> usize {
        self.tilemap.width
    }

    pub fn height(&self) -> usize {
        self.tilemap.height
    }

    pub fn tilesheet(&self) -> TextureKey {
        self.tilemap.tilesheet.clone()
    }

    pub fn tile_size(&self) -> Vector2<usize> {
        self.tilemap.tile_size.clone()
    }

    pub fn add_ruleset(&mut self, ruleset: AutoTileRuleset) {
        self.rulesets.push(ruleset);
    }

    pub fn get_autotile(&mut self, x: usize, y: usize) -> Result<AutoTile, EmeraldError> {
        let index = get_tilemap_index(x, y, self.width(), self.height())?;
        Ok(self.autotiles[index])
    }

    pub fn set_tile(&mut self, x: usize, y: usize) -> Result<(), EmeraldError> {
        self.set_autotile(x, y, AutoTile::Tile)
    }

    pub fn set_none(&mut self, x: usize, y: usize) -> Result<(), EmeraldError> {
        self.set_autotile(x, y, AutoTile::None)
    }

    pub fn set_autotile(
        &mut self,
        x: usize,
        y: usize,
        new_tile_id: AutoTile,
    ) -> Result<(), EmeraldError> {
        let index = get_tilemap_index(x, y, self.width(), self.height())?;
        self.autotiles[index] = new_tile_id;

        Ok(())
    }

    pub fn tiles(&self) -> &Vec<Option<TileId>> {
        &self.tilemap.tiles
    }

    pub fn get_ruleset(&self, tile_id: TileId) -> Option<&AutoTileRuleset> {
        self.rulesets
            .iter()
            .find(|ruleset| ruleset.tile_id == tile_id)
    }

    pub fn remove_ruleset(&mut self, tile_id: TileId) -> Option<AutoTileRuleset> {
        if let Some(index) = self
            .rulesets
            .iter()
            .position(|ruleset| ruleset.tile_id == tile_id)
        {
            return Some(self.rulesets.remove(index));
        }

        None
    }

    /// Computes the tileid for the given autotile position.
    pub fn compute_tile_id(&self, x: usize, y: usize) -> Result<Option<TileId>, EmeraldError> {
        if let Some(ruleset) = self
            .rulesets
            .iter()
            .find(|ruleset| ruleset.matches(&self.autotiles, self.width(), self.height(), x, y))
        {
            return Ok(Some(ruleset.tile_id));
        }

        Ok(None)
    }
    pub fn get_tile_id(&self, x: usize, y: usize) -> Result<Option<TileId>, EmeraldError> {
        self.tilemap.get_tile(x, y)
    }
}
#[test]
fn test_autotiles() {}
