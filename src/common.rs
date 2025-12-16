
// Every crate does this so I guess I will too
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct SubstringRef<'a> {
    pub source: &'a str,
    pub before: &'a str,
    pub substring: &'a str,
    pub after: &'a str,
}

impl<'a> SubstringRef<'a> {
    pub fn find(source: &'a str, start_pattern: &str, end_pattern: &str) -> Option<Self> {
        let (before, rest) = source.split_at_checked(source.find(start_pattern)? + start_pattern.len())?;
        let (substring, after) = rest.split_at_checked(rest.find(end_pattern)?)?;
        Some(Self { source, before, substring, after })
    }
    pub fn replace(&self, insert_string: &str) -> String {
        String::from(self.before) + insert_string + self.after
    }
}
