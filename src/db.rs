use std::{
    collections::{
        btree_map::{self, BTreeMap},
        HashMap,
    },
    io::BufRead,
    iter::Iterator,
    path::Path,
};

use crate::{
    error::Error,
    model::{
        datensatz::Datensatz,
        gemeinde::{GemeindeDaten, GemeindeSchluessel, RegionalSchluessel},
        gemeindeverband::{GemeindeverbandDaten, GemeindeverbandSchluessel},
        kreis::{KreisDaten, KreisSchluessel},
        land::{LandDaten, LandSchluessel},
        regierungsbezirk::{RegierungsbezirkDaten, RegierungsbezirkSchluessel},
        region::{RegionDaten, RegionSchluessel},
    },
    parser::Parser,
};

/// A (in-memory) database that stores GV100AD data for querying.
#[derive(Clone, Debug, Default)]
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
    gemeindeverbaende: BTreeMap<GemeindeverbandSchluessel, GemeindeverbandDaten>,

    /// Gemeinden
    gemeinden: BTreeMap<GemeindeSchluessel, GemeindeDaten>,

    gemeindeverband_schluessel: HashMap<RegionalSchluessel, u16>,
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
        let mut db = Self::default();

        while let Some(datensatz) = parser.parse_line()? {
            db.insert(datensatz);
        }

        Ok(db)
    }

    pub fn insert(&mut self, datensatz: Datensatz) {
        match datensatz {
            Datensatz::Land(land) => {
                self.laender.insert(land.schluessel, land);
            }
            Datensatz::Regierungsbezirk(regierungsbezirk) => {
                self.regierungsbezirke
                    .insert(regierungsbezirk.schluessel, regierungsbezirk);
            }
            Datensatz::Region(region) => {
                self.regionen.insert(region.schluessel, region);
            }
            Datensatz::Kreis(kreis) => {
                self.kreise.insert(kreis.schluessel, kreis);
            }
            Datensatz::Gemeindeverband(gemeindeverband) => {
                self.gemeindeverbaende
                    .insert(gemeindeverband.schluessel, gemeindeverband);
            }
            Datensatz::Gemeinde(gemeinde) => {
                self.gemeindeverband_schluessel.insert(
                    gemeinde.schluessel.into(),
                    gemeinde.schluessel.gemeindeverband.gemeindeverband,
                );
                self.gemeinden.insert(gemeinde.schluessel.into(), gemeinde);
            }
        }
    }

    pub fn regional_to_gemeinde_schluessel(
        &self,
        regional_schluessel: RegionalSchluessel,
    ) -> Option<GemeindeSchluessel> {
        let gemeindeverband = self.gemeindeverband_schluessel.get(&regional_schluessel)?;
        Some(regional_schluessel.to_gemeinde_schluessel(*gemeindeverband))
    }

    pub fn get<K, V>(&self, k: K) -> Option<&V>
    where
        V: Lookup<K>,
    {
        V::lookup(k, self)
    }

    pub fn all<'a, V>(&'a self) -> V::Iter
    where
        V: IterAll<'a>,
    {
        V::iter_all(self)
    }

    pub fn children<'a, K, V>(&'a self, k: K) -> impl Iterator<Item = &V>
    where
        V: IterChildrenOf<'a>,
        K: IntoRangeKey<V::Key>,
    {
        V::iter_children_of(self, k).map(|(_, v)| v)
    }
}

use std::ops::RangeInclusive;

/// Turns a Regionalschluessel in a range of Regionalschluessel that are
/// contained.
pub trait IntoRangeKey<T> {
    fn into_range_key(self) -> RangeInclusive<T>;
}

/// Creates a range of keys to iterate over all Regierungsbezirke in a Land
impl IntoRangeKey<RegierungsbezirkSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<RegierungsbezirkSchluessel> {
        RegierungsbezirkSchluessel::new(self, u8::MIN)
            ..=RegierungsbezirkSchluessel::new(self, u8::MAX)
    }
}

/// Creates a range of keys to iterate over all Regionen in a Land
impl IntoRangeKey<RegionSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<RegionSchluessel> {
        RegionSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MIN), u8::MIN)
            ..=RegionSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MAX), u8::MAX)
    }
}

/// Creates a range of keys to iterate over all Kreise in a Land
impl IntoRangeKey<KreisSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<KreisSchluessel> {
        KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MIN), u8::MIN)
            ..=KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MAX), u8::MAX)
    }
}

/// Creates a range of keys to iterate over all Gemeindeverbaende in a Land
impl IntoRangeKey<GemeindeverbandSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeverbandSchluessel> {
        GemeindeverbandSchluessel::new(
            KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MIN), u8::MIN),
            u16::MIN,
        )
            ..=GemeindeverbandSchluessel::new(
                KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MAX), u8::MAX),
                u16::MAX,
            )
    }
}

