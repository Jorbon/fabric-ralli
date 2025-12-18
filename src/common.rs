
// Every crate does this so I guess I will too
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

pub struct SubstringRef<'a> {
    pub before: &'a str,
    pub substring: &'a str,
    pub after: &'a str,
}

impl<'a> SubstringRef<'a> {
    pub fn find(source: &'a str, start_pattern: &str, end_pattern: &str) -> Option<Self> {
        let (before, rest) = source.split_at_checked(source.find(start_pattern)? + start_pattern.len())?;
        let (substring, after) = rest.split_at_checked(rest.find(end_pattern)?)?;
        Some(Self { before, substring, after })
    }
    pub fn replace(&self, insert_string: &str) -> String {
        String::from(self.before) + insert_string + self.after
    }
}

pub fn clean_folder(path: impl AsRef<std::path::Path>) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        if let Ok(entry) = entry {
            if let Ok(t) = entry.file_type() {
                if t.is_file() && !entry.file_name().to_string_lossy().starts_with("_") {
                    std::fs::remove_file(entry.path())?;
                }
            }
        }
    }
    Ok(())
}

pub fn run_command(host: impl AsRef<std::ffi::OsStr>, args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> Result<()> {
    std::process::Command::new(host).args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?.wait()?;
    Ok(())
}
