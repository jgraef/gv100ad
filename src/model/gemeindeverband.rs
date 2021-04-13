use std::convert::TryFrom;

use chrono::NaiveDate;

use super::kreis::KreisSchluessel;

use crate::error::Error;

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
    pub sitz_verwaltung: Option<String>,

    /// Specifies type of Gemeindeverband
    pub textkennzeichen: GemeindeverbandTextkennzeichen,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GemeindeverbandTextkennzeichen {
    VerbandsfreieGemeinde,
    Amt,
    Samtgemeinde,
    Verbandsgemeinde,
    Verwaltungsgemeinschaft,
    Kirchspielslandgemeinde,
    Verwaltungsverband,
    VGTraegermodell,
    ErfuellendeGemeinde,
}

impl TryFrom<u8> for GemeindeverbandTextkennzeichen {
    type Error = Error;

    fn try_from(n: u8) -> Result<Self, Error> {
        match n {
            50 => Ok(Self::VerbandsfreieGemeinde),
            51 => Ok(Self::Amt),
            52 => Ok(Self::Samtgemeinde),
            53 => Ok(Self::Verbandsgemeinde),
            54 => Ok(Self::Verwaltungsgemeinschaft),
            55 => Ok(Self::Kirchspielslandgemeinde),
            56 => Ok(Self::Verwaltungsverband),
            57 => Ok(Self::VGTraegermodell),
            58 => Ok(Self::ErfuellendeGemeinde),
            _ => Err(Error::InvalidTextkennzeichen(n)),
        }
    }
}
