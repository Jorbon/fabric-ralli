use std::{io::{Read, Write}, path::PathBuf, process::Command};

use crate::{api_structs::{FabricLoaderVersion, GradleVersion, LoomVersion, MinecraftVersion, ProjectVersion, YarnMappingsVersion}, common::*, semantic_version::{SemanticVersion, SemanticVersionRange, simplify_range_set}};


pub const JAVA_VERSION_TABLE: [(SemanticVersion, u32); 4] = [
    (SemanticVersion { major: 0, minor: 0, patch: 0, release: None, build: None }, 8),
    (SemanticVersion { major: 1, minor: 17, patch: 0, release: None, build: None }, 16),
    (SemanticVersion { major: 1, minor: 18, patch: 0, release: None, build: None }, 17),
    (SemanticVersion { major: 1, minor: 20, patch: 5, release: None, build: None }, 21),
];

const GRADLE: &str = "./gradlew.bat";
const GRADLE_PROPERTIES: &str = "gradle.properties";
const DEPENDENCIES: &str = "dependencies";


pub fn stop_gradle() -> Result<()> {
    println!("Stopping gradle daemons...");
    let result = Command::new(GRADLE).arg("--stop").output()?;
    if result.status.success() {
        println!("{}", String::from_utf8(result.stdout)?);
        Ok(())
    } else {
        Err(format!("Gradle command error: {result:?}").into())
    }
}

pub fn clean_gradle() -> Result<()> {
    match Command::new(GRADLE).arg("clean").arg("--no-build-cache").arg("--refresh-dependencies").output() {
        Ok(output) => {
            dbg!(output);
            Ok(())
        }
        Err(e) => Err(e.into())
    }
}



pub struct App {
    pub cwd: PathBuf,
    pub http_client: reqwest::blocking::Client,
    pub mc_versions: Box<[(SemanticVersion, u32)]>,
}

