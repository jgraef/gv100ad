use std::{
    collections::btree_map::{self, BTreeMap},
    io::BufRead,
    path::Path,
    iter::Iterator,
};

use crate::{
    error::Error,
    model::{
        land::{LandDaten, LandSchluessel},
        regierungsbezirk::{RegierungsbezirkDaten, RegierungsbezirkSchluessel},
        region::{RegionDaten, RegionSchluessel},
        kreis::{KreisDaten, KreisSchluessel},
        gemeindeverband::GemeindeverbandDaten,
        gemeinde::{GemeindeDaten, GemeindeSchluessel},
        datensatz::Datensatz,
    },
    parser::Parser,
};


/// A (in-memory) database that stores GV100AD data for querying.
pub struct Database {
    /// Laender
    laender: BTreeMap<LandSchluessel, LandDaten>,

    /// Regierunzbezirke
    regierungsbezirke: BTreeMap<RegierungsbezirkSchluessel, RegierungsbezirkDaten>,

    /// Regionen (only Baden-Wuerttenberg)
    regionen: BTreeMap<RegionSchluessel, RegionDaten>,

    /// Kreise
    kreise: BTreeMap<KreisSchluessel, KreisDaten>,

    /// Gemeindeverbaende
    gemeindeverbaende: BTreeMap<KreisSchluessel, BTreeMap<u16, GemeindeverbandDaten>>,

    /// Gemeinden
    gemeinden: BTreeMap<GemeindeSchluessel, GemeindeDaten>,
}

impl Database {
    /// Create database from a buffered reader.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, Error> {
        Self::from_parser(Parser::new(reader))
    }

    /// Create database from GV100AD file at `path`.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_parser(Parser::from_path(path)?)
    }

    /// Create database from GV100AD parser.
    pub fn from_parser<R: BufRead>(mut parser: Parser<R>) -> Result<Self, Error> {
        let mut laender = BTreeMap::new();
        let mut regierungsbezirke = BTreeMap::new();
        let mut kreise = BTreeMap::new();
        let mut gemeinden = BTreeMap::new();
        let mut regionen = BTreeMap::new();
        let mut gemeindeverbaende: BTreeMap<KreisSchluessel, BTreeMap<u16, GemeindeverbandDaten>> = BTreeMap::new();

        while let Some(record) = parser.parse_line()? {
            match record {
                Datensatz::Land(land) => {
                    laender.insert(land.schluessel.clone(), land);
                }
                Datensatz::Regierungsbezirk(regierungsbezirk) => {
                    regierungsbezirke.insert(regierungsbezirk.schluessel.clone(), regierungsbezirk);
                }
                Datensatz::Kreis(kreis) => {
                    kreise.insert(kreis.schluessel.clone(), kreis);
                }
                Datensatz::Gemeinde(gemeinde) => {
                    gemeinden.insert(gemeinde.schluessel.clone(), gemeinde);
                }
                Datensatz::Region(region) => {
                    regionen.insert(region.schluessel, region);
                }
                Datensatz::Gemeindeverband(gemeindeverband) => {
                    gemeindeverbaende
                        .entry(gemeindeverband.kreis_schluessel.clone())
                        .or_default()
                        .insert(gemeindeverband.gemeindeverband, gemeindeverband);
                }
            }
        }

        Ok(Self {
            laender,
            regierungsbezirke,
            kreise,
            gemeinden,
            regionen,
            gemeindeverbaende,
        })
    }

    pub fn get<K, V>(&self, k: K) -> Option<&V>
    where 
        V: Lookup,
        V::Key: From<K>,
    {
        V::lookup(k.into(), self)
    }

    pub fn all<'a, I>(&'a self) -> I::Iter
    where
        I: IterAll<'a>,
    {
        I::iter_all(self)
    }

    pub fn children<'a, I, K>(&'a self, k: K) -> I::Iter
    where  
        I: IterChildrenOf<'a>,
        K: IntoRangeKey<I::Key>,
    {
        I::iter_children_of(self, k)
    }
}


use std::ops::RangeInclusive;


/// Turns a Regionalschluessel in a range of Regionalschluessel that are contained.
pub trait IntoRangeKey<T> {
    fn into_range_key(self) -> RangeInclusive<T>;
}

impl IntoRangeKey<RegierungsbezirkSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<RegierungsbezirkSchluessel> {
        RegierungsbezirkSchluessel::new(self, u8::MIN) ..= RegierungsbezirkSchluessel::new(self, u8::MAX)
    }
}

impl IntoRangeKey<RegionSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<RegionSchluessel> {
        RegionSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MIN), u8::MIN) ..= RegionSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MAX), u8::MAX)
    }
}

impl IntoRangeKey<KreisSchluessel> for RegierungsbezirkSchluessel {
    fn into_range_key(self) -> RangeInclusive<KreisSchluessel> {
        KreisSchluessel::new(self, u8::MIN) ..= KreisSchluessel::new(self, u8::MAX)
    }
}

impl IntoRangeKey<KreisSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<KreisSchluessel> {
        KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MIN), u8::MIN) ..= KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MAX), u8::MAX)
    }
}

