use std::{
    str::FromStr,
    fmt::{Display, Formatter, self},
};

use chrono::NaiveDate;

use crate::error::ParseKeyError;

use super::{
    land::LandSchluessel,
};


#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RegierungsbezirkSchluessel {
    land: LandSchluessel,
    regierungsbezirk: u8,
}

impl RegierungsbezirkSchluessel {
    pub fn new(land: LandSchluessel, regierungsbezirk: u8) -> Self {
        Self {
            land,
            regierungsbezirk,
        }
    }
}

impl FromStr for RegierungsbezirkSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 3 {
            return Err(ParseKeyError::invalid_length(s, 3));
        }

        let land = s[0..2].parse()?;
        let regierungsbezirk = s[2..].parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(land, regierungsbezirk))
    }
}

impl Display for RegierungsbezirkSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{:1}", self.land, self.regierungsbezirk)
    }
}

impl From<RegierungsbezirkSchluessel> for LandSchluessel {
    fn from(regierungsbezirk: RegierungsbezirkSchluessel) -> Self {
        regierungsbezirk.land
    }
}

/// A Regierunsbezirk Daten (government district)
#[derive(Clone, Debug)]
pub struct RegierungsbezirkDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Gemeindeschluessel
    pub schluessel: RegierungsbezirkSchluessel,

    /// Name of Regierunsbezirk
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: String,
}
