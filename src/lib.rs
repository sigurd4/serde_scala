#![feature(decl_macro)]

use std::{fmt::{Display, write}, str::FromStr, num::{ParseFloatError, ParseIntError}, string::FromUtf8Error};

use num_rational::Ratio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pitch
{
    Cents(f64),
    Ratio(Ratio<u128>)
}

impl Pitch
{
    pub fn to_cents(self) -> f64
    {
        match self
        {
            Self::Cents(cents) => cents,
            Self::Ratio(ratio) => (*ratio.numer() as f64/ *ratio.denom() as f64).log2()*1200.0
        }
    }

    pub fn to_note_offset(self) -> f64
    {
        match self
        {
            Self::Cents(cents) => cents/100.0,
            Self::Ratio(ratio) => (*ratio.numer() as f64/ *ratio.denom() as f64).log2()*12.0
        }
    }
}

impl Display for Pitch
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Cents(cents) => write!(f, "{:.5}", cents),
            Self::Ratio(ratio) => write!(f, "{}/{}", ratio.numer(), ratio.denom())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePitchError
{
    ParseFloat(ParseFloatError),
    ParseInt(ParseIntError)
}
impl From<ParseFloatError> for ParsePitchError
{
    fn from(value: ParseFloatError) -> Self
    {
        Self::ParseFloat(value)
    }
}
impl From<ParseIntError> for ParsePitchError
{
    fn from(value: ParseIntError) -> Self
    {
        Self::ParseInt(value)
    }
}

impl FromStr for Pitch
{
    type Err = ParsePitchError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let s = s.replace(" ", "");
        if s.contains(".")
        {
            let s = s.replace("cents", "");
            Ok(Self::Cents(s.parse()?))
        }
        else if s.contains("/")
        {
            let (numer, denom) = s.split_once("/").unwrap();
            Ok(Self::Ratio(Ratio::new(numer.parse()?, denom.parse()?)))
        }
        else
        {
            Ok(Self::Ratio(Ratio::new(s.parse()?, 1)))
        }
    }
}

macro add_pitch {
    (
        $pitches:expr;
        $numer:literal / $denom:literal
        $($($more:tt)+)?
    ) => {
        $pitches.push(Pitch::Ratio(Ratio::new($numer, $denom)));
        $(
            add_pitch!($pitches; $($more)+);
        )?
    },
    (
        $pitches:expr;
        $cents:literal
        $($($more:tt)+)?
    ) => {
        $pitches.push(Pitch::Cents($cents));
        $(
            add_pitch!($pitches; $($more)+);
        )?
    },
    (
        $pitches:expr;
        $numer:literal
        $($($more:tt)+)?
    ) =>
    {
        $pitches.push(Pitch::Ratio(Ratio::new($numer, 1)));
        $(
            add_pitch!($pitches; $($more)+);
        )?
    }
}

pub macro scl {
    {
        $name:literal
        $($pitches:tt)*
    } => {
        {
            let mut pitches = vec![];
            add_pitch!(&mut pitches; $($pitches)*);

            Scale::new($name.to_string(), pitches)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scale
{
    pub name: String,
    pub pitches: Vec<Pitch>
}
impl Scale
{
    pub fn new(name: String, pitches: Vec<Pitch>) -> Self
    {
        Self {
            name,
            pitches
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseScaleError
{
    ParseFloat(ParseFloatError),
    ParseInt(ParseIntError),
    MissingDescription,
    MissingNoteCount,
    WrongPitchCount(usize)
}
impl From<ParseFloatError> for ParseScaleError
{
    fn from(value: ParseFloatError) -> Self
    {
        Self::ParseFloat(value)
    }
}
impl From<ParseIntError> for ParseScaleError
{
    fn from(value: ParseIntError) -> Self
    {
        Self::ParseInt(value)
    }
}
impl From<ParsePitchError> for ParseScaleError
{
    fn from(value: ParsePitchError) -> Self
    {
        match value
        {
            ParsePitchError::ParseFloat(err) => Self::ParseFloat(err),
            ParsePitchError::ParseInt(err) => Self::ParseInt(err)
        }
    }
}
impl FromStr for Scale
{
    type Err = ParseScaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let mut name = None;
        let mut pitch_count = None;
        let mut pitches = vec![];

        for s in s.lines()
        {
            let s = s.split_once("!").map(|(s, _)| s).unwrap_or(s);
            if s == ""
            {
                continue
            }
            else if name.is_none()
            {
                name = Some(s.to_string())
            }
            else
            {
                let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
                if pitch_count.is_none()
                {
                    pitch_count = Some(s.parse()?);
                }
                else
                {
                    pitches.push(s.parse()?);
                }
            }
        }

        let name = name.ok_or(ParseScaleError::MissingDescription)?;
        let pitch_count = pitch_count.ok_or(ParseScaleError::MissingNoteCount)?;

        if pitches.len() != pitch_count
        {
            return Err(ParseScaleError::WrongPitchCount(pitches.len()))
        }

        Ok(Scale::new(name, pitches))
    }
}

impl Display for Scale
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        writeln!(f, "! Generated scale:")?;
        writeln!(f, "{}", self.name)?;
        writeln!(f, "{}", self.pitches.len())?;
        writeln!(f, "!")?;

        for pitch in self.pitches.iter()
        {
            writeln!(f, "{}", pitch)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum SerdeScalaError
{
    IO(std::io::Error),
    FromUtf8(FromUtf8Error),
    ParseScale(ParseScaleError)
}
impl From<std::io::Error> for SerdeScalaError
{
    fn from(value: std::io::Error) -> Self
    {
        Self::IO(value)
    }
}
impl From<FromUtf8Error> for SerdeScalaError
{
    fn from(value: FromUtf8Error) -> Self
    {
        Self::FromUtf8(value)
    }
}
impl From<ParseScaleError> for SerdeScalaError
{
    fn from(value: ParseScaleError) -> Self
    {
        Self::ParseScale(value)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::{BTreeSet, BTreeMap}, fs::{File, self}};

    use super::*;

    #[test]
    fn it_works() -> Result<(), SerdeScalaError>
    {
        let cd = fs::read_dir("scl").or_else(|_| {
            fs::create_dir("scl")?;
            fs::read_dir("scl")
        })?;
        for entry in cd
        {
            let entry = entry?;
            println!("! {:?}", entry.file_name());
            let bytes = fs::read(entry.path())?;
            let contents = String::from_utf8_lossy(&bytes);
            let scale: Scale = contents.parse()?;

            println!("{}", scale)
        }

        Ok(())
    }
}
