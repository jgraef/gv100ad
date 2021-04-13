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
pub struct RegionSchluessel {
    regierungsbezirk: RegierungsbezirkSchluessel,
    region: u8,
}

impl RegionSchluessel {
    pub fn new(regierungsbezirk: RegierungsbezirkSchluessel, region: u8) -> Self {
        Self {
            regierungsbezirk,
            region,
        }
    }
}

impl FromStr for RegionSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(ParseKeyError::invalid_length(s, 4));
        }

        let regierungsbezirk = s[0..3].parse()?;
        let kreis = s[3..].parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(regierungsbezirk, kreis))
    }
}

impl Display for RegionSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{:1}", self.regierungsbezirk, self.region)
    }
}

impl From<RegionSchluessel> for RegierungsbezirkSchluessel {
    fn from(region: RegionSchluessel) -> Self {
        region.regierungsbezirk
    }
}

impl From<RegionSchluessel> for LandSchluessel {
    fn from(region: RegionSchluessel) -> Self {
        region.regierungsbezirk.into()
    }
}


/// A Region Daten (only Baden-Wuerttemberg)
#[derive(Clone, Debug)]
pub struct RegionDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Regionalschluessel (Land, Regierungsbezirk, Region)
    pub schluessel: RegionSchluessel,

    /// Name of Region
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: String,
}

