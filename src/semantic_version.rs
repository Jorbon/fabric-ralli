use std::cmp::Ordering;

use reqwest::Version;

use crate::common::*;

/// Version information and formatting as defined by https://semver.org/
#[derive(Default, Debug, Clone)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub release: Option<String>,
    pub build: Option<String>,
}

impl SemanticVersion {
    pub fn next_version(mut self) -> Self {
        self.release = match self.release {
            None => Some("".to_owned()),
            Some(s) => Some(s + "."),
        };
        self
    }
    
    pub fn matches_numbers(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if self.minor > 0 || self.patch > 0 { write!(f, ".{}", self.minor)? }
        if self.patch > 0 { write!(f, ".{}", self.patch)? }
        if let Some(s) = &self.release { write!(f, "-{s}")? }
        if let Some(s) = &self.build   { write!(f, "+{s}")? }
        Ok(())
    }
}

impl std::str::FromStr for SemanticVersion {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let (s, build) = match s.split_once("+") {
            Some((s, build)) => (s, Some(build.to_owned())),
            None => (s, None),
        };
        let (s, release) = match s.split_once("-") {
            Some((s, release)) => (s, Some(release.to_owned())),
            None => (s, None),
        };
        let mut parts = s.splitn(3, ".");
        Ok(Self {
            major: parts.next().ok_or("No major version")?.parse()?,
            minor: match parts.next() {
                Some(s) => s.parse()?,
                None => 0,
            },
            patch: match parts.next() {
                Some(s) => s.parse()?,
                None => 0,
            },
            release,
            build,
        })
    }
}

impl PartialEq for SemanticVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch && self.release == other.release
    }
}

impl Eq for SemanticVersion {}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => match (&self.release, &other.release) {
                        (None, None) => Ordering::Equal,
                        (None, Some(_)) => Ordering::Greater,
                        (Some(_), None) => Ordering::Less,
                        (Some(self_release), Some(other_release)) => {
                            let mut self_parts = self_release.split(".");
                            let mut other_parts = other_release.split(".");
                            loop {
                                match (self_parts.next(), other_parts.next()) {
                                    (None, None) => break Ordering::Equal,
                                    (None, Some(_)) => break Ordering::Less,
                                    (Some(_), None) => break Ordering::Greater,
                                    (Some(self_part), Some(other_part)) => {
                                        let self_number = if self_part == "" { Ok(0u32) } else { self_part.parse() };
                                        let other_number = if other_part == "" { Ok(0u32) } else { other_part.parse() };
                                        let cmp = match (self_number, other_number) {
                                            (Err(_), Err(_)) => self_part.cmp(other_part),
                                            (Err(_), Ok(_)) => break Ordering::Greater,
                                            (Ok(_), Err(_)) => break Ordering::Less,
                                            (Ok(a), Ok(b)) => a.cmp(&b),
                                        };
                                        if cmp != Ordering::Equal { break cmp }
                                    }
                                }
                            }
                        }
                    }
                    result => result
                }
                result => result
            }
            result => result
        }
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Debug, Eq, PartialEq)]
pub enum VersionMatchType {
    EqualTo,
    GreaterThan,
    LessThan,
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
    MatchMajorVersion,
    MatchMinorVersion,
}


/// Version range as specified by https://wiki.fabricmc.net/documentation:fabric_mod_json_spec#versionrange
#[derive(Default, Debug)]
pub struct SemanticVersionRange {
    pub start: Option<SemanticVersion>,
    pub end: Option<SemanticVersion>,
}

impl SemanticVersionRange {
    pub fn contains(&self, version: &SemanticVersion) -> bool {
        self.start.as_ref().map(|start| *version >= *start).unwrap_or(true) && self.end.as_ref().map(|end| *version < *end).unwrap_or(true)
    }
}

impl std::fmt::Display for SemanticVersionRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.start, &self.end) {
            (None, None) => write!(f, "*"),
            (Some(start), None) => write!(f, ">={}", start),
            (None, Some(end)) => write!(f, "<{}", end),
            (Some(start), Some(end)) => write!(f, ">={} <{}", start, end),
        }
    }
}

