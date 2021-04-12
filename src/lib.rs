///
/// Parser for GV100 files from the Statistisches Bundesamt, containing data about German Laender, Kreise, Gemeinden, etc.
///
/// Download data files from: https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/_inhalt.html
///
/// # TODO
///
///  - Implement Textkennzeichen correctly.
///
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader, Error as IoError},
    num::ParseIntError,
    path::Path,
    str::{Chars, FromStr},
};

use chrono::NaiveDate;
use thiserror::Error;

/// Error type returned by parser.
#[derive(Debug, Error)]
pub enum Error {
    /// An IO error occured while reading the data.
    #[error("IO error: {0}")]
    Io(#[from] IoError),

    /// Failed to parse a number.
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),

    /// The type of a data record is invalid.
    #[error("Invalid type: {0}")]
    InvalidType(u8),

    /// A invalid "Textkennzeichen" was read.
    #[error("Invalid Textkennzeichen: {0}")]
    InvalidTextkennzeichen(u8),
}

/// Reader to read fields from a single data record (i.e. line). Specifically this makes sure that data is read correctly as UTF-8.
pub struct FieldReader<'a> {
    it: Chars<'a>,
}

impl<'a> FieldReader<'a> {
    /// Creates a new field reader from a single line. It expects the line to not contain any line terminator.
    pub fn new(buf: &'a str) -> Self {
        FieldReader { it: buf.chars() }
    }

    /// Reads a field of length `n` as string. `n` is in characters, not bytes.
    pub fn next(&mut self, n: usize) -> &str {
        let s = self.it.as_str();

        // Count how many bytes need to be read, to read `n` UTF-8 characters.
        let mut nb = 0;
        for _ in 0..n {
            if let Some(c) = self.it.next() {
                nb += c.len_utf8();
            } else {
                break;
            }
        }

        &s[0..nb]
    }

    /// Reads a field of length `n` and parses it as `T`.
    pub fn parse_next<T: FromStr>(&mut self, n: usize) -> Result<T, <T as FromStr>::Err> {
        self.next(n).parse()
    }

    /// Skips `n` characters.
    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.it.next();
        }
    }
}

/// Parser for GV100 files.
pub struct Parser<R> {
    reader: R,
}

impl Parser<BufReader<File>> {
    /// Creates a new parser from a file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, IoError> {
        let reader = BufReader::new(File::open(path)?);
        Ok(Self::new(reader))
    }
}

impl<R: BufRead> Iterator for Parser<R> {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_line().transpose()
    }
}

