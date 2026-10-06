//! One of the platform's CSV files of Urban QOL data, read into memory.
//!
//! A file has one row for each mesh and indicator. The row whose
//! `IndicatorCode` is [`TOTAL_CODE`] holds the mesh's published total, and
//! every other row one indicator's value for it.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use csv::StringRecord;
use thiserror::Error;

use crate::mesh::{HalfMesh, MeshCodeError};

/// The `IndicatorCode` of the rows that hold the published total.
const TOTAL_CODE: &str = "QOL";

/// A file's meshes, and what it publishes for each of them.
#[derive(Debug, PartialEq)]
pub(crate) struct Dataset {
    meshes: Vec<Mesh>,
    series: Vec<Series>,
}

/// A mesh of the file.
#[derive(Debug, PartialEq)]
pub(crate) struct Mesh {
    pub(crate) code: String,
    pub(crate) square: HalfMesh,
    /// The name of the municipality the file puts the mesh in.
    pub(crate) city: String,
}

/// What the file publishes under one `IndicatorCode`: the total, or an
/// indicator.
#[derive(Debug, PartialEq)]
pub(crate) struct Series {
    pub(crate) code: String,
    pub(crate) name: String,
    /// One value for each mesh, in the order of [`Dataset::meshes`].
    pub(crate) values: Vec<f64>,
}

impl Dataset {
    /// Reads the file at `path`.
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self, DatasetError> {
        Self::read(File::open(path)?)
    }

    /// Reads a file's contents. A byte order mark at their start is skipped,
    /// which the CSV reader does of itself.
    pub(crate) fn read(contents: impl Read) -> Result<Self, DatasetError> {
        let mut reader = csv::Reader::from_reader(contents);
        let columns = Columns::find(reader.headers()?)?;
        let mut builder = Builder::default();
        for record in reader.records() {
            builder.add(&columns.row(&record?)?)?;
        }
        builder.finish()
    }

    /// The file's meshes, in the order it first names them.
    pub(crate) fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    /// The published total, then each indicator of the file.
    pub(crate) fn series(&self) -> &[Series] {
        &self.series
    }
}

/// Where the columns the tool reads are in a file.
struct Columns {
    key_code: usize,
    city: usize,
    indicator_code: usize,
    indicator: usize,
    value: usize,
}

impl Columns {
    fn find(headers: &StringRecord) -> Result<Self, DatasetError> {
        let column = |name: &'static str| {
            let mut named = headers
                .iter()
                .enumerate()
                .filter(|(_, header)| *header == name)
                .map(|(column, _)| column);
            match (named.next(), named.next()) {
                (Some(column), None) => Ok(column),
                (None, _) => Err(DatasetError::NoColumn { name }),
                (Some(_), Some(_)) => Err(DatasetError::ColumnTwice { name }),
            }
        };
        Ok(Self {
            key_code: column("KeyCode")?,
            city: column("City")?,
            indicator_code: column("IndicatorCode")?,
            indicator: column("Indicator")?,
            value: column("Value")?,
        })
    }
}

/// What one row of a file says.
struct Row<'r> {
    /// The line of the file the row starts on, for a refusal to name.
    line: u64,
    key_code: &'r str,
    city: &'r str,
    indicator_code: &'r str,
    indicator: &'r str,
    value: f64,
}

impl Columns {
    /// Takes a row out of a record of the file.
    fn row<'r>(&self, record: &'r StringRecord) -> Result<Row<'r>, DatasetError> {
        let line = record.position().map_or(0, csv::Position::line);
        let field = |column: usize| record.get(column).unwrap_or_default();
        // A name the tool shows, or lists a choice under, is never blank.
        let named = |column: usize, name: &'static str| match field(column) {
            text if text.trim().is_empty() => Err(DatasetError::Blank { line, column: name }),
            text => Ok(text),
        };
        Ok(Row {
            line,
            key_code: field(self.key_code),
            city: named(self.city, "City")?,
            indicator_code: named(self.indicator_code, "IndicatorCode")?,
            indicator: named(self.indicator, "Indicator")?,
            value: field(self.value)
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or(DatasetError::Value { line })?,
        })
    }
}

