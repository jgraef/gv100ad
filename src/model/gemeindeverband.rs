use chrono::NaiveDate;

use super::kreis::KreisSchluessel;

#[derive(Clone, Debug)]
pub struct GemeindeverbandDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Kreisschluessel
    pub kreis_schluessel: KreisSchluessel,

    /// Identifier of Gemeindeverband. This together with `kreis_schluessel`
    /// uniquely identifies a Gemeinderverband.
    pub gemeindeverband: u16,

    /// Name of Gemeindeverband
    pub name: String,

    /// Location of administration
    ///
    /// # Todo
    ///
    ///  - I think this can be empty, so we should make this an `Option`.
    pub sitz_verwaltung: String,
}
