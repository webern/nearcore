use std::collections::HashMap;

use super::{style, utils, Error, Expected, PackageOutcome, Workspace};

/// ensure all crates have the `publish = <true/false>` specification
pub fn has_publish_spec(workspace: &Workspace) -> Result<(), Error> {
    let outliers: Vec<_> = workspace
        .members
        .iter()
        .filter(|pkg| pkg.publish.is_none())
        .map(|pkg| PackageOutcome { pkg, value: None })
        .collect();

    if !outliers.is_empty() {
        return Err(Error::OutcomeError {
            msg: "These packages should have the `publish` specification".to_string(),
            expected: Some(Expected { value: "publish = <true/false>".to_string(), reason: None }),
            outliers,
        });
    }

    return Ok(());
}

/// ensure all crates specify a MSRV
pub fn has_rust_version(workspace: &Workspace) -> Result<(), Error> {
    let cargo_version = utils::cargo_version()?;
    if cargo_version.parsed < "1.58.0".parse()? {
        utils::warn!(
            "{} compliance check requires {} or later",
            style::highlight("rust-version"),
            style::highlight("cargo 1.58.0"),
        );
        return Ok(());
    }

    let outliers: Vec<_> = workspace
        .members
        .iter()
        .filter(|pkg| pkg.rust_version.is_none())
        .map(|pkg| PackageOutcome { pkg, value: None })
        .collect();

    for outlier in &outliers {
        println!("{} {:?}", outlier.pkg.name, outlier.pkg.rust_version);
    }

    if !outliers.is_empty() {
        return Err(Error::OutcomeError {
            msg: "These packages should specify a Minimum Supported Rust Version (MSRV)"
                .to_string(),
            expected: None,
            outliers,
        });
    }

    return Ok(());
}

/// ensure all crates are versioned to v0.0.0
pub fn is_unversioned(workspace: &Workspace) -> Result<(), Error> {
    let outliers = workspace
        .members
        .iter()
        .filter(|pkg| {
            !matches!(
                pkg.version,
                semver::Version {
                    major: 0,
                    minor: 0,
                    patch: 0,
                    ref pre,
                    ref build,
                } if pre == &semver::Prerelease::EMPTY
                  && build == &semver::BuildMetadata::EMPTY
            )
        })
        .map(|pkg| PackageOutcome { pkg, value: Some(pkg.version.to_string()) })
        .collect::<Vec<_>>();

    if !outliers.is_empty() {
        return Err(Error::OutcomeError {
            msg: "These packages shouldn't be versioned".to_string(),
            expected: Some(Expected { value: "0.0.0".to_string(), reason: None }),
            outliers,
        });
    }

    return Ok(());
}

/// ensures all crates have a rust-version spec less than
/// or equal to the version defined in rust-toolchain.toml
pub fn has_debuggable_rust_version(workspace: &Workspace) -> Result<(), Error> {
    let rust_toolchain =
        utils::parse_toml::<toml::Value>(workspace.root.join("rust-toolchain.toml"))?;
    let rust_toolchain = rust_toolchain["toolchain"]["channel"].as_str().unwrap().to_owned();

    let rust_toolchain = match semver::Version::parse(&rust_toolchain) {
        Ok(rust_toolchain) => rust_toolchain,
        Err(err) => {
            utils::warn!(
                "semver: unable to parse rustup channel from {}: {}",
                style::highlight("rust-toolchain.toml"),
                err
            );

            return Ok(());
        }
    };

    let outliers = workspace
        .members
        .iter()
        .filter(|pkg| {
            pkg.rust_version
                .as_ref()
                .map_or(false, |rust_version| rust_version.matches(&rust_toolchain))
        })
        .map(|pkg| PackageOutcome {
            pkg,
            value: Some(pkg.rust_version.as_ref().unwrap().to_string()),
        })
        .collect::<Vec<_>>();

    if !outliers.is_empty() {
        return Err(Error::OutcomeError {
            msg: "These packages have an incompatible `rust-version`".to_string(),
            expected: Some(Expected {
                value: format!("<={}", rust_toolchain),
                reason: Some("as defined in the `rust-toolchain`".to_string()),
            }),
            outliers,
        });
    }

    return Ok(());
}

pub fn has_unified_rust_edition(workspace: &Workspace) -> Result<(), Error> {
    let mut edition_groups = HashMap::new();

    for pkg in &workspace.members {
        *edition_groups.entry(&pkg.edition).or_insert(0) += 1;
    }

    let (most_common_edition, n_compliant) =
        edition_groups.into_iter().reduce(|a, b| if a.1 > b.1 { a } else { b }).unwrap();

    let outliers = workspace
        .members
        .iter()
        .filter(|pkg| pkg.edition != *most_common_edition)
        .map(|pkg| PackageOutcome { pkg, value: Some(pkg.edition.clone()) })
        .collect::<Vec<_>>();

    if !outliers.is_empty() {
        return Err(Error::OutcomeError {
            msg: "These packages have an unexpected rust-edition".to_string(),
            expected: Some(Expected {
                value: most_common_edition.to_string(),
                reason: Some(format!("used by {} other packages in the workspace", n_compliant)),
            }),
            outliers,
        });
    }

    return Ok(());
}

// / ensures all crates have a unified rust edition
// pub fn has_unified_rust_edition(workspace: &Workspace) -> Result<(), Error> {
//     let mut version_groups = HashMap::new();
//     // let mut outliers = vec![];

//     for pkg in pkgs {
//         if let None = pkg.rust_version {
//             outliers.push(pkg);
//         }
//     }

//     if !outliers.is_empty() {
//         return Err((
//             "These packages don't have a publish specification in their package manifest",
//             outliers,
//         ));
//     }

//     return Ok(());
// }
