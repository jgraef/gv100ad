use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use chrono::NaiveDate;

use crate::error::{Error, ParseKeyError};

use super::{
    gemeindeverband::GemeindeverbandSchluessel,
    kreis::KreisSchluessel,
    land::LandSchluessel,
    regierungsbezirk::RegierungsbezirkSchluessel,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RegionalSchluessel {
    pub kreis: KreisSchluessel,
    pub gemeinde: u16,
}

impl RegionalSchluessel {
    pub fn new(kreis: KreisSchluessel, gemeinde: u16) -> Self {
        Self { kreis, gemeinde }
    }

    pub fn to_gemeinde_schluessel(self, gemeindeverband: u16) -> GemeindeSchluessel {
        GemeindeSchluessel::from_regional_schluessel(self, gemeindeverband)
    }
}

impl FromStr for RegionalSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 8 {
            return Err(ParseKeyError::invalid_length(s, 8));
        }

        let kreis = s[0..5].parse()?;
        let gemeinde = s[5..].parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(kreis, gemeinde))
    }
}

impl Display for RegionalSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{:3}", self.kreis, self.gemeinde)
    }
}

impl From<RegionalSchluessel> for KreisSchluessel {
    fn from(gemeinde: RegionalSchluessel) -> Self {
        gemeinde.kreis
    }
}

impl From<RegionalSchluessel> for RegierungsbezirkSchluessel {
    fn from(gemeinde: RegionalSchluessel) -> Self {
        gemeinde.kreis.into()
    }
}

impl From<RegionalSchluessel> for LandSchluessel {
    fn from(gemeinde: RegionalSchluessel) -> Self {
        gemeinde.kreis.into()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GemeindeSchluessel {
    pub gemeindeverband: GemeindeverbandSchluessel,
    pub gemeinde: u16,
}

impl GemeindeSchluessel {
    pub fn new(gemeindeverband: GemeindeverbandSchluessel, gemeinde: u16) -> Self {
        Self {
            gemeindeverband,
            gemeinde,
        }
    }

    pub fn from_regional_schluessel(
        regional_schluessel: RegionalSchluessel,
        gemeindeverband: u16,
    ) -> Self {
        Self {
            gemeindeverband: GemeindeverbandSchluessel::new(
                regional_schluessel.kreis,
                gemeindeverband,
            ),
            gemeinde: regional_schluessel.gemeinde,
        }
    }
}

impl FromStr for GemeindeSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 12 {
            return Err(ParseKeyError::invalid_length(s, 12));
        }

        let gemeindeverband = s[0..9].parse()?;
        let gemeinde = s[9..].parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(gemeindeverband, gemeinde))
    }
}

impl Display for GemeindeSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{:3}", self.gemeindeverband, self.gemeinde)
    }
}

impl From<GemeindeSchluessel> for GemeindeverbandSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        gemeinde.gemeindeverband
    }
}

impl From<GemeindeSchluessel> for KreisSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        gemeinde.gemeindeverband.into()
    }
}

impl From<GemeindeSchluessel> for RegierungsbezirkSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        gemeinde.gemeindeverband.into()
    }
}

impl From<GemeindeSchluessel> for LandSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        gemeinde.gemeindeverband.into()
    }
}

impl From<GemeindeSchluessel> for RegionalSchluessel {
    fn from(gemeinde: GemeindeSchluessel) -> Self {
        let kreis = gemeinde.gemeindeverband.into();
        RegionalSchluessel {
            kreis,
            gemeinde: gemeinde.gemeinde,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GemeindeDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Gemeindeschluessel
    pub schluessel: GemeindeSchluessel,

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

impl GemeindeDaten {
    pub fn regional_schluessel(&self) -> RegionalSchluessel {
        self.schluessel.into()
    }
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
