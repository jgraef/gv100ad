use std::{
    str::{FromStr, Chars},
    io::{BufRead, BufReader},
    path::Path,
    fs::File,
};

use chrono::NaiveDate;

use crate::{
    error::Error,
    model::{
        gemeinde::{GemeindeDaten, Gerichtbarkeit, Bundestagswahlkreise},
        gemeindeverband::GemeindeverbandDaten,
        kreis::KreisDaten,
        land::LandDaten,
        regierungsbezirk::RegierungsbezirkDaten,
        region::RegionDaten,
        datensatz::Datensatz,
    },
};


/// Reader to read fields from a single data record (i.e. line). Specifically
/// this makes sure that data is read correctly as UTF-8.
pub struct FieldReader<'a> {
    chars: Chars<'a>,
}

impl<'a> FieldReader<'a> {
    /// Creates a new field reader from a single line. It expects the line to
    /// not contain any line terminator.
    pub fn new(line: &'a str) -> Self {
        FieldReader {
            chars: line.chars(),
        }
    }

    /// Reads a field of length `n` as string. `n` is in characters, not bytes.
    pub fn next(&mut self, n: usize) -> &str {
        let s = self.chars.as_str();

        // Count how many bytes need to be read, to read `n` UTF-8 characters.
        let mut nb = 0;
        for _ in 0..n {
            if let Some(c) = self.chars.next() {
                nb += c.len_utf8();
            } else {
                break;
            }
        }

        &s[0..nb]
    }

    /// Reads a field of length `n` and parses it as `T`.
    pub fn parse_next<T: FromStr>(&mut self, n: usize) -> Result<T, <T as FromStr>::Err> {
        let s = self.next(n);
        
        tracing::debug!("parsing: {:?}", s);

        s.parse()
    }

    /// Skips `n` characters.
    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.chars.next();
        }
    }
}

/// Parser for GV100AD files.
pub struct Parser<R> {
    reader: R,
}

impl Parser<BufReader<File>> {
    /// Creates a new parser from a file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let reader = BufReader::new(File::open(path)?);
        Ok(Self::new(reader))
    }
}

