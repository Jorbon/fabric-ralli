use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GradleVersion {
    pub version: String,
    // pub buildTime: String,
    // pub commitId: String,
    // pub current: bool,
    // pub snapshot: bool,
    // pub nightly: bool,
    // pub releaseNightly: bool,
    // pub activeRc: bool,
    // pub rcFor: String,
    // pub milestoneFor: String,
    // pub broken: bool,
    pub downloadUrl: String,
    // pub checksumUrl: String,
    // pub wrapperChecksumUrl: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoomVersion {
    pub tag_name: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct YarnMappingsVersion {
    pub gameVersion: String,
    // pub separator: String,
    pub build: u32,
    // pub maven: String,
    // pub version: String,
    // pub stable: bool,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct FabricIntermediaryVersion {
//     pub maven: String,
//     pub version: String,
//     pub stable: bool,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct FabricLoaderVersion {
    // pub separator: String,
    // pub build: u32,
    // pub maven: String,
    pub version: String,
    pub stable: bool,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct FabricInstallerVersion {
//     pub url: String,
//     pub maven: String,
//     pub version: String,
//     pub stable: bool,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Versions {
//     pub game: Box<[MinecraftVersion]>,
//     pub mappings: Box<[YarnMappingsVersion]>,
//     pub intermediary: Box<[FabricIntermediaryVersion]>,
//     pub loader: Box<[FabricLoaderVersion]>,
//     pub installer: Box<[FabricInstallerVersion]>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ProjectFileHashes {
//     pub sha1: String,
//     pub sha512: String,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectFile {
    // pub filename: String,
    pub url: String,
    // pub size: u64,
    // pub file_type: Option<String>,
    // pub primary: bool,
    // pub hashes: ProjectFileHashes,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ProjectDependency {
//     pub version_id: Option<String>,
//     pub project_id: String,
//     pub file_name: Option<String>,
//     pub dependency_type: String,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectVersion {
    pub game_versions: Box<[String]>,
    // pub loaders: Box<[String]>,
    // pub id: String,
    // pub project_id: String,
    // pub author_id: String,
    // pub featured: bool,
    // pub name: String,
    pub version_number: String,
    // pub changelog: String,
    // pub changelog_url: Option<String>,
    // pub date_published: String,
    // pub downloads: u64,
    // pub version_type: String,
    // pub status: String,
    // pub requested_status: Option<String>,
    // pub dependencies: Box<[String]>,
    pub files: Box<[ProjectFile]>,
}