/// Creates a range of keys to iterate over all Gemeinden in a Land
impl IntoRangeKey<GemeindeSchluessel> for LandSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeSchluessel> {
        GemeindeSchluessel::new(
            GemeindeverbandSchluessel::new(
                KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MIN), u8::MIN),
                u16::MIN,
            ),
            u16::MIN,
        )
            ..=GemeindeSchluessel::new(
                GemeindeverbandSchluessel::new(
                    KreisSchluessel::new(RegierungsbezirkSchluessel::new(self, u8::MAX), u8::MAX),
                    u16::MAX,
                ),
                u16::MAX,
            )
    }
}

/// Creates a range of keys to iterate over all Kreise in a Regierungsbezirk
impl IntoRangeKey<RegionSchluessel> for RegierungsbezirkSchluessel {
    fn into_range_key(self) -> RangeInclusive<RegionSchluessel> {
        RegionSchluessel::new(self, u8::MIN)..=RegionSchluessel::new(self, u8::MAX)
    }
}

/// Creates a range of keys to iterate over all Kreise in a Regierungsbezirk
impl IntoRangeKey<KreisSchluessel> for RegierungsbezirkSchluessel {
    fn into_range_key(self) -> RangeInclusive<KreisSchluessel> {
        KreisSchluessel::new(self, u8::MIN)..=KreisSchluessel::new(self, u8::MAX)
    }
}

/// Creates a range of keys to iterate over all Gemeindeverbaende in a
/// Regierungsbezirk
impl IntoRangeKey<GemeindeverbandSchluessel> for RegierungsbezirkSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeverbandSchluessel> {
        GemeindeverbandSchluessel::new(KreisSchluessel::new(self, u8::MIN), u16::MIN)
            ..=GemeindeverbandSchluessel::new(KreisSchluessel::new(self, u8::MAX), u16::MAX)
    }
}

/// Creates a range of keys to iterate over all Gemeinden in a Regierungsbezirk
impl IntoRangeKey<GemeindeSchluessel> for RegierungsbezirkSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeSchluessel> {
        GemeindeSchluessel::new(
            GemeindeverbandSchluessel::new(KreisSchluessel::new(self, u8::MIN), u16::MIN),
            u16::MIN,
        )
            ..=GemeindeSchluessel::new(
                GemeindeverbandSchluessel::new(KreisSchluessel::new(self, u8::MAX), u16::MAX),
                u16::MAX,
            )
    }
}

/// Creates a range of keys to iterate over all Gemeindeverbaende in a Kreis
impl IntoRangeKey<GemeindeverbandSchluessel> for KreisSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeverbandSchluessel> {
        GemeindeverbandSchluessel::new(self, u16::MIN)
            ..=GemeindeverbandSchluessel::new(self, u16::MAX)
    }
}

/// Creates a range of keys to iterate over all Gemeindeverbaende in a Kreis
impl IntoRangeKey<GemeindeSchluessel> for KreisSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeSchluessel> {
        GemeindeSchluessel::new(GemeindeverbandSchluessel::new(self, u16::MIN), u16::MIN)
            ..=GemeindeSchluessel::new(GemeindeverbandSchluessel::new(self, u16::MAX), u16::MAX)
    }
}

/// Creates a range of keys to iterate over all Gemeindeverbaende in a Kreis
impl IntoRangeKey<GemeindeSchluessel> for GemeindeverbandSchluessel {
    fn into_range_key(self) -> RangeInclusive<GemeindeSchluessel> {
        GemeindeSchluessel::new(self, u16::MIN)..=GemeindeSchluessel::new(self, u16::MAX)
    }
}

pub trait Lookup<K> {
    fn lookup<'a>(key: K, db: &'a Database) -> Option<&'a Self>;
}

impl Lookup<LandSchluessel> for LandDaten {
    fn lookup<'a>(key: LandSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.laender.get(&key)
    }
}

impl Lookup<RegierungsbezirkSchluessel> for LandDaten {
    fn lookup<'a>(key: RegierungsbezirkSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.laender.get(&key.into())
    }
}

impl Lookup<RegionSchluessel> for LandDaten {
    fn lookup<'a>(key: RegionSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.laender.get(&key.into())
    }
}

impl Lookup<KreisSchluessel> for LandDaten {
    fn lookup<'a>(key: KreisSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.laender.get(&key.into())
    }
}

impl Lookup<GemeindeverbandSchluessel> for LandDaten {
    fn lookup<'a>(key: GemeindeverbandSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.laender.get(&key.into())
    }
}

impl Lookup<GemeindeSchluessel> for LandDaten {
    fn lookup<'a>(key: GemeindeSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.laender.get(&key.into())
    }
}