impl<R: BufRead> Parser<R> {
    /// Creates a new parser from a `BufRead`.
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Parses the next data record (i.e. line).
    ///
    /// Returns `Ok(None)` if end of file is reached. Returns `Err(_)`, if an error occured, otherwise returns `Ok(Some(_))`, if a
    /// record was successfully read.
    pub fn parse_line(&mut self) -> Result<Option<Record>, Error> {
        let mut buf = String::new();

        if self.reader.read_line(&mut buf)? == 0 {
            // EOF
            return Ok(None);
        }

        // Remove trailing line terminator.
        while buf.ends_with('\n') || buf.ends_with('\r') {
            buf.pop();
        }

        // Create field reader.
        let mut fields = FieldReader::new(&buf);

        // Read type (Satzart)
        let ty = fields.parse_next::<u8>(2)?;

        let record = match ty {
            10 => {
                // Landdaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let ags = fields.parse_next(2)?;

                fields.skip(10);

                let name = fields.next(50).trim().to_owned();

                let sitz_regierung = fields.next(50).trim().to_owned();

                Record::Land(Land {
                    gebietsstand,
                    ags,
                    name,
                    sitz_regierung,
                })
            }
            20 => {
                // Regierungsbezirkdaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let ags = fields.parse_next(3)?;

                fields.skip(9);

                let name = fields.next(50).trim().to_owned();

                let sitz_verwaltung = fields.next(50).trim().to_owned();

                Record::Regierungsbezirk(Regierungsbezirk {
                    gebietsstand,
                    ags,
                    name,
                    sitz_verwaltung,
                })
            }
            30 => {
                // Regionsdaten (nur Baden-Wuerttenberg)

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let ags = fields.parse_next(3)?;
                tracing::debug!(ags = ?ags);

                let region = fields.parse_next(1)?;
                tracing::debug!(region = ?region);

                let name = fields.next(50).trim().to_owned();
                tracing::debug!(name = ?name);

                let sitz_verwaltung = fields.next(50).trim().to_owned();
                tracing::debug!(sitz_verwaltung = ?sitz_verwaltung);

                Record::Region(Region {
                    gebietsstand,
                    ags,
                    region,
                    name,
                    sitz_verwaltung,
                })
            }
            40 => {
                // Kreisdaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let ags = fields.parse_next(5)?;
                tracing::debug!(ags = ?ags);

                fields.skip(7);

                let name = fields.next(50).trim().to_owned();
                tracing::debug!(name = ?name);

                let sitz_verwaltung = fields.next(50).trim().to_owned();
                tracing::debug!(sitz_verwaltung = ?sitz_verwaltung);

                //let subtype = fields.parse_next(2)?;
                //tracing::debug!(subtype = ?subtype);
                //fields.skip(2);

                Record::Kreis(Kreis {
                    gebietsstand,
                    ags,
                    name,
                    sitz_verwaltung,
                })
            }
            50 => {
                // Gemeindeverbandsdaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let ags = fields.parse_next(5)?;
                tracing::debug!(ags = ?ags);

                fields.skip(3);

                let gemeindeverband = fields.parse_next(4)?;
                tracing::debug!(gemeindeverband = ?gemeindeverband);

                let name = fields.next(50).trim().to_owned();
                tracing::debug!(name = ?name);

                let sitz_verwaltung = fields.next(50).trim().to_owned();
                tracing::debug!(sitz_verwaltung = ?sitz_verwaltung);

                //let subtype = fields.parse_next(2)?;
                //tracing::debug!(subtype = ?subtype);

                Record::Gemeindeverband(Gemeindeverband {
                    gebietsstand,
                    ags,
                    gemeindeverband,
                    name,
                    sitz_verwaltung,
                })
            }
            60 => {
                // Gemeindedaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let ags = fields.parse_next(8)?;
                tracing::debug!(ags = ?ags);

                let gemeindeverband = fields.parse_next(4)?;
                tracing::debug!(gemeindeverband = ?gemeindeverband);

                let name = fields.next(50).trim().to_owned();
                tracing::debug!(name = ?name);

                fields.skip(50);

                //let subtype = fields.parse_next(2)?;
                //tracing::debug!(subtype = ?subtype);
                fields.skip(2);

                fields.skip(4);

                let area = fields.parse_next(11)?;
                tracing::debug!(area = ?area);

                let population_total = fields.parse_next(11)?;
                tracing::debug!(population_total = ?population_total);

                let population_male = fields.parse_next(11)?;
                tracing::debug!(population_male = ?population_male);

                fields.skip(4);

                let plz = fields.next(5).to_owned();
                tracing::debug!(plz = ?plz);

                let plz_alt = {
                    let s = fields.next(5).to_owned();
                    if s.trim().is_empty() {
                        None
                    } else {
                        Some(s.to_owned())
                    }
                };

                tracing::debug!(plz_alt = ?plz_alt);

                fields.skip(2);

                let finanzamtbezirk = fields.next(4);
                tracing::debug!(finanzamtbezirk = ?finanzamtbezirk);

                /*let gerichtbarkeit = Gerichtbarkeit {
                    oberlandesgericht: fields.parse_next(1)?,
                    landgericht: fields.parse_next(1)?,
                    amtsgericht: fields.parse_next(2)?,
                };
                tracing::debug!(gerichtbarkeit = ?gerichtbarkeit);*/
                fields.skip(4);

                let arbeitsargenturbezirk = fields.next(5);
                tracing::debug!(arbeitsargenturbezirk = ?arbeitsargenturbezirk);

                let bundestagswahlkreise_von = fields.next(3);
                tracing::debug!(bundestagswahlkreise_von = ?bundestagswahlkreise_von);

                let bundestagswahlkreise_bis = fields.next(3);
                tracing::debug!(bundestagswahlkreise_bis = ?bundestagswahlkreise_bis);
                //fields.skip(4);
                //fields.skip(20);

                Record::Gemeinde(Gemeinde {
                    gebietsstand,
                    ags,
                    gemeindeverband,
                    name,
                    area,
                    population_total,
                    population_male,
                    plz,
                })
            }
            ty => return Err(Error::InvalidType(ty)),
        };

        tracing::debug!("{:#?}", record);

        Ok(Some(record))
    }
}

