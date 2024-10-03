use std::path::{Path, PathBuf};
use std::fmt;
use std::collections::HashMap;

/// Cargo manifest as understood by this crate
pub type Manifest = cargo_toml::Manifest<Metadata>;

/// Paths to specification files
#[derive(Deserialize)]
#[non_exhaustive]
pub enum SpecificationPaths {
    #[serde(rename = "spec")]
    Single(PathBuf),
    #[serde(rename = "bin")]
    PerBinary(HashMap<String, PathBuf>),
}

/// Metadata of this crate
#[derive(Deserialize)]
#[non_exhaustive]
pub struct ConfigureMeMetadata {
    /// Path to the specification
    ///
    /// Must be relative to Cargo.toml directory
    #[serde(flatten)]
    pub spec_paths: SpecificationPaths,
}

/// Metadata used in manifest
#[derive(Deserialize)]
pub struct Metadata {
    /// Metadata of this crate
    pub configure_me: Option<ConfigureMeMetadata>,
}

/// Error that occured when loading Cargo.toml
#[derive(Debug)]
pub struct LoadError {
    /// The reason why it failed
    pub error: cargo_toml::Error,
    path: std::borrow::Cow<'static, Path>,
}

impl LoadError {
    /// Path to the file that was attempted to be opened.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "faild to load Cargo manifest from {}: {}", self.path.display(), self.error)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Load(LoadError),
    MissingPackage,
    MissingMetadata,
    MissingConfigureMeMetadata,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Load(error) => fmt::Display::fmt(error, f),
            Error::MissingPackage => write!(f, "The manifest is missing package section"),
            Error::MissingMetadata => write!(f, "The manifest is missing metadata section"),
            Error::MissingConfigureMeMetadata => write!(f, "The manifest is missing metadata.configure_me section"),
        }
    }
}


impl From<LoadError> for Error {
    fn from(value: LoadError) -> Self {
        Error::Load(value)
    }
}

mod sealed {
    pub trait LoadManifest {
    }
}

/// Used for loading manifest from current dir
pub struct CurrentDir;

/// Represents all types that can be turned into Manifest
pub trait LoadManifest: sealed::LoadManifest {
    type Error: Into<super::Error>;
    type Manifest: std::borrow::Borrow<Manifest>;

    fn load_manifest(self) -> Result<Self::Manifest, Self::Error>;
}

impl LoadManifest for Manifest {
    type Error = std::convert::Infallible;
    type Manifest = Self;

    fn load_manifest(self) -> Result<Self::Manifest, Self::Error> {
        Ok(self)
    }
}

impl<'a> LoadManifest for &'a Manifest {
    type Error = std::convert::Infallible;
    type Manifest = Self;

    fn load_manifest(self) -> Result<Self::Manifest, Self::Error> {
        Ok(self)
    }
}

impl<'a> LoadManifest for &'a Path {
    type Error = LoadError;
    type Manifest = Manifest;

    fn load_manifest(self) -> Result<Self::Manifest, Self::Error> {
        Manifest::from_path_with_metadata(self).map_err(|error| {
            LoadError {
                path: self.to_owned().into(),
                error,
            }
        })
    }
}

impl LoadManifest for PathBuf {
    type Error = LoadError;
    type Manifest = Manifest;

    fn load_manifest(self) -> Result<Self::Manifest, Self::Error> {
        Manifest::from_path_with_metadata(&self).map_err(|error| {
            LoadError {
                path: self.into(),
                error,
            }
        })
    }
}

impl<'a> LoadManifest for &'a PathBuf {
    type Error = LoadError;
    type Manifest = Manifest;

    fn load_manifest(self) -> Result<Self::Manifest, Self::Error> {
        (&**self).load_manifest()
    }
}

impl LoadManifest for CurrentDir {
    type Error = LoadError;
    type Manifest = Manifest;

    fn load_manifest(self) -> Result<Self::Manifest, Self::Error> {
        let manifest_file: &Path = "Cargo.toml".as_ref();
        manifest_file.load_manifest()
    }
}

macro_rules! impl_load_manifest {
    ($($type:ty),*) => {
        $(
            impl sealed::LoadManifest for $type {}
        )*
    }
}

macro_rules! impl_load_manifest_ref {
    ($($type:ty),*) => {
        $(
            impl<'a> sealed::LoadManifest for &'a $type {}
        )*
    }
}

impl_load_manifest!(Manifest, PathBuf, CurrentDir);
impl_load_manifest_ref!(Manifest, PathBuf, Path);

pub (crate) fn get_dir() -> Result<PathBuf, super::Error> {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or(super::Error {
            data: super::ErrorData::MissingManifestDirEnvVar,
        })
        .map(Into::into)
}

