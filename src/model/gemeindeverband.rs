use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use chrono::NaiveDate;

use super::{
    kreis::KreisSchluessel,
    land::LandSchluessel,
    regierungsbezirk::RegierungsbezirkSchluessel,
};

use crate::error::{Error, ParseKeyError};

#[derive(Clone, Debug)]
pub struct GemeindeverbandDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Gemeindeverbandschluessel
    pub schluessel: GemeindeverbandSchluessel,

    /// Name of Gemeindeverband
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: Option<String>,

    /// Specifies type of Gemeindeverband
    pub textkennzeichen: GemeindeverbandTextkennzeichen,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GemeindeverbandSchluessel {
    pub kreis: KreisSchluessel,
    pub gemeindeverband: u16,
}

impl GemeindeverbandSchluessel {
    pub fn new(kreis: KreisSchluessel, gemeindeverband: u16) -> Self {
        Self {
            kreis,
            gemeindeverband,
        }
    }
}

impl FromStr for GemeindeverbandSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 9 {
            return Err(ParseKeyError::invalid_length(s, 9));
        }

        let kreis = s[0..5].parse()?;
        let gemeindeverband = s[5..].parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(kreis, gemeindeverband))
    }
}

impl Display for GemeindeverbandSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{:4}", self.kreis, self.gemeindeverband)
    }
}

impl From<GemeindeverbandSchluessel> for KreisSchluessel {
    fn from(gemeinde: GemeindeverbandSchluessel) -> Self {
        gemeinde.kreis
    }
}

impl From<GemeindeverbandSchluessel> for RegierungsbezirkSchluessel {
    fn from(gemeinde: GemeindeverbandSchluessel) -> Self {
        gemeinde.kreis.into()
    }
}

impl From<GemeindeverbandSchluessel> for LandSchluessel {
    fn from(gemeinde: GemeindeverbandSchluessel) -> Self {
        gemeinde.kreis.into()
    }
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