/// Parses date from a field. This is just year, month, day without any seperators. German timezones apply.
pub fn parse_date(s: &str) -> Result<NaiveDate, ParseIntError> {
    Ok(NaiveDate::from_ymd(s[0..4].parse()?, s[4..6].parse()?, s[6..8].parse()?))
}

/// A GV100 record (Datensatz).
#[derive(Clone, Debug)]
pub enum Record {
    Land(Land),
    Regierungsbezirk(Regierungsbezirk),
    Region(Region),
    Kreis(Kreis),
    Gemeindeverband(Gemeindeverband),
    Gemeinde(Gemeinde),
}

impl Record {
    /// Returns the Gebietsstand (i.e. timestamp) of the record.
    pub fn gebietsstand(&self) -> &NaiveDate {
        match self {
            Record::Land(land) => &land.gebietsstand,
            Record::Regierungsbezirk(regierungsbezirk) => &regierungsbezirk.gebietsstand,
            Record::Region(_region) => todo!(),
            Record::Kreis(kreis) => &kreis.gebietsstand,
            Record::Gemeindeverband(gemeindeverband) => &gemeindeverband.gebietsstand,
            Record::Gemeinde(gemeinde) => &gemeinde.gebietsstand,
        }
    }

    /// Returns the [[Ags]] (Amtliche Gemeindeschluessel) for the entry. For Land, Regierungsbezirk, Kreis or Gemeinde this
    /// is it's unique identifier. For Region, Gemeindeverband it's the Ags of the parent unit. A Region is then further identified
    /// by it's `region` field, and a Gemeindeverband is identified by it's `gemeindeverband` field.
    pub fn ags(&self) -> &Ags {
        match self {
            Record::Land(land) => &land.ags,
            Record::Regierungsbezirk(regierungsbezirk) => &regierungsbezirk.ags,
            Record::Region(region) => &region.ags,
            Record::Kreis(kreis) => &kreis.ags,
            Record::Gemeindeverband(gemeindeverband) => &gemeindeverband.ags,
            Record::Gemeinde(gemeinde) => &gemeinde.ags,
        }
    }

    /// Returns the name of the unit.
    pub fn name(&self) -> &str {
        match self {
            Record::Land(land) => &land.name,
            Record::Regierungsbezirk(regierungsbezirk) => &regierungsbezirk.name,
            Record::Region(_region) => todo!(),
            Record::Kreis(kreis) => &kreis.name,
            Record::Gemeindeverband(gemeindeverband) => &gemeindeverband.name,
            Record::Gemeinde(gemeinde) => &gemeinde.name,
        }
    }
}

/// A Land (i.e. Bundesland, state) record.
#[derive(Clone, Debug)]
pub struct Land {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Amtlicher Gemeindeschluessel
    pub ags: Ags,

    /// Name of Land (e.g. `Saarland`)
    pub name: String,

    /// Location of the government of this state.
    pub sitz_regierung: String,
}

/// A Regierunsbezirk record (government district)
#[derive(Clone, Debug)]
pub struct Regierungsbezirk {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Amtlicher Gemeindeschluessel
    pub ags: Ags,

    /// Name of Regierunsbezirk
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: String,
}

/// A Region record (only Baden-Wuerttemberg)
#[derive(Clone, Debug)]
pub struct Region {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Amtlicher Gemeindeschluessel
    pub ags: Ags,