impl App {
    pub fn new() -> Self {
        let http_client = reqwest::blocking::Client::builder().user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))).build().unwrap();
        Self {
            cwd: std::env::current_dir().expect("No current working directory access"),
            http_client,
            mc_versions: Box::new([(SemanticVersion::default(), 0); 0]),
        }
    }

    pub fn update_gradle(&self) -> Result<()> {
        let response = self.http_client.get("https://services.gradle.org/versions/current").send()?;
        if !response.status().is_success() {
            return Err(format!("{:?}", response.error_for_status()).into())
        }
        
        let version = response.json::<GradleVersion>()?;
        let new_url = version.downloadUrl.replace(":", "\\:");
        
        let file_path = self.cwd.join("gradle/wrapper/gradle-wrapper.properties");
        let mut file = std::fs::File::options().read(true).open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let gradle_url_part = SubstringRef::find(&contents, "distributionUrl=", "\n").ok_or("Could not find location of active gradle source.")?;
        if gradle_url_part.substring == new_url {
            println!("Gradle version {} is up to date.", version.version);
            Ok(())
        } else {
            let new_contents = gradle_url_part.replace(&new_url);
            let mut file = std::fs::File::options().write(true).create(true).truncate(true).open(&file_path)?;
            file.write_all(new_contents.as_bytes())?;
            
            println!("Updating gradle version to {}", version.version);
            let result = Command::new(GRADLE).arg("--version").output()?;
            if result.status.success() {
                println!("{}", String::from_utf8(result.stdout)?);
                Ok(())
            } else {
                Err(format!("Gradle command error: {result:?}").into())
            }
        }
    }
    
    pub fn update_static_info(&self) -> Result<()> {
        let loom_response = self.http_client.get("https://api.github.com/repos/FabricMC/fabric-loom/releases/latest").send()?;
        if !loom_response.status().is_success() {
            return Err(format!("{:?}", loom_response.error_for_status()).into())
        }
        let loader_response = self.http_client.get("https://meta.fabricmc.net/v2/versions/loader").send()?;
        if !loader_response.status().is_success() {
            return Err(format!("{:?}", loader_response.error_for_status()).into())
        }
        
        let loom_version = loom_response.json::<LoomVersion>()?.tag_name;
        let loom_version_full = format!("{}-SNAPSHOT", loom_version);
        let loader_version = loader_response.json::<Box<[FabricLoaderVersion]>>()?.iter().filter(|v| v.stable).next().ok_or("No stable loader versions found.")?.version.clone();
        
        
        let file_path = self.cwd.join(GRADLE_PROPERTIES);
        let mut file = std::fs::File::options().read(true).open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let mut changed = false;
        
        let loom_version_part = SubstringRef::find(&contents, "\nloom_version=", "\n").ok_or("No property 'loom_version' found in gradle properties.")?;
        let contents = if loom_version_part.substring.trim() == loom_version_full {
            println!("Loom version {} is up to date.", loom_version);
            contents
        } else {
            println!("Updating loom to version {}", loom_version);
            changed = true;
            loom_version_part.replace(&loom_version_full)
        };
        
        let loader_version_part = SubstringRef::find(&contents, "\nloader_version=", "\n").ok_or("No property 'loader_version' found in gradle properties.")?;
        let contents = if loader_version_part.substring.trim() == loader_version {
            println!("Loader version {} is up to date.", loader_version);
            contents
        } else {
            println!("Updating loader to version {}", loader_version);
            changed = true;
            loader_version_part.replace(&loader_version)
        };
        
        if changed {
            let mut file = std::fs::File::options().write(true).create(true).truncate(true).open(&file_path)?;
            file.write_all(contents.as_bytes())?;
        }
        Ok(())
    }
    
    pub fn fetch_version_info(&mut self) -> Result<()> {
        let versions_response = self.http_client.get("https://meta.fabricmc.net/v2/versions/game").send()?;
        if !versions_response.status().is_success() {
            return Err(format!("Cound not get game version info: {:?}", versions_response.error_for_status()).into())
        }
        let mappings_response = self.http_client.get("https://meta.fabricmc.net/v2/versions/yarn").send()?;
        if !mappings_response.status().is_success() {
            return Err(format!("Cound not get yarn mappings info: {:?}", mappings_response.error_for_status()).into())
        }
        
        let mut versions = versions_response.json::<Box<[MinecraftVersion]>>()?.iter().filter_map(|v| {
            if v.stable {
                Some((v.version.parse().ok()?, 0u32))
            } else { None }
        }).collect::<Box<[_]>>();
        
        for mapping in mappings_response.json::<Box<[YarnMappingsVersion]>>()? {
            if let Ok(version) = mapping.gameVersion.parse() {
                if let Some(matching) = versions.iter_mut().find(|v| v.0 == version) {
                    matching.1 = u32::max(matching.1, mapping.build);
                }
            }
        }
        
        self.mc_versions = versions;
        Ok(())
    }
    
    pub fn clean_dependencies(&self) -> Result<()> {
        for entry in std::fs::read_dir(self.cwd.join(DEPENDENCIES))? {
            if let Ok(entry) = entry {
                if let Ok(t) = entry.file_type() {
                    if t.is_file() {
                        std::fs::remove_file(entry.path())?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_current_ranges(&self) -> Result<Vec<SemanticVersionRange>> {
        let file_path = self.cwd.join(GRADLE_PROPERTIES);
        let mut file = std::fs::File::options().read(true).open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let ranges_part = SubstringRef::find(&contents, "\nminecraft_compatible_range=", "\n").ok_or("No property 'minecraft_compatible_range' found in gradle properties.")?;
        
        let mut ranges = vec![];
        for range_string in ranges_part.substring.trim().trim_start_matches("[").trim_end_matches("]").split(",") {
            if range_string.is_empty() { continue }
            match range_string.trim().trim_matches('\"').parse() {
                Ok(range) => ranges.push(range),
                Err(e) => return Err(e),
            }
        }
        
        Ok(simplify_range_set(ranges))
    }
    
    pub fn test_version(&self, index: usize) -> Result<()> {
        let file_path = self.cwd.join(GRADLE_PROPERTIES);
        let mut file = std::fs::File::options().read(true).open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let substring = SubstringRef::find(&contents, "\nminecraft_version=", "\n").ok_or("No property 'minecraft_version' found in gradle properties.")?;
        let contents = substring.replace(&self.mc_versions[index].0.to_string());
        
        let substring = SubstringRef::find(&contents, "\nyarn_mappings=", "\n").ok_or("No property 'yarn_mappings' found in gradle properties.")?;
        let contents = substring.replace(&format!("{}+build.{}", self.mc_versions[index].0, self.mc_versions[index].1));
        
        let substring = SubstringRef::find(&contents, "\njava_version=", "\n").ok_or("No property 'java_version' found in gradle properties.")?;
        let java_version = JAVA_VERSION_TABLE.iter().rev().find_map(|(mc_version_start, java_version)| {
            if self.mc_versions[index].0 >= *mc_version_start {
                Some(*java_version)
            } else { None }
        }).unwrap_or(8);
        let contents = substring.replace(&java_version.to_string());
        
        let mut file = std::fs::File::options().write(true).create(true).truncate(true).open(&file_path)?;
        file.write_all(contents.as_bytes())?;
        
        println!("Testing Minecraft version {}.", self.mc_versions[index].0);
        Ok(())
    }
    
    pub fn fetch_dependencies(&self) -> Result<()> {
        let file_path = self.cwd.join(GRADLE_PROPERTIES);
        let mut file = std::fs::File::options().read(true).open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let version = SubstringRef::find(&contents, "\nminecraft_version=", "\n").ok_or("No property 'minecraft_version' found in gradle properties.")?.substring.parse::<SemanticVersion>()?;
        
        let mut new_contents = String::new();
        
        let mut lines = contents.split('\n');
        loop {
            if let Some(line) = lines.next() {
                new_contents.push_str("\n");
                new_contents.push_str(line);
                if let Some((_, part)) = line.split_once('#') {
                    if part.trim_start().to_lowercase().starts_with("ralli") { break }
                }
            } else { break }
        }
        
        for line in lines {
            new_contents.push_str("\n");
            if let Some((name, _)) = line.split_once("=") {
                match name.trim() {
                    "loom_version" | "loader_version" | "minecraft_compatible_range" | "minecraft_version" | "yarn_mappings" | "java_version" => {
                        new_contents.push_str(line);
                    }
                    name => {
                        let response = self.http_client.get(&format!("https://api.modrinth.com/v2/project/{}/version?loaders=[\"fabric\"]&game_versions=[\"{}\"]", name, version)).send()?;
                        if !response.status().is_success() {
                            return Err(format!("Cound not get version info for dependency '{}': {:?}", name, response.error_for_status()).into())
                        }
                        
                        let versions = response.json::<Box<[ProjectVersion]>>()?;
                        let dependency_version = versions.get(0).ok_or(format!("Dependency '{}' does not support Minecraft version {}.", name, version))?;
                        
                        new_contents.push_str(&format!("{}={}", name, dependency_version.version_number));
                        if line.ends_with("\r") { new_contents.push_str("\r"); }
                        
                        for file in &dependency_version.files {
                            let response = self.http_client.get(&file.url).send()?;
                            if !response.status().is_success() {
                                return Err(format!("Cound not download dependency '{}-{}': {:?}", name, dependency_version.version_number, response.error_for_status()).into())
                            }
                            
                            let file_path = self.cwd.join(DEPENDENCIES).join(format!("{}-{}.jar", name, dependency_version.version_number));
                            let mut file = std::fs::File::options().write(true).create(true).truncate(true).open(&file_path)?;
                            file.write_all(&response.bytes()?)?;
                            
                            break
                        }
                        
                        print!("Fetched '{}-{}', supports: ", name, dependency_version.version_number);
                        for (i, version) in dependency_version.game_versions.iter().enumerate() {
                            if i > 0 { print!(", "); }
                            print!("{}", version);
                        }
                        // for range in simplify_range_set(dependency_version.game_versions.iter().filter_map(|s| s.parse().ok()).collect()) {}
                        println!();
                    }
                }
            } else {
                new_contents.push_str(line);
            }
        }
        
        let mut file = std::fs::File::options().write(true).create(true).truncate(true).open(&file_path)?;
        file.write_all(new_contents.strip_prefix("\n").unwrap_or(&new_contents).as_bytes())?;
        Ok(())
    }
    
    pub fn confirm_version(&self) -> Result<()> {
        let file_path = self.cwd.join(GRADLE_PROPERTIES);
        let mut file = std::fs::File::options().read(true).open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let ranges_part = SubstringRef::find(&contents, "\nminecraft_compatible_range=", "\n").ok_or("No property 'minecraft_compatible_range' found in gradle properties.")?;
        let version_part = SubstringRef::find(&contents, "\nminecraft_version=", "\n").ok_or("No property 'minecraft_version' found in gradle properties.")?;
        
        let mut ranges = vec![];
        for range_string in ranges_part.substring.trim().trim_start_matches("[").trim_end_matches("]").split(",") {
            if range_string.is_empty() { continue }
            match range_string.trim().trim_matches('\"').parse() {
                Ok(range) => ranges.push(range),
                Err(e) => return Err(e),
            }
        }
        
        let version = version_part.substring.parse()?;
        let index = self.mc_versions.iter().position(|(v, _)| *v == version).ok_or("Current version not found in the Minecraft version list.")?;
        
        ranges.push(SemanticVersionRange {
            start: Some(version.clone()),
            end: match index.checked_sub(1) {
                Some(index) => Some(self.mc_versions[index].0.clone()),
                None => self.mc_versions.first().map(|(v, _)| SemanticVersion {
                    major: v.major,
                    minor: v.minor + 1,
                    patch: 0,
                    release: None,
                    build: None,
                })
            },
        });
        
        let mut new_ranges_string = String::from("[");
        for (i, range) in simplify_range_set(ranges).iter().enumerate() {
            if i != 0 { new_ranges_string.push_str(", "); }
            new_ranges_string.push_str("\"");
            new_ranges_string.push_str(&range.to_string());
            new_ranges_string.push_str("\"");
        }
        new_ranges_string.push_str("]");
        
        let mut file = std::fs::File::options().write(true).create(true).truncate(true).open(&file_path)?;
        file.write_all(ranges_part.replace(&new_ranges_string).as_bytes())?;
        
        println!("Added Minecraft version {} to the compatibility range.", version);
        Ok(())
    }
}

