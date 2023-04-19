use std::{
    fs::{self, File},
    io::{BufReader, Cursor},
    path::Path,
    process::Command,
};

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Skip cooking step and repackage existing cooked files
    #[arg(long)]
    no_cook: bool,

    /// Skip zipping
    #[arg(long)]
    no_zip: bool,
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    if !Path::new("FSD.uproject").exists() {
        bail!("FSD.uproject missing. Is the FSD project the working directory?")
    }

    if cli.no_cook {
        println!("Skipping cook...");
    } else {
        cook_project()?;
        println!("Finished building");
    }

    package_mods(cli.no_zip)?;

    Ok(())
}

#[cfg(target_os = "windows")]
const TARGET: &str = "WindowsNoEditor";
#[cfg(target_os = "linux")]
const TARGET: &str = "LinuxNoEditor";

fn cook_project() -> Result<()> {
    let ue = &std::env::var("UNREAL")
        .context("$UNREAL env var not set (must point to Unreal Engine editor install directory")?;

    use path_absolutize::*;

    println!("Cooking project...");
    let success = Command::new(Path::new(ue).join("UE4Editor-Cmd"))
        .arg(Path::new("FSD.uproject").absolutize()?.to_str().unwrap())
        .arg("-run=cook")
        .arg(format!("-targetplatform={TARGET}"))
        .status()
        .context("Cook failed")?
        .success();
    if success {
        Ok(())
    } else {
        Err(anyhow!("Cook failed"))
    }
}

struct PackageJob {
    mod_name: &'static str,
    globs: &'static [&'static str],
}
fn package_mods(no_zip: bool) -> Result<()> {
    let jobs = &[
        PackageJob {
            mod_name: "mission-log",
            globs: &["FSD/Content/_AssemblyStorm/MissionLog/**"],
        },
        PackageJob {
            mod_name: "customdifficulty",
            globs: &[
                "FSD/Content/_AssemblyStorm/CustomDifficulty/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/ResupplyCost/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob {
            mod_name: "custom-difficulty2",
            globs: &[
                "FSD/Content/_AssemblyStorm/CustomDifficulty2/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/ResupplyCost/**",
                "FSD/Content/_Interop/StateManager/**",
                // TODO patch PLS_Base
            ],
        },
        PackageJob {
            mod_name: "betterspectator",
            globs: &["FSD/Content/_AssemblyStorm/BetterSpectator/**"],
        },
        PackageJob {
            mod_name: "missionselector",
            globs: &["FSD/Content/_AssemblyStorm/MissionSelector/**"],
        },
        PackageJob {
            mod_name: "sandboxutilities",
            globs: &[
                "FSD/Content/_AssemblyStorm/SandboxUtilities/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob {
            mod_name: "generousmanagement",
            globs: &[
                "FSD/Content/_AssemblyStorm/GenerousManagement/**",
                "FSD/Content/_AssemblyStorm/Common/**",
            ],
        },
        PackageJob {
            mod_name: "better-post-processing",
            globs: &["FSD/Content/_AssemblyStorm/BetterPostProcessing/**"],
        },
        PackageJob {
            mod_name: "advanced-darkness",
            globs: &[
                "FSD/Content/_AssemblyStorm/AdvancedDarkness/**",
                "FSD/Content/_AssemblyStorm/Common/GlobalFunctionsV4.{uasset,uexp}",
            ],
        },
        PackageJob {
            mod_name: "event-log",
            globs: &["FSD/Content/_AssemblyStorm/EventLog/**"],
        },
        PackageJob {
            mod_name: "test-mod",
            globs: &[
                "FSD/Content/_AssemblyStorm/TestMod/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob {
            mod_name: "test-assets",
            globs: &["FSD/Content/_AssemblyStorm/TestAssets/**"],
        },
        PackageJob {
            mod_name: "mod-integration",
            globs: &["FSD/Content/_AssemblyStorm/ModIntegration/**"],
        },
        PackageJob {
            mod_name: "take-me-home",
            globs: &[
                "FSD/Content/_AssemblyStorm/TakeMeHome/**",
                "FSD/Content/_AssemblyStorm/Common/Logger/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob {
            mod_name: "build-inspector",
            globs: &[
                "FSD/Content/_AssemblyStorm/BuildInspector/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob {
            mod_name: "no-ragdolls",
            globs: &["FSD/Content/_AssemblyStorm/NoRagdolls/**"],
        },
        PackageJob {
            mod_name: "a-better-modding-menu",
            globs: &["FSD/Content/_AssemblyStorm/ABetterModdingMenu/**"],
        },
        PackageJob {
            mod_name: "testing",
            globs: &[
                "FSD/Content/_AssemblyStorm/Testing/**",
                "FSD/Content/_AssemblyStorm/Common/Logger/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob {
            mod_name: "skipstart",
            globs: &["FSD/Content/UI/Menu_StartScreen/UI_StartScreen.{uasset,uexp}"],
        },
        PackageJob {
            mod_name: "open-hub",
            globs: &["FSD/Content/_AssemblyStorm/OpenHub/**"],
        },
    ];
    use rayon::prelude::*;
    jobs.par_iter().try_for_each(|j| package_mod(j, no_zip))?;
    Ok(())
}

fn package_mod(job: &'static PackageJob, no_zip: bool) -> Result<()> {
    let output = Path::new("PackagedMods");
    fs::create_dir(output).ok();
    let mut buf = vec![];
    let mut pak = repak::PakWriter::new(
        Cursor::new(&mut buf),
        None,
        repak::Version::V11,
        "../../../".to_owned(),
        None,
    );
    let base = Path::new("Saved/Cooked").join(TARGET);

    let walker = globwalk::GlobWalkerBuilder::from_patterns(&base, job.globs)
        .follow_links(true)
        .build()?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for entry in &walker {
        if entry.file_type().is_file() {
            //println!("{}", entry.path().strip_prefix(&base)?.display());
            pak.write_file(
                entry.path().strip_prefix(&base)?.to_str().unwrap(),
                &mut BufReader::new(File::open(entry.path())?),
            )?;
        }
    }
    pak.write_index()?;
    let out = output.join(format!("{}.pak", job.mod_name));
    fs::write(&out, &buf)?;
    println!("Packaged {} files to {}", walker.len(), out.display());

    if !no_zip {
        let mut zip_buf = vec![];
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_buf));
        zip.start_file(format!("{}.pak", job.mod_name), Default::default())?;
        use std::io::Write;
        zip.write_all(&buf)?;
        zip.finish()?;
        drop(zip);

        let out = output.join(format!("{}.zip", job.mod_name));
        fs::write(&out, &zip_buf)?;
        println!("Zipped mod to {}", out.display());
    }

    Ok(())
}