    /// Region identifier. This together with `ags` uniquely identifies a Region. In theory the ags can also be ommitted, since the
    /// next higher unit is a Land, but this record only applies to the Land Baden-Wuerttemberg.
    pub region: u8,

    /// Name of Region
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: String,
}

/// A Kreis record
#[derive(Clone, Debug)]
pub struct Kreis {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Amtlicher Gemeindeschluessel
    pub ags: Ags,

    /// Name of Kreis
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: String,
}

#[derive(Clone, Debug)]
pub struct Gemeindeverband {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Amtlicher Gemeindeschluessel
    pub ags: Ags,

    /// Identifier of Gemeindeverband. This together with `ags` uniquely identifies a Gemeinderverband.
    pub gemeindeverband: u16,

    /// Name of Gemeindeverband
    pub name: String,

    /// Location of administration
    pub sitz_verwaltung: String,
}

#[derive(Clone, Debug)]
pub struct Gemeinde {
    /// Timestamp
    pub gebietsstand: NaiveDate,

    /// Amtlicher Gemeindeschluessel
    pub ags: Ags,

    /// Identifier of the Gemeinderverband this Gemeinde belongs to.
    pub gemeindeverband: u16,

    /// Name of Gemeinde
    pub name: String,

    /// Area in hectare (10000 square-meter)
    pub area: u64,

    /// Total population
    pub population_total: u64,

    /// Male population
    pub population_male: u64,

    /// Postleitzahl (PLZ, Postcode)
    pub plz: String,
}

/// FIXME: Different numbers can have the same meaning, but it depends in which kind of record it is used.
#[derive(Clone, Debug)]
pub enum Textkennzeichen {
    Markt,
    KreisfreieStadt,
    Stadtkreis,
    Stadt,
    KreisangehoerigeGemeinde,
    GemeindefreiesGebietBewohnt,
    GemeindefreiesGebietUnbewohnt,
    GrosseKreisstadt,
}

impl FromStr for Textkennzeichen {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.parse()?;

        Ok(match c {
            60 => Self::Markt,
            61 => Self::KreisfreieStadt,
            62 => Self::Stadtkreis,
            63 => Self::Stadt,
            64 => Self::KreisangehoerigeGemeinde,
            65 => Self::GemeindefreiesGebietBewohnt,
            66 => Self::GemeindefreiesGebietUnbewohnt,
            67 => Self::GrosseKreisstadt,
            _ => return Err(Error::InvalidTextkennzeichen(c)),
        })
    }
}

/// Amtlicher Gemeindeschluessel
///
/// # FIXME
///
/// This is called "Regionalschluessel" in the description, but information about this is ambiguous.
///
/// [1] https://en.wikipedia.org/wiki/Community_Identification_Number#Germany
///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ags {
    pub land: u8,
    pub regierungsbezirk: Option<u8>,
    pub kreis: Option<u8>,
    pub gemeinde: Option<u16>,
}

impl FromStr for Ags {
    type Err = Error;

    /// Parses AGS from string. The string can be of forms:
    ///
    ///  * Land
    ///  * Land, Regierungzbezirk
    ///  * Land, Regierungzbezirk, Kreis
    ///  * Land, Regierungzbezirk, Kreis, Gemeinde
    ///
    /// Where each form identifies a unit on the particular level.
    ///
    /// Each part consists of one or more digits, if not omitted:
    ///
    ///  * Land: 2 digits
    ///  * Regierungsbezirk: 1 digit
    ///  * Kreis: 2 digits
    ///  * Gemeinde: 3 digits
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let land = s[0..2].parse()?;

        let regierungsbezirk = s.get(2..3).map(|s| s.parse()).transpose()?;
        let kreis = s.get(3..5).map(|s| s.parse()).transpose()?;
        let gemeinde = s.get(5..8).map(|s| s.parse()).transpose()?;

        Ok(Self {
            land,
            regierungsbezirk,
            kreis,
            gemeinde,
        })
    }
}