/// A file while its rows are being read.
#[derive(Default)]
struct Builder {
    meshes: Vec<Mesh>,
    /// Where each mesh code is in `meshes`.
    mesh_at: HashMap<String, usize>,
    series: Vec<UnfinishedSeries>,
    /// Where each `IndicatorCode` is in `series`.
    series_at: HashMap<String, usize>,
}

/// A series some of whose rows may not have been read yet.
struct UnfinishedSeries {
    code: String,
    name: String,
    /// One place for each mesh read so far, `None` until its row is read.
    values: Vec<Option<f64>>,
}

impl Builder {
    /// Takes in a row, which may be the first of its mesh or of its indicator.
    fn add(&mut self, row: &Row<'_>) -> Result<(), DatasetError> {
        let mesh = self.mesh(row)?;
        let series = self.series(row)?;
        let values = &mut self.series[series].values;
        if values.len() <= mesh {
            values.resize(mesh + 1, None);
        }
        if values[mesh].replace(row.value).is_some() {
            return Err(DatasetError::Repeated { line: row.line });
        }
        Ok(())
    }

    /// Where the row's mesh is in `meshes`, once it is known to be there.
    fn mesh(&mut self, row: &Row<'_>) -> Result<usize, DatasetError> {
        let line = row.line;
        match self.mesh_at.get(row.key_code) {
            Some(&at) if self.meshes[at].city == row.city => Ok(at),
            Some(_) => Err(DatasetError::Disagrees {
                line,
                column: "City",
            }),
            None => {
                let square = HalfMesh::from_code(row.key_code)
                    .map_err(|source| DatasetError::MeshCode { line, source })?;
                self.meshes.push(Mesh {
                    code: row.key_code.to_owned(),
                    square,
                    city: row.city.to_owned(),
                });
                let at = self.meshes.len() - 1;
                self.mesh_at.insert(row.key_code.to_owned(), at);
                Ok(at)
            }
        }
    }

    /// Where the row's indicator is in `series`, once it is known to be there.
    fn series(&mut self, row: &Row<'_>) -> Result<usize, DatasetError> {
        match self.series_at.get(row.indicator_code) {
            Some(&at) if self.series[at].name == row.indicator => Ok(at),
            Some(_) => Err(DatasetError::Disagrees {
                line: row.line,
                column: "Indicator",
            }),
            None => {
                self.series.push(UnfinishedSeries {
                    code: row.indicator_code.to_owned(),
                    name: row.indicator.to_owned(),
                    values: Vec::new(),
                });
                let at = self.series.len() - 1;
                self.series_at.insert(row.indicator_code.to_owned(), at);
                Ok(at)
            }
        }
    }

    /// The file, with the total first, if every mesh has every series.
    fn finish(mut self) -> Result<Dataset, DatasetError> {
        let total = *self
            .series_at
            .get(TOTAL_CODE)
            .ok_or(DatasetError::NoTotal)?;
        let total = self.series.remove(total);
        self.series.insert(0, total);
        let mesh_count = self.meshes.len();
        let series = self
            .series
            .into_iter()
            .map(
                |UnfinishedSeries {
                     code,
                     name,
                     mut values,
                 }| {
                    values.resize(mesh_count, None);
                    match values.into_iter().collect::<Option<Vec<f64>>>() {
                        Some(values) => Ok(Series { code, name, values }),
                        None => Err(DatasetError::Incomplete { indicator: code }),
                    }
                },
            )
            .collect::<Result<_, _>>()?;
        Ok(Dataset {
            meshes: self.meshes,
            series,
        })
    }
}

