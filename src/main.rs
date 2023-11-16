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

    let platform = if cli.no_cook {
        println!("Skipping cook...");
        if util::get_cooked_dir(util::Platform::Win64).exists() {
            util::Platform::Win64
        } else if util::get_cooked_dir(util::Platform::Linux).exists() {
            util::Platform::Linux
        } else {
            bail!("No existing cooked assets found, please run without --no-cook to cook");
        }
    } else {
        let platform = cook_project()?;
        println!("Finished cooking");
        platform
    };

    package_mods(platform, cli.no_zip)?;

    Ok(())
}

fn cook_project() -> Result<util::Platform> {
    let ue = &std::env::var("UNREAL").context(
        "$UNREAL env var not set (must point to root of Unreal Engine editor install directory",
    )?;

    use path_absolutize::*;

    println!("Cooking project using Unreal Engine {}", ue);

    let path_win = Path::new(ue).join("Engine/Binaries/Win64/UE4Editor-Cmd.exe");
    let path_linux = Path::new(ue).join("Engine/Binaries/Linux/UE4Editor-Cmd");

    let (platform, cmd) = if path_win.exists() {
        (util::Platform::Win64, path_win)
    } else if path_linux.exists() {
        (util::Platform::Linux, path_linux)
    } else {
        bail!("Could not locate UE4Editor-Cmd");
    };

    let success = Command::new(cmd)
        .arg(Path::new("FSD.uproject").absolutize()?.to_str().unwrap())
        .arg("-run=cook")
        .arg(format!("-targetplatform={}", platform.target()))
        .status()
        .context("Cook failed")?
        .success();
    if success {
        Ok(platform)
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

struct Ctx {
    platform: util::Platform,
}

type FileProvider = fn(ctx: &Ctx) -> Result<Vec<FileEntry>>;

struct PackageJob {
    mod_name: &'static str,
    globs: &'static [&'static str],
    providers: &'static [FileProvider],
}
fn package_mods(platform: util::Platform, no_zip: bool) -> Result<()> {
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
        PackageJob {
            mod_name: "custom-map-rotation",
            globs: &["FSD/Content/_AssemblyStorm/CustomMapRotation/**"],
            providers: &[cmr::make],
        },
        PackageJob {
            mod_name: "launch-override",
            globs: &["FSD/Content/_AssemblyStorm/LaunchOverride/**"],
            providers: &[],
        },
        PackageJob {
            mod_name: "disable-season",
            globs: &[],
            providers: &[disable_season::make],
        },
    ];
    let output = Path::new("PackagedMods");
    fs::create_dir(output).ok();
    jobs.par_iter()
        .try_for_each(|j| package_mod(platform, j, no_zip))?;
    Ok(())
}

fn package_mod(platform: util::Platform, job: &'static PackageJob, no_zip: bool) -> Result<()> {
    let paks = make_mod(Ctx { platform }, job)?;

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

fn make_mod(ctx: Ctx, job: &PackageJob) -> Result<Vec<PakOutput>> {
    let mut data = vec![];
    let mut pak = repak::PakWriter::new(
        Cursor::new(&mut data),
        repak::Version::V11,
        "../../../".to_owned(),
        None,
    );
    let base = util::get_cooked_dir(ctx.platform);

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
        for file in provider(&ctx)? {
            pak.write_file(&file.path, &mut Cursor::new(file.data))?;
        }
    }
    pak.write_index()?;
    Ok(vec![PakOutput {
        name: job.mod_name.to_string(),
        data,
    }])
}