impl Ags {
    /// Creates an AGS that identifies a Land.
    pub fn new_land(land: u8) -> Self {
        Self::new(land, None, None, None)
    }

    /// Creates an AGS that identifies a Regierungsbezirk.
    pub fn new_regierungsbezirk(land: u8, regierungsbezirk: u8) -> Self {
        Self::new(land, Some(regierungsbezirk), None, None)
    }

    /// Creates an AGS that identifies a Kreis.
    pub fn new_kreis(land: u8, regierungsbezirk: u8, kreis: u8) -> Self {
        Self::new(land, Some(regierungsbezirk), Some(kreis), None)
    }

    /// Creates an AGS that identifies a Gemeinde.
    pub fn new_gemeinde(land: u8, regierungsbezirk: u8, kreis: u8, gemeinde: u16) -> Self {
        Self::new(land, Some(regierungsbezirk), Some(kreis), Some(gemeinde))
    }

    fn new(land: u8, regierungsbezirk: Option<u8>, kreis: Option<u8>, gemeinde: Option<u16>) -> Self {
        Self {
            land,
            regierungsbezirk,
            kreis,
            gemeinde,
        }
    }

    /// Converts a AGS that is at least specific to a Regierungsbezirk to the AGS of that Regierungsbezirk.
    pub fn to_regierungsbezirk(&self) -> Option<Ags> {
        if let Some(regierungsbezirk) = self.regierungsbezirk {
            Some(Ags::new_regierungsbezirk(self.land, regierungsbezirk))
        } else {
            None
        }
    }

    /// Converts a AGS that is at least specific to a Kreis to the AGS of that Kreis.
    pub fn to_kreis(&self) -> Option<Ags> {
        if let (Some(regierungsbezirk), Some(kreis)) = (self.regierungsbezirk, self.kreis) {
            Some(Ags::new_kreis(self.land, regierungsbezirk, kreis))
        } else {
            None
        }
    }

    /// Returns whether this AGS identifies a Land (and thus has no further specifying information).
    pub fn is_land(&self) -> bool {
        self.regierungsbezirk.is_none()
    }

    /// Returns whether this AGS identifies a Regierungsbezirk (and thus has no further specifying information).
    pub fn is_regierungsbezirk(&self) -> bool {
        self.regierungsbezirk.is_some() && self.kreis.is_none()
    }

    /// Returns whether this AGS identifies a Kreis (and thus has no further specifying information).
    pub fn is_kreis(&self) -> bool {
        self.kreis.is_some() && self.gemeinde.is_none()
    }

    /// Returns whether this AGS identifies a Gemeinde (and thus has no further specifying information).
    pub fn is_gemeinde(&self) -> bool {
        self.gemeinde.is_some()
    }

    /// Returns whether this AGS contains the Land, Regierungsbezirk, Kreis, or Gemeinde specified by `other`. This also returns
    /// `true` if the keys identify the same unit.
    pub fn contains(&self, other: &Self) -> bool {
        if self.land != other.land {
            return false;
        }
        match (self.regierungsbezirk, other.regierungsbezirk) {
            (Some(a), Some(b)) if a != b => return false,
            (Some(_), None) => return false,
            _ => {}
        }
        match (self.kreis, other.kreis) {
            (Some(a), Some(b)) if a != b => return false,
            (Some(_), None) => return false,
            _ => {}
        }
        match (self.gemeinde, other.gemeinde) {
            (Some(a), Some(b)) if a != b => return false,
            (Some(_), None) => return false,
            _ => {}
        }
        true
    }
}

impl Display for Ags {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:02}", self.land)?;
        if let Some(regierungsbezirk) = self.regierungsbezirk {
            write!(f, "{:01}", regierungsbezirk)?;
        }
        if let Some(kreis) = self.kreis {
            write!(f, "{:02}", kreis)?;
        }
        if let Some(gemeinde) = self.gemeinde {
            write!(f, "{:03}", gemeinde)?;
        }
        Ok(())
    }
}