/// Why a file could not be read as Urban QOL data.
#[derive(Debug, Error)]
pub(crate) enum DatasetError {
    #[error("it could not be opened or read: {0}")]
    Io(#[from] std::io::Error),
    #[error("it is not laid out as CSV: {0}")]
    Csv(csv::Error),
    #[error("it has no `{name}` column")]
    NoColumn { name: &'static str },
    #[error("it has more than one `{name}` column")]
    ColumnTwice { name: &'static str },
    #[error("line {line}: its `{column}` is not what an earlier row of its mesh or indicator has")]
    Disagrees { line: u64, column: &'static str },
    #[error("line {line}: its `{column}` is blank")]
    Blank { line: u64, column: &'static str },
    #[error("line {line}: its `KeyCode` is not a 500 m mesh: {source}")]
    MeshCode { line: u64, source: MeshCodeError },
    #[error("line {line}: its `Value` is not a number")]
    Value { line: u64 },
    #[error("line {line}: its mesh already has a value for its indicator")]
    Repeated { line: u64 },
    #[error("it has no row whose `IndicatorCode` is `QOL`")]
    NoTotal,
    #[error("a mesh has no row whose `IndicatorCode` is `{indicator}`")]
    Incomplete { indicator: String },
}

/// The CSV reader hands a failure to read the file on as one of its own
/// errors, which is told apart here from a file that is not CSV.
impl From<csv::Error> for DatasetError {
    fn from(error: csv::Error) -> Self {
        if !error.is_io_error() {
            return Self::Csv(error);
        }
        match error.into_kind() {
            csv::ErrorKind::Io(error) => Self::Io(error),
            _ => unreachable!("an I/O error is of the I/O kind"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use proptest::collection::{btree_map, vec};
    use proptest::prelude::*;

    use super::*;
    use crate::test_support::valid_code;

    const HEADER: [&str; 8] = [
        "KeyCode",
        "PrefectureCode",
        "CityCode",
        "Prefecture",
        "City",
        "IndicatorCode",
        "Indicator",
        "Value",
    ];

    /// What a test puts in a row; the other columns are filled with text the
    /// tool does not read.
    #[derive(Clone, Debug)]
    struct Row {
        key_code: String,
        city: String,
        indicator_code: String,
        indicator: String,
        value: String,
    }

    fn row(key_code: &str, city: &str, indicator_code: &str, indicator: &str, value: &str) -> Row {
        Row {
            key_code: key_code.to_owned(),
            city: city.to_owned(),
            indicator_code: indicator_code.to_owned(),
            indicator: indicator.to_owned(),
            value: value.to_owned(),
        }
    }

    /// A file in the platform's layout, made of `rows`.
    fn file(rows: &[Row]) -> Vec<u8> {
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.write_record(HEADER).unwrap();
        for r in rows {
            writer
                .write_record([
                    r.key_code.as_str(),
                    "00",
                    "00000",
                    "a prefecture",
                    r.city.as_str(),
                    r.indicator_code.as_str(),
                    r.indicator.as_str(),
                    r.value.as_str(),
                ])
                .unwrap();
        }
        writer.into_inner().unwrap()
    }

    fn with_byte_order_mark(mut contents: Vec<u8>) -> Vec<u8> {
        contents.splice(0..0, [0xEF, 0xBB, 0xBF]);
        contents
    }

    /// Two meshes and two indicators, the totals coming last.
    fn small() -> Vec<Row> {
        vec![
            row("543823431", "East", "A01", "Stations", "1.5"),
            row("543823432", "West", "A01", "Stations", "-2"),
            row("543823431", "East", "B02", "Floods", "0"),
            row("543823432", "West", "B02", "Floods", "0.25"),
            row("543823431", "East", "QOL", "Total", "1.5"),
            row("543823432", "West", "QOL", "Total", "-1.75"),
        ]
    }

    fn read(rows: &[Row]) -> Result<Dataset, DatasetError> {
        Dataset::read(file(rows).as_slice())
    }

    #[test]
    fn a_file_becomes_its_meshes_and_the_total_then_its_indicators() {
        let dataset = read(&small()).unwrap();

        let meshes: Vec<_> = dataset
            .meshes()
            .iter()
            .map(|m| (m.code.as_str(), m.city.as_str(), m.square))
            .collect();
        let square = |code| HalfMesh::from_code(code).unwrap();
        assert_eq!(
            meshes,
            [
                ("543823431", "East", square("543823431")),
                ("543823432", "West", square("543823432")),
            ]
        );

        let series: Vec<_> = dataset
            .series()
            .iter()
            .map(|s| (s.code.as_str(), s.name.as_str(), s.values.as_slice()))
            .collect();
        assert_eq!(
            series,
            [
                ("QOL", "Total", [1.5, -1.75].as_slice()),
                ("A01", "Stations", [1.5, -2.0].as_slice()),
                ("B02", "Floods", [0.0, 0.25].as_slice()),
            ]
        );
    }

    #[test]
    fn a_byte_order_mark_changes_nothing() {
        let plain = file(&small());
        let marked = with_byte_order_mark(plain.clone());
        assert_eq!(
            Dataset::read(marked.as_slice()).unwrap(),
            Dataset::read(plain.as_slice()).unwrap()
        );
    }

    #[test]
    fn columns_are_found_by_name_wherever_they_are() {
        let contents = "Value,Extra,Indicator,IndicatorCode,City,KeyCode\n\
                        1.5,x,Total,QOL,East,543823431\n";
        let dataset = Dataset::read(contents.as_bytes()).unwrap();
        assert_eq!(dataset.meshes()[0].code, "543823431");
        assert_eq!(dataset.series()[0].values, [1.5]);
    }

    #[test]
    fn a_file_without_a_column_the_tool_reads_is_refused() {
        let contents = "KeyCode,City,IndicatorCode,Indicator\n543823431,East,QOL,Total\n";
        assert!(matches!(
            Dataset::read(contents.as_bytes()),
            Err(DatasetError::NoColumn { name: "Value" })
        ));
    }

    #[test]
    fn a_file_with_a_column_twice_is_refused() {
        let contents = "KeyCode,City,IndicatorCode,Indicator,Value,Value\n\
                        543823431,East,QOL,Total,1.5,2.5\n";
        assert!(matches!(
            Dataset::read(contents.as_bytes()),
            Err(DatasetError::ColumnTwice { name: "Value" })
        ));
    }

    #[test]
    fn a_failure_to_read_is_not_taken_for_a_file_that_is_not_csv() {
        struct Unreadable;
        impl Read for Unreadable {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("the disk is gone"))
            }
        }
        assert!(matches!(
            Dataset::read(Unreadable),
            Err(DatasetError::Io(_))
        ));
        // A row shorter than the header is the reader's own complaint.
        let contents = "KeyCode,City,IndicatorCode,Indicator,Value\n543823431,East\n";
        assert!(matches!(
            Dataset::read(contents.as_bytes()),
            Err(DatasetError::Csv(_))
        ));
    }

    #[test]
    fn rows_that_disagree_on_a_name_are_refused() {
        let mut rows = small();
        rows[4].city = "North".to_owned();
        assert!(matches!(
            read(&rows),
            Err(DatasetError::Disagrees {
                line: 6,
                column: "City"
            })
        ));

        let mut rows = small();
        rows[3].indicator = "Storms".to_owned();
        assert!(matches!(
            read(&rows),
            Err(DatasetError::Disagrees {
                line: 5,
                column: "Indicator"
            })
        ));
    }

    #[test]
    fn a_row_whose_mesh_code_is_not_a_half_grid_square_is_refused() {
        let mut rows = small();
        // The nine-digit code of a 2 km square.
        rows[1].key_code = "543823435".to_owned();
        assert!(matches!(
            read(&rows),
            Err(DatasetError::MeshCode {
                line: 3,
                source: MeshCodeError::NoSuchQuarter
            })
        ));
    }

    #[test]
    fn a_row_whose_value_is_not_a_finite_number_is_refused() {
        for value in ["", "high", "NaN", "inf", " 1", "1 "] {
            let mut rows = small();
            rows[2].value = value.to_owned();
            assert!(
                matches!(read(&rows), Err(DatasetError::Value { line: 4 })),
                "{value:?}"
            );
        }
    }

    #[test]
    fn a_second_row_for_a_mesh_and_indicator_is_refused() {
        let mut rows = small();
        rows.push(row("543823431", "East", "A01", "Stations", "9"));
        assert!(matches!(
            read(&rows),
            Err(DatasetError::Repeated { line: 8 })
        ));
    }

    #[test]
    fn a_file_without_totals_is_refused() {
        let rows: Vec<Row> = small()
            .into_iter()
            .filter(|r| r.indicator_code != "QOL")
            .collect();
        assert!(matches!(read(&rows), Err(DatasetError::NoTotal)));
    }

    #[test]
    fn a_mesh_that_lacks_an_indicator_is_refused() {
        for missing in 0..small().len() {
            let mut rows = small();
            let removed = rows.remove(missing);
            assert!(
                matches!(
                    read(&rows),
                    Err(DatasetError::Incomplete { ref indicator })
                        if *indicator == removed.indicator_code
                ),
                "without row {missing}"
            );
        }
    }

    /// A file's content before it is laid out as rows: each mesh's code and
    /// municipality, each indicator's code and name, and a value for every
    /// pair of them, the total being the indicator at 0.
    #[derive(Clone, Debug)]
    struct Content {
        meshes: Vec<(String, String)>,
        indicators: Vec<(String, String)>,
        values: Vec<Vec<f64>>,
    }

    impl Content {
        fn rows(&self) -> Vec<Row> {
            let mut rows = Vec::new();
            for (i, (code, name)) in self.indicators.iter().enumerate() {
                for (m, (key_code, city)) in self.meshes.iter().enumerate() {
                    let value = self.values[i][m].to_string();
                    rows.push(row(key_code, city, code, name, &value));
                }
            }
            rows
        }
    }

    /// Up to twelve meshes and five indicators beside the total, named in
    /// Japanese scripts as the platform's files name theirs.
    fn content() -> impl Strategy<Value = Content> {
        let meshes = btree_map(valid_code(), r"\p{Katakana}{1,4}", 1..=12);
        let indicators = btree_map("[A-Z][0-9]{2}", r"[\p{Hiragana}\p{Han}]{1,6}", 0..=5);
        (meshes, indicators, r"\p{Han}{1,3}")
            .prop_flat_map(|(meshes, indicators, total)| {
                let mut named = vec![(TOTAL_CODE.to_owned(), total)];
                named.extend(indicators);
                let finite = prop::num::f64::NORMAL | prop::num::f64::ZERO;
                let values = vec(vec(finite, meshes.len()), named.len());
                (Just(meshes), Just(named), values)
            })
            .prop_map(|(meshes, indicators, values)| Content {
                meshes: meshes.into_iter().collect(),
                indicators,
                values,
            })
    }

    proptest! {
        /// Empty, or spaces of any script.
        #[test]
        fn a_row_with_a_blank_name_is_refused(blank in r"\s{0,3}", column in 0..3usize) {
            let mut rows = small();
            let name = match column {
                0 => { rows[2].city = blank; "City" }
                1 => { rows[2].indicator_code = blank; "IndicatorCode" }
                _ => { rows[2].indicator = blank; "Indicator" }
            };
            prop_assert!(
                matches!(read(&rows), Err(DatasetError::Blank { column, .. }) if column == name),
                "{}", name
            );
        }

        /// Whatever order the rows come in, and with or without a byte order
        /// mark.
        #[test]
        fn a_file_reads_back_as_what_was_written(
            (content, rows) in content().prop_flat_map(|content| {
                let rows = Just(content.rows()).prop_shuffle();
                (Just(content), rows)
            }),
            marked in any::<bool>(),
        ) {
            let contents = file(&rows);
            let contents = if marked { with_byte_order_mark(contents) } else { contents };
            let dataset = Dataset::read(contents.as_slice()).unwrap();

            let read_meshes: BTreeMap<_, _> = dataset
                .meshes()
                .iter()
                .map(|m| (m.code.clone(), m.city.clone()))
                .collect();
            let written_meshes: BTreeMap<_, _> = content.meshes.iter().cloned().collect();
            prop_assert_eq!(dataset.meshes().len(), content.meshes.len());
            prop_assert_eq!(&read_meshes, &written_meshes);

            prop_assert_eq!(&dataset.series()[0].code, TOTAL_CODE);
            prop_assert_eq!(dataset.series().len(), content.indicators.len());
            for (i, (code, name)) in content.indicators.iter().enumerate() {
                let series = dataset.series().iter().find(|s| s.code == *code).unwrap();
                prop_assert_eq!(&series.name, name);
                for (m, (key_code, _)) in content.meshes.iter().enumerate() {
                    let at = dataset.meshes().iter().position(|x| x.code == *key_code).unwrap();
                    prop_assert_eq!(series.values[at].to_bits(), content.values[i][m].to_bits());
                }
            }
        }
    }
}