impl Lookup<RegierungsbezirkSchluessel> for RegierungsbezirkDaten {
    fn lookup<'a>(key: RegierungsbezirkSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.regierungsbezirke.get(&key)
    }
}

impl Lookup<RegionSchluessel> for RegierungsbezirkDaten {
    fn lookup<'a>(key: RegionSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.regierungsbezirke.get(&key.into())
    }
}

impl Lookup<KreisSchluessel> for RegierungsbezirkDaten {
    fn lookup<'a>(key: KreisSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.regierungsbezirke.get(&key.into())
    }
}

impl Lookup<GemeindeverbandSchluessel> for RegierungsbezirkDaten {
    fn lookup<'a>(key: GemeindeverbandSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.regierungsbezirke.get(&key.into())
    }
}

impl Lookup<GemeindeSchluessel> for RegierungsbezirkDaten {
    fn lookup<'a>(key: GemeindeSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.regierungsbezirke.get(&key.into())
    }
}

impl Lookup<RegionSchluessel> for RegionDaten {
    fn lookup<'a>(key: RegionSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.regionen.get(&key)
    }
}

impl Lookup<KreisSchluessel> for KreisDaten {
    fn lookup<'a>(key: KreisSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.kreise.get(&key)
    }
}

impl Lookup<GemeindeverbandSchluessel> for KreisDaten {
    fn lookup<'a>(key: GemeindeverbandSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.kreise.get(&key.into())
    }
}

impl Lookup<GemeindeSchluessel> for KreisDaten {
    fn lookup<'a>(key: GemeindeSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.kreise.get(&key.into())
    }
}



impl Lookup<GemeindeverbandSchluessel> for GemeindeverbandDaten {
    fn lookup<'a>(key: GemeindeverbandSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.gemeindeverbaende.get(&key)
    }
}

impl Lookup<GemeindeSchluessel> for GemeindeverbandDaten {
    fn lookup<'a>(key: GemeindeSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.gemeindeverbaende.get(&key.into())
    }
}

impl Lookup<GemeindeSchluessel> for GemeindeDaten {
    fn lookup<'a>(key: GemeindeSchluessel, db: &'a Database) -> Option<&'a Self> {
        db.gemeinden.get(&key)
    }
}

impl Lookup<RegionalSchluessel> for GemeindeDaten {
    fn lookup<'a>(key: RegionalSchluessel, db: &'a Database) -> Option<&'a Self> {
        let key = db.regional_to_gemeinde_schluessel(key)?;
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

impl<'a> IterAll<'a> for GemeindeverbandDaten {
    type Iter = btree_map::Values<'a, GemeindeverbandSchluessel, GemeindeverbandDaten>;

    fn iter_all(db: &'a Database) -> Self::Iter {
        db.gemeindeverbaende.values()
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

impl<'a> IterChildrenOf<'a> for GemeindeverbandDaten {
    type Iter = btree_map::Range<'a, Self::Key, Self>;
    type Key = GemeindeverbandSchluessel;

    fn iter_children_of<K: IntoRangeKey<Self::Key>>(db: &'a Database, key: K) -> Self::Iter {
        db.gemeindeverbaende.range(key.into_range_key())
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
        gemeinde::GemeindeDaten,
        kreis::{KreisDaten, KreisSchluessel},
        land::{LandDaten, LandSchluessel},
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
        let land: &LandDaten = db
            .get(KreisSchluessel::new_land(LandSchluessel::new(10), 100))
            .unwrap();
        assert_eq!(land.name, "Saarland");
    }

    #[test]
    fn get_land_from_gemeindeschluessel() {
        let db = load_testset();
        let land: &LandDaten = db
            .get("100420111111".parse::<GemeindeSchluessel>().unwrap())
            .unwrap();
        assert_eq!(land.name, "Saarland");
    }

    #[test]
    fn get_gemeinde() {
        let db = load_testset();
        let gemeinde: &GemeindeDaten = db
            .get("100420111111".parse::<GemeindeSchluessel>().unwrap())
            .unwrap();
        assert_eq!(gemeinde.name, "Beckingen");
    }

    #[test]
    fn get_gemeinde_from_regional_schluessel() {
        let db = load_testset();
        let gemeinde: &GemeindeDaten = db
            .get("10042111".parse::<RegionalSchluessel>().unwrap())
            .unwrap();
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
        let gemeinden = db
            .children::<_, GemeindeDaten>(KreisSchluessel::new_land(LandSchluessel::new(10), 41))
            .collect::<Vec<_>>();

        assert_eq!(gemeinden.len(), 2);
        assert_eq!(gemeinden[0].name, "Saarbrücken, Landeshauptstadt");
        assert_eq!(gemeinden[1].name, "Friedrichsthal, Stadt");
    }
}