fn make_remove_all_particles(ctx: &Ctx) -> Result<Vec<FileEntry>> {
    let fsd = util::get_fsd_pak()?;
    let mut reader = BufReader::new(File::open(&fsd)?);
    let pak = repak::PakReader::new_any(&mut reader)?;

    let ar = ar::AssetRegistry::read(&mut Cursor::new(
        pak.get("FSD/AssetRegistry.bin", &mut reader)?,
    ))?;

    let cooked = util::get_cooked_dir(ctx.platform);
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

    pub(crate) fn splice_pls_base(ctx: &Ctx) -> Result<Vec<FileEntry>> {
        let version = EngineVersion::VER_UE4_27;
        let pls_base_path = "FSD/Content/Landscape/PLS_Base";

        let mut src = {
            let cooked = util::get_cooked_dir(ctx.platform);
            splice::read_asset(
                cooked.join("FSD/Content/_AssemblyStorm/CustomDifficulty2/Hook_PLS_Base.uasset"),
                version,
            )?
        };

        let mut dst = {
            let fsd = util::get_fsd_pak()?;
            let mut reader = BufReader::new(File::open(&fsd)?);
            let pak = repak::PakReader::new_any(&mut reader)?;

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

mod cmr {
    use std::io::{Read, Seek};

    use super::*;
    use uasset_utils::splice::{self, walk};
    use unreal_asset::{
        engine_version::EngineVersion,
        kismet::{EExprToken, ExLocalVirtualFunction, ExObjectConst, ExSelf, KismetExpression},
        types::PackageIndex,
        Asset,
    };

    fn find_import<C: Read + Seek>(
        asset: &Asset<C>,
        class_package: &str,
        class_name: &str,
        object_name: &str,
    ) -> Option<PackageIndex> {
        asset
            .imports
            .iter()
            .enumerate()
            .find(|(_, import)| {
                import.class_package.get_content(|n| n == class_package)
                    && import.class_name.get_content(|n| n == class_name)
                    && import.object_name.get_content(|n| n == object_name)
            })
            .map(|(index, _)| PackageIndex::from_import(index as i32).unwrap())
    }

    type ImportChain<'a> = Vec<ImportStr<'a>>;

    struct ImportStr<'a> {
        class_package: &'a str,
        class_name: &'a str,
        object_name: &'a str,
    }
    impl<'a> ImportStr<'a> {
        fn new(class_package: &'a str, class_name: &'a str, object_name: &'a str) -> ImportStr<'a> {
            ImportStr {
                class_package,
                class_name,
                object_name,
            }
        }
    }

    fn get_import<C: Read + Seek>(asset: &mut Asset<C>, import: ImportChain) -> PackageIndex {
        let mut pi = PackageIndex::new(0);
        for i in import {
            let ai = &asset
                .imports
                .iter()
                .enumerate()
                .find(|(_, ai)| {
                    ai.class_package.get_content(|n| n == i.class_package)
                        && ai.class_name.get_content(|n| n == i.class_name)
                        && ai.object_name.get_content(|n| n == i.object_name)
                        && ai.outer_index == pi
                })
                .map(|(index, _)| PackageIndex::from_import(index as i32).unwrap());
            pi = ai.unwrap_or_else(|| {
                let new_import = unreal_asset::Import::new(
                    asset.add_fname(i.class_package),
                    asset.add_fname(i.class_name),
                    pi,
                    asset.add_fname(i.object_name),
                    false,
                );
                asset.add_import(new_import)
            });
        }
        pi
    }

    pub(crate) fn make(_ctx: &Ctx) -> Result<Vec<FileEntry>> {
        let mut files = vec![];
        files.extend(patch_asset(
            "FSD/Content/UI/Menu_MissionSelectionMK3/_SCREEN_MissionSelectionMK3",
        )?);
        files.extend(patch_asset(
            "FSD/Content/UI/Menu_MissionSelectionMK3/ITM_MisSel_PlanetZone",
        )?);
        Ok(files)
    }

    fn patch_asset(path: &str) -> Result<Vec<FileEntry>> {
        let version = EngineVersion::VER_UE4_27;

        let mut asset = {
            let fsd = util::get_fsd_pak()?;
            let mut reader = BufReader::new(File::open(&fsd)?);
            let pak = repak::PakReader::new_any(&mut reader)?;

            let uasset = Cursor::new(pak.get(&format!("{path}.uasset"), &mut reader)?);
            let uexp = Cursor::new(pak.get(&format!("{path}.uexp"), &mut reader)?);
            unreal_asset::Asset::new(uasset, Some(uexp), version, None)?
        };

        let import = find_import(
            &asset,
            "/Script/CoreUObject",
            "Function",
            "GetAvailableMissions",
        )
        .unwrap();

        let cmr_lib = get_import(
            &mut asset,
            vec![
                ImportStr::new(
                    "/Script/CoreUObject",
                    "Package",
                    "/Game/_AssemblyStorm/CustomMapRotation/CMR_Lib",
                ),
                ImportStr::new(
                    "/Game/_AssemblyStorm/CustomMapRotation/CMR_Lib",
                    "CMR_Lib_C",
                    "Default__CMR_Lib_C",
                ),
            ],
        );

        let get_available_missions = asset
            .get_name_map()
            .get_mut()
            .add_fname("GetAvailableMissions");

        let ver = splice::AssetVersion::new_from(&asset);
        let mut statements = splice::extract_tracked_statements(&mut asset, ver, &None);

        for (_pi, statements) in statements.iter_mut() {
            for statement in statements {
                walk(&mut statement.ex, &|ex| {
                    if let KismetExpression::ExLet(let_) = ex {
                        if let KismetExpression::ExContext(ctx) = &mut *let_.expression {
                            if let KismetExpression::ExFinalFunction(f) = &*ctx.context_expression {
                                if f.stack_node == import {
                                    ctx.object_expression = Box::new(
                                        ExObjectConst {
                                            token: EExprToken::ExObjectConst,
                                            value: cmr_lib,
                                        }
                                        .into(),
                                    );

                                    ctx.context_expression = Box::new(
                                        ExLocalVirtualFunction {
                                            token: EExprToken::ExLocalVirtualFunction,
                                            virtual_function_name: get_available_missions.clone(),
                                            parameters: vec![
                                                ExSelf {
                                                    token: EExprToken::ExSelf,
                                                }
                                                .into(),
                                                *let_.variable.clone(),
                                            ],
                                        }
                                        .into(),
                                    );

                                    ctx.r_value_pointer.new.as_mut().unwrap().path.clear();
                                    ctx.r_value_pointer
                                        .new
                                        .as_mut()
                                        .unwrap()
                                        .resolved_owner
                                        .index = 0;

                                    *ex = *let_.expression.clone();
                                }
                            }
                        }
                    }
                });
            }
        }
        splice::inject_tracked_statements(&mut asset, ver, statements);

        let mut uasset = Cursor::new(vec![]);
        let mut uexp = Cursor::new(vec![]);
        asset.write_data(&mut uasset, Some(&mut uexp))?;
        Ok(vec![
            FileEntry {
                path: format!("{path}.uasset",),
                data: uasset.into_inner(),
            },
            FileEntry {
                path: format!("{path}.uexp"),
                data: uexp.into_inner(),
            },
        ])
    }
}

mod disable_season {
    use std::io::{Read, Seek};

    use super::*;
    use uasset_utils::splice::{self, walk};
    use unreal_asset::{
        engine_version::EngineVersion,
        kismet::{EExprToken, ExTrue, KismetExpression},
        types::PackageIndex,
        Asset,
    };

    fn find_import<C: Read + Seek>(
        asset: &Asset<C>,
        class_package: &str,
        class_name: &str,
        object_name: &str,
    ) -> Option<PackageIndex> {
        asset
            .imports
            .iter()
            .enumerate()
            .find(|(_, import)| {
                import.class_package.get_content(|n| n == class_package)
                    && import.class_name.get_content(|n| n == class_name)
                    && import.object_name.get_content(|n| n == object_name)
            })
            .map(|(index, _)| PackageIndex::from_import(index as i32).unwrap())
    }
    pub(crate) fn make(_ctx: &Ctx) -> Result<Vec<FileEntry>> {
        let mut files = vec![];
        files.extend(
            [
                patch_asset("FSD/Content/UI/Menu_Seasons/ITM_SeasonContentToggle")?,
                patch_asset("FSD/Content/UI/Menu_Seasons/WND_SeasonOverview")?,
            ]
            .into_iter()
            .flatten(),
        );
        Ok(files)
    }

    fn patch_asset(path: &str) -> Result<Vec<FileEntry>> {
        let version = EngineVersion::VER_UE4_27;

        let mut asset = {
            let fsd = util::get_fsd_pak()?;
            let mut reader = BufReader::new(File::open(&fsd)?);
            let pak = repak::PakReader::new_any(&mut reader)?;

            let uasset = Cursor::new(pak.get(&format!("{path}.uasset"), &mut reader)?);
            let uexp = Cursor::new(pak.get(&format!("{path}.uexp"), &mut reader)?);
            unreal_asset::Asset::new(uasset, Some(uexp), version, None)?
        };

        let can_opt_out = find_import(
            &asset,
            "/Script/CoreUObject",
            "Function",
            "CanOptOutOfSeasonContent",
        )
        .unwrap();

        let ver = splice::AssetVersion::new_from(&asset);
        let mut statements = splice::extract_tracked_statements(&mut asset, ver, &None);

        for (_pi, statements) in statements.iter_mut() {
            for statement in statements {
                walk(&mut statement.ex, &|ex| {
                    if matches!(ex, KismetExpression::ExFinalFunction(f) if f.stack_node == can_opt_out)
                    {
                        *ex = ExTrue {
                            token: EExprToken::ExTrue,
                        }
                        .into()
                    }
                });
            }
        }
        splice::inject_tracked_statements(&mut asset, ver, statements);

        let mut uasset = Cursor::new(vec![]);
        let mut uexp = Cursor::new(vec![]);
        asset.write_data(&mut uasset, Some(&mut uexp))?;
        Ok(vec![
            FileEntry {
                path: format!("{path}.uasset",),
                data: uasset.into_inner(),
            },
            FileEntry {
                path: format!("{path}.uexp"),
                data: uexp.into_inner(),
            },
        ])
    }
}
