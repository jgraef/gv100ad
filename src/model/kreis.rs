use std::{
    str::FromStr,
    fmt::{Display, Formatter, self},
};

use chrono::NaiveDate;

use crate::error::ParseKeyError;

use super::{
    regierungsbezirk::RegierungsbezirkSchluessel,
    land::LandSchluessel,
};


#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KreisSchluessel {
    regierungsbezirk: RegierungsbezirkSchluessel,
    kreis: u8,
}

impl KreisSchluessel {
    pub fn new(regierungsbezirk: RegierungsbezirkSchluessel, kreis: u8) -> Self {
        Self {
            regierungsbezirk,
            kreis,
        }
    }

    /// Creates a Kreisschluessel directly from the parent Landschluessel and the Kreis identifier. This sets the Regierungsbezirk part to 0.
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
}