impl std::str::FromStr for SemanticVersionRange {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let mut range = SemanticVersionRange::default();
        for mut s in s.split_whitespace() {
            let (start, end) = {
                let mut match_type = VersionMatchType::EqualTo;
                for (operator, mt) in [
                    (">=", VersionMatchType::GreaterThanOrEqualTo),
                    ("<=", VersionMatchType::LessThanOrEqualTo),
                    (">" , VersionMatchType::GreaterThan),
                    ("<" , VersionMatchType::LessThan),
                    ("=" , VersionMatchType::EqualTo),
                    ("^" , VersionMatchType::MatchMajorVersion),
                    ("~" , VersionMatchType::MatchMinorVersion),
                ] {
                    if let Some(remaining) = s.strip_prefix(operator) {
                        s = remaining;
                        match_type = mt;
                        break
                    }
                }
                
                let (s, build) = match s.split_once("+") {
                    Some((s, build)) => (s, Some(build.to_owned())),
                    None => (s, None),
                };
                let (s, release) = match s.split_once("-") {
                    Some((s, release)) => (s, Some(release.to_owned())),
                    None => (s, None),
                };
                let mut parts = s.splitn(3, ".");
                
                let major = parts.next().ok_or("No major version")?.parse()?;
                let minor = match parts.next() {
                    Some("X") | Some("x") | Some("*") => {
                        match_type = VersionMatchType::MatchMajorVersion;
                        0
                    }
                    Some(s) => s.parse()?,
                    None => 0,
                };
                let patch = match parts.next() {
                    Some("X") | Some("x") | Some("*") => {
                        if match_type != VersionMatchType::MatchMajorVersion {
                            match_type = VersionMatchType::MatchMinorVersion;
                        }
                        0
                    }
                    Some(s) => s.parse()?,
                    None => 0,
                };
                
                let base_version = SemanticVersion { major, minor, patch, release, build };
                
                match match_type {
                    VersionMatchType::EqualTo => (Some(base_version.clone()), Some(base_version)),
                    VersionMatchType::GreaterThanOrEqualTo => (Some(base_version), None),
                    VersionMatchType::LessThan => (None, Some(base_version)),
                    VersionMatchType::GreaterThan => (Some(base_version.next_version()), None),
                    VersionMatchType::LessThanOrEqualTo => (None, Some(base_version.next_version())),
                    VersionMatchType::MatchMajorVersion => (
                        Some(SemanticVersion { major: base_version.major    , minor: 0, patch: 0, release: Some("".to_owned()), build: None }),
                        Some(SemanticVersion { major: base_version.major + 1, minor: 0, patch: 0, release: Some("".to_owned()), build: None }),
                    ),
                    VersionMatchType::MatchMinorVersion => (
                        Some(SemanticVersion { major: base_version.major, minor: base_version.minor    , patch: 0, release: Some("".to_owned()), build: None }),
                        Some(SemanticVersion { major: base_version.major, minor: base_version.minor + 1, patch: 0, release: Some("".to_owned()), build: None }),
                    ),
                }
            };
            
            match (start, &range.start) {
                (Some(new_start), Some(current_start)) => if new_start > *current_start { range.start = Some(new_start) }
                (Some(new_start), None) => range.start = Some(new_start),
                (None, _) => ()
            }
            match (end, &range.end) {
                (Some(new_end), Some(current_end)) => if new_end < *current_end { range.end = Some(new_end) }
                (Some(new_end), None) => range.end = Some(new_end),
                (None, _) => ()
            }
        }
        
        Ok(range)
    }
}

/// Sort and merge overlapping ranges.
pub fn simplify_range_set(mut ranges: Vec<SemanticVersionRange>) -> Vec<SemanticVersionRange> {
    ranges.sort_by(|a, b| a.start.cmp(&b.start));
    let mut sorted_ranges = ranges.into_iter();
    let mut merged_ranges = vec![];
    
    let mut current = match sorted_ranges.next() {
        Some(range) => range,
        None => return merged_ranges,
    };
    
    for next in sorted_ranges {
        if next.start <= current.end {
            current.end = next.end;
        } else {
            merged_ranges.push(current);
            current = next;
        }
    }
    
    merged_ranges.push(current);
    merged_ranges
}

