use std::{
    str::FromStr,
    fmt::{Display, Formatter, self},
};

use chrono::NaiveDate;

use crate::error::ParseKeyError;

use super::{
    kreis::KreisSchluessel,
    regierungsbezirk::RegierungsbezirkSchluessel,
    land::LandSchluessel,
};


#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GemeindeSchluessel {
    kreis: KreisSchluessel,
    gemeinde: u16,
}

impl GemeindeSchluessel {
    pub fn new(kreis: KreisSchluessel, gemeinde: u16) -> Self {
        Self {
            kreis,
            gemeinde,
        }
    }
}

impl FromStr for GemeindeSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 8 {
            return Err(ParseKeyError::invalid_length(s, 8));
        }

        let regierungsbezirk = s[0..5].parse()?;
        let kreis = s[5..].parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(regierungsbezirk, kreis))
    }
}

impl Display for GemeindeSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{:3}", self.kreis, self.gemeinde)
    }
}


impl From<GemeindeSchluessel> for KreisSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        gemeinde.kreis
    }
}

impl From<GemeindeSchluessel> for RegierungsbezirkSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        gemeinde.kreis.into()
    }
}

impl From<GemeindeSchluessel> for LandSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        gemeinde.kreis.into()
    }
}


/// # Todo
/// 
///  - Add missing data
/// 
#[derive(Clone, Debug)]
pub struct GemeindeDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Gemeindeschluessel
    pub schluessel: GemeindeSchluessel,

    /// Identifier of the Gemeinderverband this Gemeinde belongs to.
    pub gemeindeverband: u16,

    /// Name of Gemeinde
    pub name: String,

    /// Area in hectare (10000 square-meter)
    pub area: u64,

    /// Total population
    pub population_total: u64,

    /// Male population
    pub population_male: u64,

    /// Postleitzahl (PLZ, Postcode)
    pub plz: String,

    /// Whether the PLZ is unambiguous or not
    pub plz_unambiguous: bool,

    pub finanzamtbezirk: u16,

    pub gerichtbarkeit: Gerichtbarkeit,

    pub arbeitsargenturbezirk: u32,

    pub bundestagswahlkreise: Bundestagswahlkreise,
}

/// Information regarding juristical districts
#[derive(Clone, Debug)]
pub struct Gerichtbarkeit {
    pub oberlandesgericht: u8,
    pub landgericht: u8,
    pub amtsgericht: u8,
}

/// Associated election districts. If `Range`, it can include gaps.
#[derive(Clone, Debug)]
pub enum Bundestagswahlkreise {
    Single(u16),
    Range(u16, u16),
}
