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
//! TODO
//! 
//! ## Language
//! 
//! The primary language used for the software is English, thus most of documentation and code is in English. Nevertheless a lot of terms are inherently German, and a lot of identifiers in the software use these terms. Here are a few translations:
//! 
//!  * Land: State (also called Bundesland)
//!  * Regierungsbezirk: Government district
//!  * Kreis: District
//!  * Gemeinde: Municipality (more literally "community")
//!  * Verband: Association
//!  * Schluessel: Key
//!  * Textkennzeichen: Textual (it's actually a number) identifier for type of Kreis, Gemeindeverband or Gemeinde.
//!  * Daten: data, in context e.g. "Landdaten" means "state data" or "state record".
//! 
//!  If you think a translation is incorrect or missing, please open an issue.
//! 
//! ## Key structure
//! 
//! The primary type of key used is a "Regionalschluessel", which is a hierarchical key containing:
//! 
//!  1. Land: 2 digits, or `u8`
//!  2. Regierungsbezirk: 1 digit, or `u8`
//!  3. Kreis: 2 digits, or `u8`
//!  4. Gemeinde: 3 digits, or `u8`.
//! 
//! E.g. a Landschluessel (e.g. `10` for Saarland) only identifies the state. A Kreisschluessel contains keys to identify the Land and Regierungsbezirk the Kreis is in, and the key for the Kreis itself. E.g. `10RKK` identifies the Gemeinde ? in Regierungsbezirk ? in the state of Saarland.
//! 
//! Regionen and Gemeindeverbaende are identified somewhat idenpendently from the Regionalschluessel.
//! 
//! Regionen have a 1 digit identifier and only need the Land to be furthe identified. Thus a 3-digit key `LLR` would uniquely identify the Region. Furthermore since Regionen are only valid in the state of Baden-Wuerttemberg, the land can be ommitted too.
//! 
//! TODO: Gemeindeverband-Schluessel?
//! 
//!
//! # TODO
//!
//!  - Implement Textkennzeichen correctly.

pub mod db;
pub mod error;
pub mod model;
pub mod parser;
