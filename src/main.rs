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

struct FileEntry {
    path: String,
    data: Vec<u8>,
}

type FileProvider = fn() -> Result<Vec<FileEntry>>;

struct PackageJob {
    mod_name: &'static str,
    globs: &'static [&'static str],
    providers: &'static [FileProvider],
}
fn package_mods(no_zip: bool) -> Result<()> {
    let jobs = &[
        PackageJob {
            mod_name: "mission-log",
            globs: &["FSD/Content/_AssemblyStorm/MissionLog/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "customdifficulty",
            globs: &[
                "FSD/Content/_AssemblyStorm/CustomDifficulty/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/ResupplyCost/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "custom-difficulty2",
            globs: &[
                "FSD/Content/_AssemblyStorm/CustomDifficulty2/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/ResupplyCost/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
            providers: &[cd2::splice_pls_base],
        },
        PackageJob {
            mod_name: "betterspectator",
            globs: &["FSD/Content/_AssemblyStorm/BetterSpectator/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "missionselector",
            globs: &["FSD/Content/_AssemblyStorm/MissionSelector/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "sandboxutilities",
            globs: &[
                "FSD/Content/_AssemblyStorm/SandboxUtilities/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "generousmanagement",
            globs: &[
                "FSD/Content/_AssemblyStorm/GenerousManagement/**",
                "FSD/Content/_AssemblyStorm/Common/**",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "better-post-processing",
            globs: &["FSD/Content/_AssemblyStorm/BetterPostProcessing/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "advanced-darkness",
            globs: &[
                "FSD/Content/_AssemblyStorm/AdvancedDarkness/**",
                "FSD/Content/_AssemblyStorm/Common/GlobalFunctionsV4.{uasset,uexp}",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "event-log",
            globs: &["FSD/Content/_AssemblyStorm/EventLog/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "test-mod",
            globs: &[
                "FSD/Content/_AssemblyStorm/TestMod/**",
                "FSD/Content/_AssemblyStorm/Common/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "test-assets",
            globs: &["FSD/Content/_AssemblyStorm/TestAssets/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "mod-integration",
            globs: &["FSD/Content/_AssemblyStorm/ModIntegration/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "take-me-home",
            globs: &[
                "FSD/Content/_AssemblyStorm/TakeMeHome/**",
                "FSD/Content/_AssemblyStorm/Common/Logger/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "build-inspector",
            globs: &[
                "FSD/Content/_AssemblyStorm/BuildInspector/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "no-ragdolls",
            globs: &["FSD/Content/_AssemblyStorm/NoRagdolls/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "a-better-modding-menu",
            globs: &["FSD/Content/_AssemblyStorm/ABetterModdingMenu/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "testing",
            globs: &[
                "FSD/Content/_AssemblyStorm/Testing/**",
                "FSD/Content/_AssemblyStorm/Common/Logger/**",
                "FSD/Content/_Interop/StateManager/**",
            ],
            providers: &[],
        },
        PackageJob {
            mod_name: "skipstart",
            globs: &["FSD/Content/UI/Menu_StartScreen/UI_StartScreen.{uasset,uexp}"],
            providers: &[],
        },
        PackageJob {
            mod_name: "open-hub",
            globs: &["FSD/Content/_AssemblyStorm/OpenHub/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "remove-all-particles",
            globs: &[],
            providers: &[make_remove_all_particles],
        },
    ];
    let output = Path::new("PackagedMods");
    fs::create_dir(output).ok();
    jobs.par_iter().try_for_each(|j| package_mod(j, no_zip))?;
    Ok(())
}

fn package_mod(job: &'static PackageJob, no_zip: bool) -> Result<()> {
    let paks = make_mod(job)?;

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

fn make_mod(job: &PackageJob) -> Result<Vec<PakOutput>> {
    let mut data = vec![];
    let mut pak = repak::PakWriter::new(
        Cursor::new(&mut data),
        None,
        repak::Version::V11,
        "../../../".to_owned(),
        None,
    );
    let base = util::get_cooked_dir();

    let walker = globwalk::GlobWalkerBuilder::from_patterns(&base, job.globs)
        .follow_links(true)
        .file_type(globwalk::FileType::FILE)
        .build()?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    for entry in &walker {
        pak.write_file(
            entry.path().strip_prefix(&base)?.to_str().unwrap(),
            &mut BufReader::new(File::open(entry.path())?),
        )?;
    }
    for provider in job.providers {
        for file in provider()? {
            pak.write_file(&file.path, &mut Cursor::new(file.data))?;
        }
    }
    pak.write_index()?;
    Ok(vec![PakOutput {
        name: job.mod_name.to_string(),
        data,
    }])
}

fn make_remove_all_particles() -> Result<Vec<FileEntry>> {
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
            unreal_asset::engine_version::EngineVersion::VER_UE4_27,
            None,
        )?;
        (
            asset.search_name_reference("EmptyParticleSystem").unwrap(),
            asset
                .search_name_reference("/Game/_Tests/Dummy/EmptyParticleSystem")
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
            unreal_asset::engine_version::EngineVersion::VER_UE4_27,
            None,
        )?;
        (
            asset.search_name_reference("EmptyNiagaraSystem").unwrap(),
            asset
                .search_name_reference("/Game/_Tests/Dummy/EmptyNiagaraSystem")
                .unwrap(),
            asset,
        )
    };

    let mut files = vec![];
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
            *asset
                .get_name_map()
                .get_mut()
                .get_name_reference_mut(*name_index) = name.to_owned();
            *asset
                .get_name_map()
                .get_mut()
                .get_name_reference_mut(*path_index) = path.to_owned();
            //println!("{} {}", path, name);
            let mut uasset = Cursor::new(vec![]);
            let mut uexp = Cursor::new(vec![]);
            asset.write_data(&mut uasset, Some(&mut uexp))?;
            let path = path.strip_prefix("/Game/").unwrap();
            files.push(FileEntry {
                path: format!("FSD/Content/{path}.uasset",),
                data: uasset.into_inner(),
            });
            files.push(FileEntry {
                path: format!("FSD/Content/{path}.uexp"),
                data: uexp.into_inner(),
            });
        }
    }
    Ok(files)
}

mod cd2 {
    use super::*;
    use uasset_utils::splice;
    use unreal_asset::{
        engine_version::EngineVersion,
        kismet::{EExprToken, ExJump, KismetExpression},
        reader::archive_trait::ArchiveTrait,
    };

    pub(crate) fn splice_pls_base() -> Result<Vec<FileEntry>> {
        let version = EngineVersion::VER_UE4_27;
        let pls_base_path = "FSD/Content/Landscape/PLS_Base";

        let mut src = {
            let cooked = util::get_cooked_dir();
            splice::read_asset(
                cooked.join("FSD/Content/_AssemblyStorm/CustomDifficulty2/Hook_PLS_Base.uasset"),
                version,
            )?
        };

        let mut dst = {
            let fsd = util::get_fsd_pak()?;
            let mut reader = BufReader::new(File::open(&fsd)?);
            let pak = repak::PakReader::new_any(&mut reader, None)?;

            let uasset = Cursor::new(pak.get(&format!("{pls_base_path}.uasset"), &mut reader)?);
            let uexp = Cursor::new(pak.get(&format!("{pls_base_path}.uexp"), &mut reader)?);
            unreal_asset::Asset::new(uasset, Some(uexp), version, None)?
        };

        let ver = splice::AssetVersion::new_from(&dst);
        let src_statements =
            splice::extract_tracked_statements(&mut src, ver, &Some("src".to_string()));
        let hook = &splice::find_hooks(&src, &src_statements)["wait loop"];
        let mut statements = splice::extract_tracked_statements(&mut dst, ver, &None);

        for (pi, statements) in statements.iter_mut() {
            let insert_index = statements
                .iter()
                .enumerate()
                .find(|(_, ex)| {
                    if let KismetExpression::ExFinalFunction(ex) = &ex.ex {
                        if dst
                            .get_import(ex.stack_node)
                            .is_some_and(|f| f.object_name.get_content(|s| s == "SetSeed"))
                        {
                            if let [KismetExpression::ExLocalVariable(ex)] =
                                ex.parameters.as_slice()
                            {
                                return ex
                                    .variable
                                    .new
                                    .as_ref()
                                    .and_then(|p| p.path.last())
                                    .is_some_and(|n| n.get_content(|s| s == "K2Node_Event_seed"));
                            }
                        }
                    }
                    false
                })
                .map(|(i, _)| i);

            if let Some(insert_index) = insert_index {
                let mut iter = std::mem::take(statements)
                    .into_iter()
                    .enumerate()
                    .peekable();

                let mut new = vec![];

                while let Some((index, statement)) = iter.next() {
                    new.push(statement);

                    if index == insert_index {
                        new.push(splice::TrackedStatement {
                            ex: ExJump {
                                token: EExprToken::ExJump,
                                code_offset: hook.start_offset as u32,
                            }
                            .into(),
                            origin: hook.statements[0].origin.clone(),
                            points_to: None,
                            original_offset: None,
                        });
                        for inst in &hook.statements {
                            if inst.original_offset == hook.end_offset {
                                if let Some((_index, next)) = iter.peek() {
                                    new.push(splice::TrackedStatement {
                                        ex: ExJump {
                                            token: EExprToken::ExJump,
                                            code_offset: next.original_offset.unwrap() as u32,
                                        }
                                        .into(),
                                        origin: inst.origin.clone(),
                                        points_to: Some((None, *pi)),
                                        original_offset: inst.original_offset,
                                    });
                                }
                            } else {
                                new.push(splice::TrackedStatement {
                                    origin: inst.origin.clone(),
                                    points_to: None,
                                    original_offset: inst.original_offset,
                                    ex: splice::copy_expression(
                                        &src,
                                        &mut dst,
                                        hook.function,
                                        *pi,
                                        inst,
                                    ),
                                })
                            }
                        }
                    }
                }
                *statements = new;
            }
        }
        splice::inject_tracked_statements(&mut dst, ver, statements);

        let mut uasset = Cursor::new(vec![]);
        let mut uexp = Cursor::new(vec![]);
        dst.write_data(&mut uasset, Some(&mut uexp))?;
        Ok(vec![
            FileEntry {
                path: format!("{pls_base_path}.uasset",),
                data: uasset.into_inner(),
            },
            FileEntry {
                path: format!("{pls_base_path}.uexp"),
                data: uexp.into_inner(),
            },
        ])
    }
}
