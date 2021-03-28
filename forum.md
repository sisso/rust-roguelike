# Map

The "real" map is a composition of many different maps. For instance, 
if a ship is landed in a terrain, this terrain has other ship landed
and some construction. 

Ships can not just be copied into the game map as we need to fly it again with
everything we have inside, leaving the previous tiles behind.

- If we copy the whole ship map grid, what about empty grid spaces? We will stil 
  need to deal with cell by cell.
  
- The ship grid can be "stale" until the ship take flight and we copy the tiles 
  back
  
- What about objects? 

- Looks like each cell must have a reference of what Grid it belongs.

- Each real cell, can have multiple layers of cells. (grid, tile), a object
  should stay in the top tile, or, belong ot tail (Z level?)

- Should be easy to have a composite map, where can resolve the gird for

## Landing

Giving a Ship Grid (SG) and Land Grid (LG), we will land SG into a position LXY 
relative to SG. 

For each non empty SG cell, we verify if LG cell can be landed. If all cells
are good. Land.


