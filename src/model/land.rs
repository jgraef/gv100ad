use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use chrono::NaiveDate;

use crate::error::ParseKeyError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LandSchluessel {
    land: u8,
}

impl LandSchluessel {
    pub fn new(land: u8) -> Self {
        Self { land }
    }
}

impl FromStr for LandSchluessel {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(ParseKeyError::invalid_length(s, 2));
        }

        let land = s.parse().map_err(|_| ParseKeyError::non_numeric(s))?;

        Ok(Self::new(land))
    }
}

impl Display for LandSchluessel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:02}", self.land)
    }
}

/// A Land (i.e. Bundesland, state) Daten.
#[derive(Clone, Debug)]
pub struct LandDaten {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Landschluessel
    pub schluessel: LandSchluessel,

    /// Name of Land (e.g. `Saarland`)
    pub name: String,

    /// Location of the government of this state.
    pub sitz_regierung: String,
}
