mod ar;
mod util;

use std::{
    fs::{self, File},
    io::{BufReader, Cursor},
    path::Path,
    process::Command,
};

use anyhow::{anyhow, bail, Context, Result};
use ar::Readable;
use clap::Parser;
use rayon::prelude::*;

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
        println!("Finished cooking");
    }

    package_mods(cli.no_zip)?;

    make_remove_all_particles()?;

    Ok(())
}

fn cook_project() -> Result<()> {
    let ue = &std::env::var("UNREAL")
        .context("$UNREAL env var not set (must point to Unreal Engine editor install directory")?;

    use path_absolutize::*;

    println!("Cooking project...");
    let success = Command::new(Path::new(ue).join("UE4Editor-Cmd"))
        .arg(Path::new("FSD.uproject").absolutize()?.to_str().unwrap())
        .arg("-run=cook")
        .arg(format!("-targetplatform={}", util::TARGET))
        .status()
        .context("Cook failed")?
        .success();
    if success {
        Ok(())
    } else {
        Err(anyhow!("Cook failed"))
    }
}

struct PakOutput {
    name: String,
    data: Vec<u8>,
}

enum PackageJob {
    Normal {
        mod_name: &'static str,
        globs: &'static [&'static str],
    },
    Custom(fn() -> Result<Vec<PakOutput>>),
}
fn package_mods(no_zip: bool) -> Result<()> {
    let jobs = &[
        PackageJob::Normal {
            mod_name: "mission-log",
            globs: &["FSD/Content/_AssemblyStorm/MissionLog/**"],
        },
        PackageJob::Normal {
            mod_name: "customdifficulty",
            globs: &[
                "FSD/Content/_AssemblyStorm/CustomDifficulty/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/ResupplyCost/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob::Normal {
            mod_name: "custom-difficulty2",
            globs: &[
                "FSD/Content/_AssemblyStorm/CustomDifficulty2/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/ResupplyCost/**",
                "FSD/Content/_Interop/StateManager/**",
                // TODO patch PLS_Base
            ],
        },
        PackageJob::Normal {
            mod_name: "betterspectator",
            globs: &["FSD/Content/_AssemblyStorm/BetterSpectator/**"],
        },
        PackageJob::Normal {
            mod_name: "missionselector",
            globs: &["FSD/Content/_AssemblyStorm/MissionSelector/**"],
        },
        PackageJob::Normal {
            mod_name: "sandboxutilities",
            globs: &[
                "FSD/Content/_AssemblyStorm/SandboxUtilities/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob::Normal {
            mod_name: "generousmanagement",
            globs: &[
                "FSD/Content/_AssemblyStorm/GenerousManagement/**",
                "FSD/Content/_AssemblyStorm/Common/**",
            ],
        },
        PackageJob::Normal {
            mod_name: "better-post-processing",
            globs: &["FSD/Content/_AssemblyStorm/BetterPostProcessing/**"],
        },
        PackageJob::Normal {
            mod_name: "advanced-darkness",
            globs: &[
                "FSD/Content/_AssemblyStorm/AdvancedDarkness/**",
                "FSD/Content/_AssemblyStorm/Common/GlobalFunctionsV4.{uasset,uexp}",
            ],
        },
        PackageJob::Normal {
            mod_name: "event-log",
            globs: &["FSD/Content/_AssemblyStorm/EventLog/**"],
        },
        PackageJob::Normal {
            mod_name: "test-mod",
            globs: &[
                "FSD/Content/_AssemblyStorm/TestMod/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob::Normal {
            mod_name: "test-assets",
            globs: &["FSD/Content/_AssemblyStorm/TestAssets/**"],
        },
        PackageJob::Normal {
            mod_name: "mod-integration",
            globs: &["FSD/Content/_AssemblyStorm/ModIntegration/**"],
        },
        PackageJob::Normal {
            mod_name: "take-me-home",
            globs: &[
                "FSD/Content/_AssemblyStorm/TakeMeHome/**",
                "FSD/Content/_AssemblyStorm/Common/Logger/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob::Normal {
            mod_name: "build-inspector",
            globs: &[
                "FSD/Content/_AssemblyStorm/BuildInspector/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob::Normal {
            mod_name: "no-ragdolls",
            globs: &["FSD/Content/_AssemblyStorm/NoRagdolls/**"],
        },
        PackageJob::Normal {
            mod_name: "a-better-modding-menu",
            globs: &["FSD/Content/_AssemblyStorm/ABetterModdingMenu/**"],
        },
        PackageJob::Normal {
            mod_name: "testing",
            globs: &[
                "FSD/Content/_AssemblyStorm/Testing/**",
                "FSD/Content/_AssemblyStorm/Common/Logger/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
        },
        PackageJob::Normal {
            mod_name: "skipstart",
            globs: &["FSD/Content/UI/Menu_StartScreen/UI_StartScreen.{uasset,uexp}"],
        },
        PackageJob::Normal {
            mod_name: "open-hub",
            globs: &["FSD/Content/_AssemblyStorm/OpenHub/**"],
        },
        PackageJob::Custom(make_remove_all_particles),
    ];
    let output = Path::new("PackagedMods");
    fs::create_dir(output).ok();
    jobs.par_iter().try_for_each(|j| package_mod(j, no_zip))?;
    Ok(())
}

fn package_mod(job: &'static PackageJob, no_zip: bool) -> Result<()> {
    let paks = match job {
        PackageJob::Normal { mod_name, globs } => make_mod(mod_name, globs)?,
        PackageJob::Custom(f) => f()?,
    };

    let output = Path::new("PackagedMods");
    for pak in paks {
        let out = output.join(format!("{}.pak", pak.name));
        fs::write(&out, &pak.data)?;
        println!("Packaged mod to {}", out.display());

        if !no_zip {
            let mut zip_buf = vec![];
            let mut zip = zip::ZipWriter::new(Cursor::new(&mut zip_buf));
            zip.start_file(format!("{}.pak", pak.name), Default::default())?;
            use std::io::Write;
            zip.write_all(&pak.data)?;
            zip.finish()?;
            drop(zip);

            let out = output.join(format!("{}.zip", pak.name));
            fs::write(&out, &zip_buf)?;
            println!("Zipped mod to {}", out.display());
        }
    }

    Ok(())
}

fn make_mod(mod_name: &str, globs: &[&str]) -> Result<Vec<PakOutput>> {
    let mut data = vec![];
    let mut pak = repak::PakWriter::new(
        Cursor::new(&mut data),
        None,
        repak::Version::V11,
        "../../../".to_owned(),
        None,
    );
    let base = util::get_cooked_dir();

    let walker = globwalk::GlobWalkerBuilder::from_patterns(&base, globs)
        .follow_links(true)
        .file_type(globwalk::FileType::FILE)
        .build()?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for entry in &walker {
        //println!("{}", entry.path().strip_prefix(&base)?.display());
        pak.write_file(
            entry.path().strip_prefix(&base)?.to_str().unwrap(),
            &mut BufReader::new(File::open(entry.path())?),
        )?;
    }
    pak.write_index()?;
    Ok(vec![PakOutput {
        name: mod_name.to_string(),
        data,
    }])
}

fn make_remove_all_particles() -> Result<Vec<PakOutput>> {
    let fsd = util::get_fsd_pak()?;
    let mut reader = BufReader::new(File::open(&fsd)?);
    let pak = repak::PakReader::new_any(&mut reader, None)?;

    let ar = ar::AssetRegistry::read(&mut Cursor::new(
        pak.get("FSD/AssetRegistry.bin", &mut reader)?,
    ))?;

    let cooked = util::get_cooked_dir();
    let mut ps_asset = {
        let path = cooked.join("FSD/Content/_Tests/Dummy/EmptyParticleSystem.uasset");
        let uasset = BufReader::new(File::open(&path)?);
        let uexp = BufReader::new(File::open(path.with_extension("uexp"))?);
        let asset = unreal_asset::Asset::new(
            uasset,
            Some(uexp),
            unreal_asset::engine_version::EngineVersion::UNKNOWN,
        )?;
        (
            asset
                .search_name_reference(&"EmptyParticleSystem".to_owned())
                .unwrap(),
            asset
                .search_name_reference(&"/Game/_Tests/Dummy/EmptyParticleSystem".to_owned())
                .unwrap(),
            asset,
        )
    };
    let mut ns_asset = {
        let path = cooked.join("FSD/Content/_Tests/Dummy/EmptyNiagaraSystem.uasset");
        let uasset = BufReader::new(File::open(&path)?);
        let uexp = BufReader::new(File::open(path.with_extension("uexp"))?);
        let asset = unreal_asset::Asset::new(
            uasset,
            Some(uexp),
            unreal_asset::engine_version::EngineVersion::UNKNOWN,
        )?;
        (
            asset
                .search_name_reference(&"EmptyNiagaraSystem".to_owned())
                .unwrap(),
            asset
                .search_name_reference(&"/Game/_Tests/Dummy/EmptyNiagaraSystem".to_owned())
                .unwrap(),
            asset,
        )
    };

    let mut data = vec![];
    let mut pak = repak::PakWriter::new(
        Cursor::new(&mut data),
        None,
        repak::Version::V11,
        "../../../".to_owned(),
        None,
    );
    for asset in ar.asset_data {
        let path = &ar.names[asset.package_name.0];
        let name = &ar.names[asset.asset_name.0];
        let class = &ar.names[asset.asset_class.0];
        let asset = match class.as_str() {
            "ParticleSystem" => Some(&mut ps_asset),
            "NiagaraSystem" => Some(&mut ns_asset),
            _ => None,
        };
        if let Some((name_index, path_index, asset)) = asset {
            *asset.get_name_map().get_mut().get_name_reference_mut(*name_index) = name.to_owned();
            *asset.get_name_map().get_mut().get_name_reference_mut(*path_index) = path.to_owned();
            //println!("{} {}", path, name);
            let mut uasset = Cursor::new(vec![]);
            let mut uexp = Cursor::new(vec![]);
            asset.write_data(&mut uasset, Some(&mut uexp))?;
            uasset.set_position(0);
            uexp.set_position(0);
            pak.write_file(
                &format!(
                    "FSD/Content/{}.uasset",
                    path.strip_prefix("/Game/").unwrap()
                ),
                &mut uasset,
            )?;
            pak.write_file(
                &format!("FSD/Content/{}.uexp", path.strip_prefix("/Game/").unwrap()),
                &mut uexp,
            )?;
        }
    }
    pak.write_index()?;

    Ok(vec![PakOutput {
        name: String::from("remove-all-particles"),
        data,
    }])
}
