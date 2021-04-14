[![crates.io](https://img.shields.io/crates/v/gv100ad.svg)](https://crates.io/crates/gv100ad)
[![docs.rs](https://docs.rs/gv100ad/badge.svg)](https://docs.rs/gv100ad)
![MIT license](https://img.shields.io/crates/l/gv100ad)
![Maintenance](https://img.shields.io/badge/maintenance-experimental-blue.svg)

# gv100ad

**This software is experimental and might change a lot in future**

This is a Rust implementation of a parser for GV100AD data sets. These data
sets contain information about the structure, population, area of german
municipalities.

The data sets can be obtained at: https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/_inhalt.html

The parser was tested with this data set: https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/Administrativ/Archiv/GV100ADQ/GV100AD3004.html

The ZIP files contain a text file `GV100AD_DDMMYY.txt` that contains the
data set, and a PDF file describing the format.

## Example

This example lists all municipalities of the state *Saarland* with
population:

```rust
use gv100ad::{
    model::{
        gemeinde::GemeindeDaten,
        kreis::KreisDaten,
        land::{LandDaten, LandSchluessel},
    },
    Database,
};

// Open the database. Refer to the `README.md` file for the source of the datasets.
let db = Database::from_path("GV100AD3004/GV100AD_300421.txt").unwrap();

// Parse a key for the state of Saarland
let schluessel = "10".parse::<LandSchluessel>().unwrap();

// Get the record for the state of Saarland
let land = db.get::<_, LandDaten>(schluessel).unwrap();
println!("{}:", land.name);

// Iterate over the districts (Kreise) in Saarland
for kreis in db.children::<_, KreisDaten>(schluessel) {
    println!("  {}:", kreis.name);

    // Iterate over the municipalities (Gemeinde) in the district (Kreis)
    for gemeinde in db.children::<_, GemeindeDaten>(kreis.schluessel) {
        println!(
            "    {}: {} residents",
            gemeinde.name, gemeinde.population_total
        );
    }
}

// Get the sum of the population of all municipalities in Saarland
let total_population: u64 = db.children::<_, GemeindeDaten>(schluessel)
    .map(|gemeinde| gemeinde.population_total)
    .sum();
println!("Total population of {}: {}", land.name, total_population);
```

### Language

The primary language used for the software is English, thus most of
documentation and code is in English. Nevertheless a lot of terms are
inherently German, and a lot of identifiers in the software use these terms.
Here are a few translations:

 * Land: State (also called Bundesland)
 * Regierungsbezirk: Government district
 * Kreis: District
 * Gemeinde: Municipality (more literally "community")
 * Verband: Association
 * Schluessel: Key
 * Textkennzeichen: Textual (it's actually a number) identifier for type of
   Kreis, Gemeindeverband or Gemeinde.
 * Daten: data, in context e.g. "Landdaten" means "state data" or "state
   record".

 If you think a translation is incorrect or missing, please open an issue.

## License

Licensed under MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)
