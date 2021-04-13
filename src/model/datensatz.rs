use chrono::NaiveDate;

use super::{
    gemeinde::GemeindeDaten,
    gemeindeverband::GemeindeverbandDaten,
    kreis::KreisDaten,
    land::LandDaten,
    regierungsbezirk::RegierungsbezirkDaten,
    region::RegionDaten,
};

/// A GV100AD Daten (Datensatz).
#[derive(Clone, Debug)]
pub enum Datensatz {
    Land(LandDaten),
    Regierungsbezirk(RegierungsbezirkDaten),
    Region(RegionDaten),
    Kreis(KreisDaten),
    Gemeindeverband(GemeindeverbandDaten),
    Gemeinde(GemeindeDaten),
}

impl Datensatz {
    /// Returns the Gebietsstand (i.e. timestamp) of the Daten.
    pub fn gebietsstand(&self) -> &NaiveDate {
        match self {
            Self::Land(land) => &land.gebietsstand,
            Self::Regierungsbezirk(regierungsbezirk) => &regierungsbezirk.gebietsstand,
            Self::Region(_region) => todo!(),
            Self::Kreis(kreis) => &kreis.gebietsstand,
            Self::Gemeindeverband(gemeindeverband) => &gemeindeverband.gebietsstand,
            Self::Gemeinde(gemeinde) => &gemeinde.gebietsstand,
        }
    }

    /// Returns the name of the unit.
    pub fn name(&self) -> &str {
        match self {
            Self::Land(land) => &land.name,
            Self::Regierungsbezirk(regierungsbezirk) => &regierungsbezirk.name,
            Self::Region(_region) => todo!(),
            Self::Kreis(kreis) => &kreis.name,
            Self::Gemeindeverband(gemeindeverband) => &gemeindeverband.name,
            Self::Gemeinde(gemeinde) => &gemeinde.name,
        }
    }
}