impl IntoRangeKey<RegionSchluessel> for RegierungsbezirkSchluessel {
    fn into_range_key(self) -> RangeInclusive<RegionSchluessel> {
        RegionSchluessel::new(self, u8::MIN) ..= RegionSchluessel::new(self, u8::MAX)
    }
}

impl IntoRangeKey<GemeindeSchluessel> for KreisSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeSchluessel> {
        GemeindeSchluessel::new(self, u16::MIN) ..= GemeindeSchluessel::new(self, u16::MAX)
    }
}

impl IntoRangeKey<GemeindeSchluessel> for RegierungsbezirkSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeSchluessel> {
        GemeindeSchluessel::new(KreisSchluessel::new(self, u8::MIN), u16::MIN) ..= GemeindeSchluessel::new(KreisSchluessel::new(self, u8::MAX), u16::MAX)
    }
}

impl IntoRangeKey<GemeindeSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeSchluessel> {
        GemeindeSchluessel::new(KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MIN), u8::MIN), u16::MIN) ..= GemeindeSchluessel::new(KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MAX), u8::MAX), u16::MAX)
    }
}



pub trait Lookup {
    type Key;

    fn lookup<'a>(key: Self::Key, db: &'a Database) -> Option<&'a Self>;
}

impl Lookup for LandDaten {
    type Key = LandSchluessel;

    fn lookup<'a>(key: Self::Key, db: &'a Database) -> Option<&'a Self> {
        db.laender.get(&key)
    }
}

impl Lookup for RegierungsbezirkDaten {
    type Key = RegierungsbezirkSchluessel;

    fn lookup<'a>(key: Self::Key, db: &'a Database) -> Option<&'a Self> {
        db.regierungsbezirke.get(&key)
    }
}

impl Lookup for RegionDaten {
    type Key = RegionSchluessel;

    fn lookup<'a>(key: Self::Key, db: &'a Database) -> Option<&'a Self> {
        db.regionen.get(&key)
    }
}

impl Lookup for KreisDaten {
    type Key = KreisSchluessel;

    fn lookup<'a>(key: Self::Key, db: &'a Database) -> Option<&'a Self> {
        db.kreise.get(&key)
    }
}

impl Lookup for GemeindeDaten {
    type Key = GemeindeSchluessel;

    fn lookup<'a>(key: Self::Key, db: &'a Database) -> Option<&'a Self> {
        db.gemeinden.get(&key)
    }
}


/// Trait to iterate over records
pub trait IterAll<'a>: 'a {
    type Iter: Iterator<Item = &'a Self> + 'a;

    fn iter_all(db: &'a Database) -> Self::Iter;
}

impl<'a> IterAll<'a> for LandDaten {
    type Iter = btree_map::Values<'a, LandSchluessel, LandDaten>;

    fn iter_all(db: &'a Database) -> Self::Iter {
        db.laender.values()
    }
}

impl<'a> IterAll<'a> for RegierungsbezirkDaten {
    type Iter = btree_map::Values<'a, RegierungsbezirkSchluessel, RegierungsbezirkDaten>;

    fn iter_all(db: &'a Database) -> Self::Iter {
        db.regierungsbezirke.values()
    }
}

impl<'a> IterAll<'a> for RegionDaten {
    type Iter = btree_map::Values<'a, RegionSchluessel, RegionDaten>;

    fn iter_all(db: &'a Database) -> Self::Iter {
        db.regionen.values()
    }
}

impl<'a> IterAll<'a> for KreisDaten {
    type Iter = btree_map::Values<'a, KreisSchluessel, KreisDaten>;

    fn iter_all(db: &'a Database) -> Self::Iter {
        db.kreise.values()
    }
}

impl<'a> IterAll<'a> for GemeindeDaten {
    type Iter = btree_map::Values<'a, GemeindeSchluessel, GemeindeDaten>;

    fn iter_all(db: &'a Database) -> Self::Iter {
        db.gemeinden.values()
    }
}


/// Trait to iterate over subsets of records (e.g. all Kreise in Saarland).
pub trait IterChildrenOf<'a>: 'a {
    type Iter: Iterator<Item = (&'a Self::Key, &'a Self)>;
    type Key;

    fn iter_children_of<K: IntoRangeKey<Self::Key>>(db: &'a Database, key: K) -> Self::Iter;
}

impl<'a> IterChildrenOf<'a> for RegierungsbezirkDaten {
    type Iter = btree_map::Range<'a, Self::Key, Self>;
    type Key = RegierungsbezirkSchluessel;

    fn iter_children_of<K: IntoRangeKey<Self::Key>>(db: &'a Database, key: K) -> Self::Iter {
        db.regierungsbezirke.range(key.into_range_key())
    }
}

impl<'a> IterChildrenOf<'a> for RegionDaten {
    type Iter = btree_map::Range<'a, Self::Key, Self>;
    type Key = RegionSchluessel;