impl<R: BufRead> Iterator for Parser<R> {
    type Item = Result<Datensatz, Error>;

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
    /// Returns `Ok(None)` if end of file is reached. Returns `Err(_)`, if an
    /// error occured, otherwise returns `Ok(Some(_))`, if a record was
    /// successfully read.
    pub fn parse_line(&mut self) -> Result<Option<Datensatz>, Error> {
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

                let schluessel = fields.parse_next(2)?;

                fields.skip(10);

                let name = fields.next(50).trim().to_owned();

                let sitz_regierung = fields.next(50).trim().to_owned();

                Datensatz::Land(LandDaten {
                    gebietsstand,
                    schluessel,
                    name,
                    sitz_regierung,
                })
            }
            20 => {
                // Regierungsbezirkdaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let schluessel = fields.parse_next(3)?;

                fields.skip(9);

                let name = fields.next(50).trim().to_owned();

                let sitz_verwaltung = fields.next(50).trim().to_owned();

                Datensatz::Regierungsbezirk(RegierungsbezirkDaten {
                    gebietsstand,
                    schluessel,
                    name,
                    sitz_verwaltung,
                })
            }
            30 => {
                // Regionsdaten (nur Baden-Wuerttenberg)

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let schluessel = fields.parse_next(4)?;
                tracing::debug!(schluessel = ?schluessel);

                let name = fields.next(50).trim().to_owned();
                tracing::debug!(name = ?name);

                let sitz_verwaltung = fields.next(50).trim().to_owned();
                tracing::debug!(sitz_verwaltung = ?sitz_verwaltung);

                Datensatz::Region(RegionDaten {
                    gebietsstand,
                    schluessel,
                    name,
                    sitz_verwaltung,
                })
            }
            40 => {
                // Kreisdaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let schluessel = fields.parse_next(5)?;
                tracing::debug!(schluessel = ?schluessel);

                fields.skip(7);

                let name = fields.next(50).trim().to_owned();
                tracing::debug!(name = ?name);

                let sitz_verwaltung = fields.next(50).trim().to_owned();
                tracing::debug!(sitz_verwaltung = ?sitz_verwaltung);

                //let subtype = fields.parse_next(2)?;
                //tracing::debug!(subtype = ?subtype);
                //fields.skip(2);

                Datensatz::Kreis(KreisDaten {
                    gebietsstand,
                    schluessel,
                    name,
                    sitz_verwaltung,
                })
            }
            50 => {
                // Gemeindeverbandsdaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let kreis_schluessel = fields.parse_next(5)?;
                tracing::debug!(kreis_schluessel = ?kreis_schluessel);

                fields.skip(3);

                let gemeindeverband = fields.parse_next(4)?;
                tracing::debug!(gemeindeverband = ?gemeindeverband);

                let name = fields.next(50).trim().to_owned();
                tracing::debug!(name = ?name);

                let sitz_verwaltung = fields.next(50).trim().to_owned();
                tracing::debug!(sitz_verwaltung = ?sitz_verwaltung);

                //let subtype = fields.parse_next(2)?;
                //tracing::debug!(subtype = ?subtype);

                Datensatz::Gemeindeverband(GemeindeverbandDaten {
                    gebietsstand,
                    kreis_schluessel,
                    gemeindeverband,
                    name,
                    sitz_verwaltung,
                })
            }
            60 => {
                // Gemeindedaten

                let gebietsstand = parse_date(fields.next(8))?;
                tracing::debug!(gebietsstand = ?gebietsstand);

                let schluessel = fields.parse_next(8)?;
                tracing::debug!(schluessel = ?schluessel);

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

                let plz_unambiguous = fields.next(5).chars().all(|c| c == ' ');
                tracing::debug!(plz_unambiguous = ?plz_unambiguous);

                fields.skip(2);

                let finanzamtbezirk = fields.parse_next(4)?;
                tracing::debug!(finanzamtbezirk = ?finanzamtbezirk);

                let gerichtbarkeit = Gerichtbarkeit {
                    oberlandesgericht: fields.parse_next(1)?,
                    landgericht: fields.parse_next(1)?,
                    amtsgericht: fields.parse_next(2)?,
                };
                tracing::debug!(gerichtbarkeit = ?gerichtbarkeit);

                let arbeitsargenturbezirk = fields.parse_next(5)?;
                tracing::debug!(arbeitsargenturbezirk = ?arbeitsargenturbezirk);

                let bundestagswahlkreise = {
                    let von = fields.parse_next(3)?;

                    let s = fields.next(3);
                    if s.chars().all(|c| c == ' ') {
                        Bundestagswahlkreise::Single(von)
                    }
                    else {
                        Bundestagswahlkreise::Range(von, s.parse()?)
                    }
                };
                tracing::debug!(bundestagswahlkreise = ?bundestagswahlkreise);
                //fields.skip(4);
                //fields.skip(20);

                Datensatz::Gemeinde(GemeindeDaten {
                    gebietsstand,
                    schluessel,
                    gemeindeverband,
                    name,
                    area,
                    population_total,
                    population_male,
                    plz,
                    plz_unambiguous,
                    finanzamtbezirk,
                    gerichtbarkeit,
                    arbeitsargenturbezirk,
                    bundestagswahlkreise,
                })
            }
            ty => return Err(Error::InvalidType(ty)),
        };

        tracing::debug!("{:#?}", record);

        Ok(Some(record))
    }
}

