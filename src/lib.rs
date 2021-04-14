//! **This software is experimental and might change a lot in future**
//!
//! This is a Rust implementation of a parser for GV100AD data sets. These data
//! sets contain information about the structure, population, area of german
//! municipalities.
//!
//! The data sets can be obtained at: https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/_inhalt.html
//!
//! The parser was tested with this data set: https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/Administrativ/Archiv/GV100ADQ/GV100AD3004.html
//!
//! The ZIP files contain a text file `GV100AD_DDMMYY.txt` that contains the
//! data set, and a PDF file describing the format.
//!
//! # Example
//!
//! This example lists all municipalities of the state *Saarland* with
//! population:
//!
//! ```rust
//! use gv100ad::{
//!     model::{
//!         land::{LandDaten, LandSchluessel},
//!         kreis::KreisDaten,
//!         gemeinde::GemeindeDaten,
//!     },
//!     Database,
//! };
//!
//! let db = Database::from_path("GV100AD3004/GV100AD_300421.txt").unwrap();
//!
//! let schluessel = "10".parse::<LandSchluessel>().unwrap();
//! let land = db.get::<_, LandDaten>(schluessel).unwrap();
//!
//! println!("{}:", land.name);
//!
//! for kreis in db.children::<_, KreisDaten>(schluessel) {
//!     println!("  {}:", kreis.name);
//!
//!     for gemeinde in db.children::<_, GemeindeDaten>(kreis.schluessel) {
//!         println!(
//!             "    {}: {} residents",
//!             gemeinde.name, gemeinde.population_total
//!         );
//!     }
//! }
//! ```
//!
//! ## Language
//!
//! The primary language used for the software is English, thus most of
//! documentation and code is in English. Nevertheless a lot of terms are
//! inherently German, and a lot of identifiers in the software use these terms.
//! Here are a few translations:
//!
//!  * Land: State (also called Bundesland)
//!  * Regierungsbezirk: Government district
//!  * Kreis: District
//!  * Gemeinde: Municipality (more literally "community")
//!  * Verband: Association
//!  * Schluessel: Key
//!  * Textkennzeichen: Textual (it's actually a number) identifier for type of
//!    Kreis, Gemeindeverband or Gemeinde.
//!  * Daten: data, in context e.g. "Landdaten" means "state data" or "state
//!    record".
//!
//!  If you think a translation is incorrect or missing, please open an issue.
//!

pub mod db;
pub mod error;
pub mod model;
pub mod parser;

pub use db::Database;
pub use parser::Parser;
