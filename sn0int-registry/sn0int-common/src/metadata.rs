use errors::*;

use std::str::FromStr;


#[derive(Debug, PartialEq)]
pub enum EntryType {
    Description,
    Version,
    Source,
    License,
}

impl FromStr for EntryType {
    type Err = Error;

    fn from_str(s: &str) -> Result<EntryType> {
        match s {
            "Description" => Ok(EntryType::Description),
            "Version" => Ok(EntryType::Version),
            "Source" => Ok(EntryType::Source),
            "License" => Ok(EntryType::License),
            x => bail!("Unknown EntryType: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Source {
    Domains,
    Subdomains,
}

impl FromStr for Source {
    type Err = Error;

    fn from_str(s: &str) -> Result<Source> {
        match s {
            "domains" => Ok(Source::Domains),
            "subdomains" => Ok(Source::Subdomains),
            x => bail!("Unknown Source: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum License {
    MIT,
    GPL3,
    LGPL3,
    BSD2,
    BSD3,
    WTFPL,
}

impl FromStr for License {
    type Err = Error;

    fn from_str(s: &str) -> Result<License> {
        match s {
            "MIT" => Ok(License::MIT),
            "GPL-3.0" => Ok(License::GPL3),
            "LGPL-3.0" => Ok(License::LGPL3),
            "BSD-2-Clause" => Ok(License::BSD2),
            "BSD-3-Clause" => Ok(License::BSD3),
            "WTFPL" => Ok(License::WTFPL),
            x => bail!("Unsupported license: {:?}", x),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Metadata {
    pub description: String,
    pub version: String,
    pub source: Option<Source>,
    pub license: License,
}

impl FromStr for Metadata {
    type Err = Error;

    fn from_str(code: &str) -> Result<Metadata> {
        let (_, lines) = metalines(code)
            .map_err(|_| format_err!("Failed to parse header"))?;

        let mut data = NewMetadata::default();

        for (k, v) in lines {
            match k {
                EntryType::Description => data.description = Some(v),
                EntryType::Version => data.version = Some(v),
                EntryType::Source => data.source = Some(v),
                EntryType::License => data.license = Some(v),
            }
        }

        data.try_from()
    }
}

#[derive(Default)]
pub struct NewMetadata<'a> {
    pub description: Option<&'a str>,
    pub version: Option<&'a str>,
    pub source: Option<&'a str>,
    pub license: Option<&'a str>,
}

impl<'a> NewMetadata<'a> {
    fn try_from(self) -> Result<Metadata> {
        let description = self.description.ok_or_else(|| format_err!("Description is required"))?;
        let version = self.version.ok_or_else(|| format_err!("Version is required"))?;
        let source = match self.source {
            Some(x) => Some(x.parse()?),
            _ => None,
        };
        let license = self.license.ok_or_else(|| format_err!("License is required"))?;
        let license = license.parse()?;

        Ok(Metadata {
            description: description.to_string(),
            version: version.to_string(),
            source,
            license,
        })
    }
}

named!(metaline<&str, (EntryType, &str)>, do_parse!(
    tag!("-- ") >>
    name: map_res!(take_until!(": "), EntryType::from_str) >>
    tag!(": ") >>
    value: take_until!("\n") >>
    tag!("\n") >>
    (
        (name, value)
    )
));

named!(metalines<&str, Vec<(EntryType, &str)>>, do_parse!(
    lines: fold_many0!(metaline, Vec::new(), |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
    }) >>
    tag!("\n") >>
    (lines)
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_simple() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Source: domains
-- License: WTFPL

"#).expect("parse");
        assert_eq!(metadata, Metadata {
            description: "Hello world, this is my description".to_string(),
            version: "1.0.0".to_string(),
            license: License::WTFPL,
            source: Some(Source::Domains),
        });
    }

    #[test]
    fn verify_require_license() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Source: domains

"#);
        assert!(metadata.is_err());
    }

    #[test]
    fn verify_require_opensource_license() {
        let metadata = Metadata::from_str(r#"-- Description: Hello world, this is my description
-- Version: 1.0.0
-- Source: domains
-- License: Proprietary

"#);
        assert!(metadata.is_err());
    }
}