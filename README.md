# GV100AD parser

**This software is experimental and might change a lot in future**

This is a Rust implementation of a parser for GV100AD data sets. These data sets contain information about the structure, population, area of german municipalities.

The data sets can be obtained at: https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/_inhalt.html

The parser was tested with this data set: https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/Administrativ/Archiv/GV100ADQ/GV100AD3004.html

The ZIP files contain a text file `GV100AD_DDMMYY.txt` that contains the data set, and a PDF file describing the format.

## Example

This example lists all municipalities of the state *Saarland* with population:

```rust
use gv100ad::{Ags, Database};

fn main() {
    let db = Database::from_path("GV100AD_300421.txt").unwrap();

    let ags_land = Ags::new_land(10);

    let land = db.get_land(&ags_land).unwrap();
    println!("{}:", land.name);

    for kreis in db.iter_kreise_in(&ags_land) {
        println!("  {}:", kreis.name);

        for gemeinde in db.iter_gemeinden_in(&kreis.ags) {
            println!("    {}: {} residents", gemeinde.name, gemeinde.population_total);
        }
    }
}
```