/// Information regarding juristical districts
#[derive(Clone, Debug)]
pub struct Gerichtbarkeit {
    oberlandesgericht: u8,
    landgericht: u8,
    amtsgericht: u8,
}

/// A (in-memory) database that stores GV100 data for querying.
pub struct Database {
    /// Laender
    laender: BTreeMap<Ags, Land>,

    /// Regierunzbezirke
    regierungsbezirke: BTreeMap<Ags, Regierungsbezirk>,

    /// Regionen (only Baden-Wuerttenberg)
    regionen: BTreeMap<u8, Region>,

    /// Kreise
    kreise: BTreeMap<Ags, Kreis>,

    /// Gemeindeverbaende
    gemeindeverbaende: BTreeMap<Ags, BTreeMap<u16, Gemeindeverband>>,

    /// Gemeinden
    gemeinden: BTreeMap<Ags, Gemeinde>,
}

impl Database {
    /// Create database from a buffered reader.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, Error> {
        Self::from_parser(Parser::new(reader))
    }

    /// Create database from GV100 file at `path`.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_parser(Parser::from_path(path)?)
    }

    /// Create database from GV100 parser.
    pub fn from_parser<R: BufRead>(mut parser: Parser<R>) -> Result<Self, Error> {
        let mut laender = BTreeMap::new();
        let mut regierungsbezirke = BTreeMap::new();
        let mut kreise = BTreeMap::new();
        let mut gemeinden = BTreeMap::new();
        let mut regionen = BTreeMap::new();
        let mut gemeindeverbaende: BTreeMap<Ags, BTreeMap<u16, Gemeindeverband>> = BTreeMap::new();

        while let Some(record) = parser.parse_line()? {
            match record {
                Record::Land(land) => {
                    laender.insert(land.ags.clone(), land);
                }
                Record::Regierungsbezirk(regierungsbezirk) => {
                    regierungsbezirke.insert(regierungsbezirk.ags.clone(), regierungsbezirk);
                }
                Record::Kreis(kreis) => {
                    kreise.insert(kreis.ags.clone(), kreis);
                }
                Record::Gemeinde(gemeinde) => {
                    gemeinden.insert(gemeinde.ags.clone(), gemeinde);
                }
                Record::Region(region) => {
                    regionen.insert(region.region, region);
                }
                Record::Gemeindeverband(gemeindeverband) => {
                    gemeindeverbaende
                        .entry(gemeindeverband.ags.clone())
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

    /// Returns Land by AGS.
    pub fn get_land(&self, ags: &Ags) -> Option<&Land> {
        self.laender.get(ags)
    }

    /// Returns Regierungsbezirk by AGS.
    pub fn get_regierungsbezirk(&self, ags: &Ags) -> Option<&Regierungsbezirk> {
        self.regierungsbezirke.get(ags)
    }

    /// Returns a Region record (only Baden-Wuerttenberg)
    pub fn get_region(&self, region: u8) -> Option<&Region> {
        self.regionen.get(&region)
    }

    /// Returns Kreis by AGS.
    pub fn get_kreis(&self, ags: &Ags) -> Option<&Kreis> {
        self.kreise.get(ags)
    }

    /// Returns a Gemeindeverband by AGS (Land, Regierungsbezirk and Land) and Gemeinderverband key.
    pub fn get_gemeindeverband(&self, ags: &Ags, gemeindeverband: u16) -> Option<&Gemeindeverband> {
        self.gemeindeverbaende.get(ags)?.get(&gemeindeverband)
    }

    /// Returns Gemeinde by AGS.
    pub fn get_gemeinde(&self, ags: &Ags) -> Option<&Gemeinde> {
        self.gemeinden.get(ags)
    }

    /// Returns an iterator over all Laender.
    pub fn iter_laender(&self) -> impl Iterator<Item = &Land> {
        self.laender.values()
    }

    /// Returns an iterator over Kreise contained in Land or Regierungsbezirk `ags`.
    pub fn iter_kreise_in(&self, ags: &Ags) -> impl Iterator<Item = &Kreis> {
        let mut first = ags.clone();
        let mut last = ags.clone();

        if ags.regierungsbezirk.is_none() {
            first.regierungsbezirk = Some(u8::MIN);
            last.regierungsbezirk = Some(u8::MAX);
        }
        if ags.kreis.is_none() {
            first.kreis = Some(u8::MIN);
            last.kreis = Some(u8::MAX);
        }

        self.kreise.range(first..=last).map(|(_, kreis)| kreis)
    }

    /// Returns an iterator over Kreise contained in Land, Regierungsbezirk, or Gemeinde `ags`.
    pub fn iter_gemeinden_in(&self, ags: &Ags) -> impl Iterator<Item = &Gemeinde> {
        let mut first = ags.clone();
        let mut last = ags.clone();

        if ags.regierungsbezirk.is_none() {
            first.regierungsbezirk = Some(u8::MIN);
            last.regierungsbezirk = Some(u8::MAX);
        }
        if ags.kreis.is_none() {
            first.kreis = Some(u8::MIN);
            last.kreis = Some(u8::MAX);
        }
        if ags.gemeinde.is_none() {
            first.gemeinde = Some(u16::MIN);
            last.gemeinde = Some(u16::MAX);
        }

        self.gemeinden.range(first..=last).map(|(_, gemeinde)| gemeinde)
    }
}

trait Visitor {
    fn begin_land(&mut self, _land: &Land) -> bool {
        false
    }

    fn end_land(&mut self, _land: &Land) {}

    fn begin_regierunsbezirk(&mut self, _regierungsbezirk: &Regierungsbezirk) -> bool {
        false
    }

    fn end_regierungsbezirk(&mut self, _regierungsbezirk: &Regierungsbezirk) {}

    fn begin_kreis(&mut self, _kreis: &Kreis) -> bool {
        false
    }

    fn end_kreis(&mut self, _kreis: &Kreis) {}

    fn begin_gemeindeverband(&mut self, _gemeindeverband: &Gemeindeverband) -> bool {
        false
    }

    fn end_gemeindeverband(&mut self, _gemeindeverband: &Gemeindeverband) {}

    fn gemeinde(&mut self, _gemeinde: &Gemeinde) {}

    /// TODO: Doesn't currently visit Regierungsbezirke, Regionen, or Gemeindeverbaende
    fn visit(&mut self, db: &Database) {
        for land in db.iter_laender() {
            if self.begin_land(land) {
                for kreis in db.iter_kreise_in(&land.ags) {
                    if self.begin_kreis(kreis) {
                        for gemeinde in db.iter_gemeinden_in(&land.ags) {
                            self.gemeinde(gemeinde);
                        }
                        self.end_kreis(kreis);
                    }
                }
                self.end_land(land);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// These tests need the file `GV100AD_300421.txt` that can be downloaded from [1]
    ///
    /// [1] https://www.destatis.de/DE/Themen/Laender-Regionen/Regionales/Gemeindeverzeichnis/_inhalt.html
    ///
    /// # TODO
    ///
    /// * Write more tests
    ///
    use super::*;

    #[test]
    pub fn parse_full_dataset() {
        let mut parser = Parser::from_path("GV100AD_300421.txt").unwrap();

        while let Some(record) = parser.parse_line().unwrap() {
            println!("{:#?}", record);
        }
    }

    #[test]
    pub fn iter_gebiete() {
        let db = Database::from_file("GV100AD_300421.txt").unwrap();

        let kreise_saarland = db.iter_kreise_in(&Ags::new_land(10)).collect::<Vec<&Kreis>>();

        assert_eq!(kreise_saarland.len(), 6);
        assert_eq!(kreise_saarland[0].name, "Regionalverband Saarbrücken");
        assert_eq!(kreise_saarland[1].name, "Merzig-Wadern");
        assert_eq!(kreise_saarland[2].name, "Neunkirchen");
        assert_eq!(kreise_saarland[3].name, "Saarlouis");
        assert_eq!(kreise_saarland[4].name, "Saarpfalz-Kreis");
        assert_eq!(kreise_saarland[5].name, "St. Wendel");
    }
}