/// Parses date from a field. This is just year, month, day without any
/// seperators. German timezones apply.
pub fn parse_date(s: &str) -> Result<NaiveDate, Error> {
    Ok(NaiveDate::from_ymd(
        s[0..4].parse()?,
        s[4..6].parse()?,
        s[6..8].parse()?,
    ))
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::model::{
        land::LandSchluessel,
        kreis::KreisSchluessel,
        gemeinde::GemeindeSchluessel,
        datensatz::Datensatz,
    };

    use super::*;

    fn parse_single_line(line: &str) -> Datensatz {
        let mut parser = Parser::new(Cursor::new(line));
        let record = parser.parse_line().unwrap().unwrap();
        assert!(parser.parse_line().unwrap().is_none());
        record
    }

    #[test]
    fn it_parses_landdaten() {
        let l = "102021043010          Saarland                                          Saarbrücken, Landeshauptstadt                                                                                                                       ";
        let record = parse_single_line(l);

        match record {
            Datensatz::Land(land) => {
                assert_eq!(land.gebietsstand, NaiveDate::from_ymd(2021, 04, 30));
                assert_eq!(land.schluessel, LandSchluessel::new(10));
                assert_eq!(land.name, "Saarland");
                assert_eq!(land.sitz_regierung, "Saarbrücken, Landeshauptstadt");
            },
            _ => panic!("Incorrect record type"),
        }
    }

    #[test]
    fn it_parses_kreisdaten() {
        let l = "402021043010041       Regionalverband Saarbrücken                       Saarbrücken, Landeshauptstadt                     45                                                                                                ";
        let record = parse_single_line(l);

        match record {
            Datensatz::Kreis(kreis) => {
                assert_eq!(kreis.gebietsstand, NaiveDate::from_ymd(2021, 04, 30));
                assert_eq!(kreis.schluessel, KreisSchluessel::new_land(LandSchluessel::new(10), 41));
                assert_eq!(kreis.name, "Regionalverband Saarbrücken");
                assert_eq!(kreis.sitz_verwaltung, "Saarbrücken, Landeshauptstadt");
            },
            _ => panic!("Incorrect record type"),
        }
    }

    #[test]
    fn it_parses_gemeindeverbanddaten() {
        let l = "502021043010041   0100Saarbrücken, Landeshauptstadt                                                                       50                                                                                                ";
        let record = parse_single_line(l);

        match record {
            Datensatz::Gemeindeverband(gemeindeverband) => {
                assert_eq!(gemeindeverband.gebietsstand, NaiveDate::from_ymd(2021, 04, 30));
                assert_eq!(gemeindeverband.kreis_schluessel, KreisSchluessel::new_land(LandSchluessel::new(10), 41));
                assert_eq!(gemeindeverband.gemeindeverband, 100);
                assert_eq!(gemeindeverband.name, "Saarbrücken, Landeshauptstadt");
                assert_eq!(gemeindeverband.sitz_verwaltung, "");
            },
            _ => panic!("Incorrect record type"),
        }
    }

    #[test]
    fn it_parses_gemeindedaten() {
        let l = "6020210430100411000100Saarbrücken, Landeshauptstadt                                                                       63    000000167520000018037400000089528    66111*****  1040110955501296                           ";
        let record = parse_single_line(l);

        match record {
            Datensatz::Gemeinde(gemeinde) => {
                assert_eq!(gemeinde.gebietsstand, NaiveDate::from_ymd(2021, 04, 30));
                assert_eq!(gemeinde.schluessel, GemeindeSchluessel::new(KreisSchluessel::new_land(LandSchluessel::new(10), 41), 100));
                assert_eq!(gemeinde.gemeindeverband, 100);
                assert_eq!(gemeinde.name, "Saarbrücken, Landeshauptstadt");
                assert_eq!(gemeinde.area, 16752);
                assert_eq!(gemeinde.population_total, 180374);
                assert_eq!(gemeinde.population_male, 89528);
                assert_eq!(gemeinde.plz, "66111");
                assert_eq!(gemeinde.plz_unambiguous, false);
                assert_eq!(gemeinde.finanzamtbezirk, 1040);
                assert_eq!(gemeinde.gerichtbarkeit.oberlandesgericht, 1);
                assert_eq!(gemeinde.gerichtbarkeit.landgericht, 1);
                assert_eq!(gemeinde.gerichtbarkeit.amtsgericht, 9);
                assert_eq!(gemeinde.arbeitsargenturbezirk, 55501);
                match gemeinde.bundestagswahlkreise {
                    Bundestagswahlkreise::Single(n) => assert_eq!(n, 296),
                    _ => panic!("Expected there to be a single Bundestagswahlkreis"),
                }
            },
            _ => panic!("Incorrect record type"),
        }
    }
}