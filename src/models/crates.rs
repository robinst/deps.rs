use std::borrow::Borrow;
use std::str::FromStr;

use failure::Error;
use ordermap::OrderMap;
use relative_path::RelativePathBuf;
use semver::{Version, VersionReq};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateName(String);

impl Into<String> for CrateName {
    fn into(self) -> String {
        self.0
    }
}

impl Borrow<str> for CrateName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for CrateName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for CrateName {
    type Err = Error;

    fn from_str(input: &str) -> Result<CrateName, Error> {
        let is_valid = input.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '_' || c == '-'
        });

        if !is_valid {
            Err(format_err!("failed to validate crate name: {}", input))
        } else {
            Ok(CrateName(input.to_string()))
        }
    }
}

#[derive(Debug)]
pub struct CrateRelease {
    pub name: CrateName,
    pub version: Version,
    pub yanked: bool
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrateDep {
    External(VersionReq),
    Internal(RelativePathBuf)
}

impl CrateDep {
    pub fn is_external(&self) -> bool {
        if let &CrateDep::External(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CrateDeps {
    pub main: OrderMap<CrateName, CrateDep>,
    pub dev: OrderMap<CrateName, CrateDep>,
    pub build: OrderMap<CrateName, CrateDep>
}

#[derive(Debug)]
pub struct AnalyzedDependency {
    pub required: VersionReq,
    pub latest_that_matches: Option<Version>,
    pub latest: Option<Version>
}

impl AnalyzedDependency {
    pub fn new(required: VersionReq) -> AnalyzedDependency {
        AnalyzedDependency {
            required,
            latest_that_matches: None,
            latest: None
        }
    }

    pub fn is_outdated(&self) -> bool {
        self.latest > self.latest_that_matches
    }
}

#[derive(Debug)]
pub struct AnalyzedDependencies {
    pub main: OrderMap<CrateName, AnalyzedDependency>,
    pub dev: OrderMap<CrateName, AnalyzedDependency>,
    pub build: OrderMap<CrateName, AnalyzedDependency>
}

impl AnalyzedDependencies {
    pub fn new(deps: &CrateDeps) -> AnalyzedDependencies {
        let main = deps.main.iter().filter_map(|(name, dep)| {
            if let &CrateDep::External(ref req) = dep {
                Some((name.clone(), AnalyzedDependency::new(req.clone())))
            } else {
                None
            }
        }).collect();
        let dev = deps.dev.iter().filter_map(|(name, dep)| {
            if let &CrateDep::External(ref req) = dep {
                Some((name.clone(), AnalyzedDependency::new(req.clone())))
            } else {
                None
            }
        }).collect();
        let build = deps.build.iter().filter_map(|(name, dep)| {
            if let &CrateDep::External(ref req) = dep {
                Some((name.clone(), AnalyzedDependency::new(req.clone())))
            } else {
                None
            }
        }).collect();
        AnalyzedDependencies { main, dev, build }
    }

    pub fn any_outdated(&self) -> bool {
        let main_any_outdated = self.main.iter()
            .any(|(_, dep)| dep.is_outdated());
        let dev_any_outdated = self.dev.iter()
            .any(|(_, dep)| dep.is_outdated());
        let build_any_outdated = self.build.iter()
            .any(|(_, dep)| dep.is_outdated());
        main_any_outdated || dev_any_outdated || build_any_outdated
    }
}

#[derive(Clone, Debug)]
pub enum CrateManifest {
    Package(CrateName, CrateDeps),
    Workspace { members: Vec<RelativePathBuf> },
    Mixed { name: CrateName, deps: CrateDeps, members: Vec<RelativePathBuf> }
}
