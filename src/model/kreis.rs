use std::{
    convert::TryFrom,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use chrono::NaiveDate;

use crate::error::{Error, ParseKeyError};

use super::{land::LandSchluessel, regierungsbezirk::RegierungsbezirkSchluessel};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KreisSchluessel {
    pub regierungsbezirk: RegierungsbezirkSchluessel,
    pub kreis: u8,
}

impl KreisSchluessel {
    pub fn new(regierungsbezirk: RegierungsbezirkSchluessel, kreis: u8) -> Self {
        Self {
            regierungsbezirk,
            kreis,
        }
    }

    /// Creates a Kreisschluessel directly from the parent Landschluessel and
    /// the Kreis identifier. This sets the Regierungsbezirk part to 0.
    pub fn new_land(land: LandSchluessel, kreis: u8) -> Self {
        Self {
            regierungsbezirk: RegierungsbezirkSchluessel::new(land, 0),
            kreis,
        }
    }
}

impl FromStr for KreisSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 5 {
            return Err(ParseKeyError::invalid_length(s, 5));
        }

        let regierungsbezirk = s[0..3].parse()?;
        let kreis = s[3..].parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(regierungsbezirk, kreis))
    }
}

impl Display for KreisSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{:2}", self.regierungsbezirk, self.kreis)
    }
}

impl From<KreisSchluessel> for RegierungsbezirkSchluessel {
    fn from(kreis: KreisSchluessel) -> Self {
        kreis.regierungsbezirk
    }
}

impl From<KreisSchluessel> for LandSchluessel {
    fn from(kreis: KreisSchluessel) -> Self {
        kreis.regierungsbezirk.into()
    }
}

/// A Kreis Daten
#[derive(Clone, Debug)]
pub struct KreisDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Kreisschluessel
    pub schluessel: KreisSchluessel,

    /// Name of Kreis
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: String,

    /// Specifies type of Kreis
    pub textkennzeichen: KreisTextkennzeichen,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum KreisTextkennzeichen {
    KreisfreieStadt,
    Stadtkreis,
    Kreis,
    Landkreis,
    Regionalverband,
}

impl TryFrom<u8> for KreisTextkennzeichen {
    type Error = Error;

    fn try_from(n: u8) -> Result<Self, Error> {
        match n {
            41 => Ok(Self::KreisfreieStadt),
            42 => Ok(Self::Stadtkreis),
            43 => Ok(Self::Kreis),
            44 => Ok(Self::Landkreis),
            45 => Ok(Self::Regionalverband),
            _ => Err(Error::InvalidTextkennzeichen(n)),
        }
    }
}
