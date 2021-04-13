use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use chrono::NaiveDate;

use crate::error::{Error, ParseKeyError};

use super::{
    kreis::KreisSchluessel,
    land::LandSchluessel,
    regierungsbezirk::RegierungsbezirkSchluessel,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GemeindeSchluessel {
    kreis: KreisSchluessel,
    gemeinde: u16,
}

impl GemeindeSchluessel {
    pub fn new(kreis: KreisSchluessel, gemeinde: u16) -> Self {
        Self { kreis, gemeinde }
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

    /// Specifies type of Gemeinde
    pub textkennzeichen: GemeindeTextkennzeichen,

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

    pub finanzamtbezirk: Option<u16>,

    pub gerichtbarkeit: Option<Gerichtbarkeit>,

    pub arbeitsargenturbezirk: Option<u32>,

    pub bundestagswahlkreise: Option<Bundestagswahlkreise>,
}

/// Information regarding juristical districts
#[derive(Clone, Debug)]
pub struct Gerichtbarkeit {
    pub oberlandesgericht: String,
    pub landgericht: String,
    pub amtsgericht: String,
}

impl FromStr for Gerichtbarkeit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Gerichtbarkeit {
            oberlandesgericht: s[0..1].to_owned(),
            landgericht: s[1..2].to_owned(),
            amtsgericht: s[2..4].to_owned(),
        })
    }
}

/// Associated election districts. If `Range`, it can include gaps.
#[derive(Clone, Debug)]
pub enum Bundestagswahlkreise {
    Single(u16),
    Range(u16, u16),
}

impl FromStr for Bundestagswahlkreise {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let von = s[..3].parse()?;
        tracing::trace!(von = ?von);

        let bis = &s[3..];
        tracing::trace!(bis = ?bis);
        if bis.chars().all(|c| c == ' ') {
            Ok(Bundestagswahlkreise::Single(von))
        } else {
            Ok(Bundestagswahlkreise::Range(von, bis.parse()?))
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GemeindeTextkennzeichen {
    Markt,
    KreisfreieStadt,
    Stadtkreis,
    Stadt,
    KreisangehoerigeGemeinde,
    GemeindefreiesGebietBewohnt,
    GemeindefreiesGebietUnbewohnt,
    GrosseKreisstadt,
}

impl TryFrom<u8> for GemeindeTextkennzeichen {
    type Error = Error;

    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            60 => Ok(Self::Markt),
            61 => Ok(Self::KreisfreieStadt),
            62 => Ok(Self::Stadtkreis),
            63 => Ok(Self::Stadt),
            64 => Ok(Self::KreisangehoerigeGemeinde),
            65 => Ok(Self::GemeindefreiesGebietBewohnt),
            66 => Ok(Self::GemeindefreiesGebietUnbewohnt),
            67 => Ok(Self::GrosseKreisstadt),
            _ => Err(Error::InvalidTextkennzeichen(n)),
        }
    }
}