    fn iter_children_of<K: IntoRangeKey<Self::Key>>(db: &'a Database, key: K) -> Self::Iter {
        db.regionen.range(key.into_range_key())
    }
}

impl<'a> IterChildrenOf<'a> for KreisDaten {
    type Iter = btree_map::Range<'a, Self::Key, Self>;
    type Key = KreisSchluessel;

    fn iter_children_of<K: IntoRangeKey<Self::Key>>(db: &'a Database, key: K) -> Self::Iter {
        db.kreise.range(key.into_range_key())
    }
}

impl<'a> IterChildrenOf<'a> for GemeindeDaten {
    type Iter = btree_map::Range<'a, Self::Key, Self>;
    type Key = GemeindeSchluessel;

    fn iter_children_of<K: IntoRangeKey<Self::Key>>(db: &'a Database, key: K) -> Self::Iter {
        db.gemeinden.range(key.into_range_key())
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::model::{
        land::{LandSchluessel, LandDaten},
        kreis::{KreisSchluessel, KreisDaten},
        gemeinde::GemeindeDaten,
    };

    use super::*;


    fn load_testset() -> Database {
        let data = r#"102021043010          Saarland                                          Saarbrücken, Landeshauptstadt                                                                                                                       
402021043010041       Regionalverband Saarbrücken                       Saarbrücken, Landeshauptstadt                     45                                                                                                
502021043010041   0100Saarbrücken, Landeshauptstadt                                                                       50                                                                                                
502021043010041   0511Friedrichsthal, Stadt                                                                               50                                                                                                
6020210430100411000100Saarbrücken, Landeshauptstadt                                                                       63    000000167520000018037400000089528    66111*****  1040110955501296                           
6020210430100415110511Friedrichsthal, Stadt                                                                               63    000000008990000000998700000004907    66299       1070110955513299                           
402021043010042       Merzig-Wadern                                     Merzig, Kreisstadt                                44                                                                                                
502021043010042   0111Beckingen                                                                                           50                                                                                                
502021043010042   0112Losheim am See                                                                                      50                                                                                                
6020210430100421110111Beckingen                                                                                           64    000000051850000001488900000007315    66701       1020110455523297                           
6020210430100421120112Losheim am See                                                                                      64    000000096950000001603800000007974    66679       1020110455525297                           
102021043011          Berlin                                            Berlin                                                                                                                                              "#;

        Database::from_reader(Cursor::new(data)).unwrap()
    }

    #[test]
    fn get_land_from_landschluessel() {
        let db = load_testset();
        let land: &LandDaten = db.get(LandSchluessel::new(10)).unwrap();
        assert_eq!(land.name, "Saarland");
    }

    #[test]
    fn get_land_from_kreisschluessel() {
        let db = load_testset();
        let land: &LandDaten = db.get(KreisSchluessel::new_land(LandSchluessel::new(10), 100)).unwrap();
        assert_eq!(land.name, "Saarland");
    }

    #[test]
    fn get_land_from_gemeindeschluessel() {
        let db = load_testset();
        let land: &LandDaten = db.get("10042111".parse::<GemeindeSchluessel>().unwrap()).unwrap();
        assert_eq!(land.name, "Saarland");
    }

    #[test]
    fn get_gemeinde() {
        let db = load_testset();
        let gemeinde: &GemeindeDaten = db.get("10042111".parse::<GemeindeSchluessel>().unwrap()).unwrap();
        assert_eq!(gemeinde.name, "Beckingen");
    }

    #[test]
    fn iter_all_laender() {
        let db = load_testset();
        let laender = db.all::<LandDaten>().collect::<Vec<_>>();

        assert_eq!(laender.len(), 2);
        assert_eq!(laender[0].name, "Saarland");
        assert_eq!(laender[1].name, "Berlin");
    }

    #[test]
    fn iter_all_kreise() {
        let db = load_testset();
        let kreise = db.all::<KreisDaten>().collect::<Vec<_>>();

        assert_eq!(kreise.len(), 2);
        assert_eq!(kreise[0].name, "Regionalverband Saarbrücken");
        assert_eq!(kreise[1].name, "Merzig-Wadern");
    }

    #[test]
    fn iter_all_gemeinden() {
        let db = load_testset();
        let gemeinden = db.all::<GemeindeDaten>().collect::<Vec<_>>();

        assert_eq!(gemeinden.len(), 4);
        assert_eq!(gemeinden[0].name, "Saarbrücken, Landeshauptstadt");
        assert_eq!(gemeinden[1].name, "Friedrichsthal, Stadt");
        assert_eq!(gemeinden[2].name, "Beckingen");
        assert_eq!(gemeinden[3].name, "Losheim am See");
    }

    #[test]
    fn iter_gemeinden_in_kreis() {
        let db = load_testset();
        let gemeinden = db.children::<GemeindeDaten, _>(KreisSchluessel::new_land(LandSchluessel::new(10), 41))
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

        assert_eq!(gemeinden.len(), 2);
        assert_eq!(gemeinden[0].name, "Saarbrücken, Landeshauptstadt");
        assert_eq!(gemeinden[1].name, "Friedrichsthal, Stadt");
    }
}
