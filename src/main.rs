use std::path::PathBuf;

use addressables_rs::{catalog::{Catalog, CatalogError}, lookup::{EntryId, KeyDataValue}};
use camino::Utf8PathBuf;
use dialoguer::{ Select };
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use std::io::Error;

use astra_formats::TextBundle;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Catalog Bundle Tool",
    about = "Command-line tool to consult and edit a Unity Addressables Catalog"
)]
struct Opt {
    /// Treat the catalog as a bundle
    #[structopt(short, long)]
    bundled: bool,
    /// Path to the catalog file as a bundle or a JSON
    catalog_path: Utf8PathBuf,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Output dependencies for a prefab
    Dependencies(Dependencies),
    /// Extract the JSON from a bundle file
    Extract(Extract),
    /// Output a file addition compliant file for an existing Catalog entry
    Dump(Dump),
    /// Bring every bundle related to a prefab in a directory for decompilation.
    Gather(Gather),
}

#[derive(Debug, StructOpt)]
struct Add {
    /// Output path for the catalog file
    out_path: Utf8PathBuf,
    /// Path to the TOML with the entries to append
    toml_path: Utf8PathBuf,
}

#[derive(Debug, StructOpt)]
struct Dependencies {
    /// InternalId to find dependencies for. Make sure to surround it in quotation marks to not run into trouble.
    internal_id: String,
}

#[derive(Debug, StructOpt)]
struct Extract {
    /// Output path for the JSON file
    out_path: Utf8PathBuf,
}

#[derive(Debug, StructOpt)]
struct Dump {
    /// InternalId to dump. Make sure to surround it in quotation marks to not run into trouble.
    internal_id: String,
    /// Output path for the dumped entry
    out_path: Utf8PathBuf,
}


#[derive(Debug, StructOpt)]
struct Gather {
    /// InternalId to gather for. Make sure to surround it in quotation marks to not run into trouble.
    internal_id: String,
    /// Path for the "StreamingAssets/aa" directory in your dump
    aa_path: Utf8PathBuf,
    /// Output path for the gathered files
    out_path: Utf8PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct CatalogEntries {
    bundles: Vec<ExtraBundles>,
    prefabs: Vec<ExtraPrefabs>,
}

#[derive(Deserialize, Serialize)]
pub struct ExtraBundles {
    internal_id: String,
    internal_path: String,
}

#[derive(Deserialize, Serialize)]
pub struct ExtraPrefabs {
    internal_id: String,
    internal_path: String,
    dependencies: Vec<String>,
}

fn main() {
    let opt = Opt::from_args_safe().unwrap_or_else(|err| {
        println!("{}", err);
        std::process::exit(1);
    });

    match opt.cmd {
        Command::Dependencies(args) => {
                let res = if opt.bundled {
                    let mut bundle = TextBundle::load(&opt.catalog_path).unwrap();

                    Catalog::from_str(bundle.take_string().unwrap())
                } else {
                    Catalog::open(opt.catalog_path)
                };

                let catalog = match res {
                    Ok(val) => val,
                    Err(err) => {
                        match err {
                            CatalogError::Io(io) => {
                                println!("An error happened while trying to open the Catalog: {}", io)
                            }
                            CatalogError::Json(json) => {
                                println!("An error happened while trying to read the JSON: {}", json)
                            }
                            _ => (),
                        }

                        std::process::exit(1);
                    }
                };

                let internal_id = match catalog.get_internal_id_index(&args.internal_id) {
                    Some(id) => id,
                    None => {
                        let search: Vec<&String> = catalog
                            .m_InternalIds
                            .iter()
                            .filter(|id| id.contains(&args.internal_id))
                            .collect();

                        if search.is_empty() {
                            println!("Couldn't find the index for this InternalId. Make sure you've got the spelling right.");
                            std::process::exit(1);
                            unreachable!()
                        } else {
                            let selection = dialoguer::FuzzySelect::new()
                                .with_prompt(
                                    "Multiple InternalIds matching your input have been found, pick one or refine your search",
                                )
                                .items(&search)
                                .interact()
                                .unwrap();
                            catalog.get_internal_id_index(search[selection]).unwrap()
                        }
                    }
                };

                let entry = catalog
                    .get_entry_by_internal_id(internal_id)
                    .expect("No entry found for this InternalId. Is the file corrupted?");

                let dependencies = catalog
                    .get_dependencies(entry)
                    .expect("No dependency found for this InternalId. Are you sure this is a prefab?");

                dependencies.iter().for_each(|id| {
                    println!(
                        "Dependency found: {}",
                        catalog
                            .get_internal_id_from_index(catalog.get_entry(*id).unwrap().internal_id)
                            .unwrap()
                    )
                });
            }
        Command::Extract(args) => {
                let mut bundle = match TextBundle::load(&opt.catalog_path) {
                    Ok(bundle) => bundle,
                    Err(err) => {
                        println!("Couldn't not open the bundle file: {}", err);
                        std::process::exit(1);
                    }
                };

                std::fs::write(args.out_path, bundle.take_string().unwrap()).unwrap();
            },
        Command::Dump(args) => {
                // Get a Catalog instance depending on the opening method
                let res = if opt.bundled {
                    let mut bundle = TextBundle::load(&opt.catalog_path).unwrap();

                    Catalog::from_str(bundle.take_string().unwrap())
                } else {
                    Catalog::open(&opt.catalog_path)
                };

                // Check for any obvious error
                let catalog = match res {
                    Ok(val) => val,
                    Err(err) => {
                        match err {
                            CatalogError::Io(io) => {
                                println!("An error happened while trying to open the Catalog: {}", io)
                            }
                            CatalogError::Json(json) => {
                                println!("An error happened while trying to read the JSON: {}", json)
                            }
                            _ => (),
                        }

                        std::process::exit(1);
                    }
                };

                let internal_id = match catalog.get_internal_id_index(&args.internal_id) {
                    Some(id) => id,
                    None => {
                        let search: Vec<&String> = catalog
                            .m_InternalIds
                            .iter()
                            .filter(|id| id.contains(&args.internal_id))
                            .collect();

                        if search.is_empty() {
                            println!("Couldn't find the index for this InternalId. Make sure you've got the spelling right.");
                            std::process::exit(1);
                            unreachable!()
                        } else {
                            let selection = dialoguer::FuzzySelect::new()
                                .with_prompt(
                                    "Multiple InternalIds matching your input have been found, pick one or refine your search",
                                )
                                .items(&search)
                                .interact()
                                .unwrap();
                            catalog.get_internal_id_index(search[selection]).unwrap()
                        }
                    }
                };

                let entry = catalog
                    .get_entry_by_internal_id(internal_id)
                    .expect("No entry found for this InternalId. Is the file corrupted?");

                println!("Resource type: {}", entry.resource_type);
                println!("Provider type: {}", entry.provider_index);

                let internal_path = match catalog.get_key(entry.primary_key).expect("Couldn't get the KeyDataValue???") {
                    KeyDataValue::String { string, .. } => Some(string),
                    KeyDataValue::Hash(_) => None,
                }.expect("KeyDataValue is of type Hash. Is the file corrupted?");

                // TODO: Add CatalogEntries::new()
                let mut entries = CatalogEntries {
                    bundles: vec![],
                    prefabs: vec![],
                };

                let id = catalog.get_internal_id_from_index(internal_id).unwrap();

                // If 0, we're dealing with a bundle
                if entry.dependency_hash == 0 {
                    entries.bundles.push(ExtraBundles { internal_id: id.to_owned(), internal_path: internal_path.to_string() })
                } else {
                    let deps = catalog
                    .get_dependencies(entry)
                    .expect("No dependency found for this InternalId. Are you sure this is a prefab?");

                    let dependencies = deps.iter().map(|id| {
                            catalog
                                .get_internal_id_from_index(catalog.get_entry(*id).unwrap().internal_id)
                                .unwrap().to_owned()
                    }).collect();

                    // Just in case
                    if !deps.is_empty() {
                        let bundle_entry = catalog.get_entry(deps[0]).unwrap();

                        let bundle_id = catalog.get_internal_id_from_index(bundle_entry.internal_id).unwrap();
                        let bundle_path = match catalog.get_key(bundle_entry.primary_key).expect("Couldn't get the KeyDataValue???") {
                            KeyDataValue::String { string, .. } => Some(string),
                            KeyDataValue::Hash(_) => None,
                        }.expect("KeyDataValue is of type Hash. Is the file corrupted?");
                        entries.bundles.push(ExtraBundles { internal_id: bundle_id.to_owned(), internal_path: bundle_path.to_string() })
                    }

                    entries.prefabs.push(ExtraPrefabs {
                        internal_id: id.to_owned(),
                        internal_path: internal_path.to_string(),
                        dependencies
                    })
                }

                std::fs::write(args.out_path, serde_toml::to_string_pretty(&entries).unwrap()).unwrap();
                println!("Entry exported successfully.");
            }
        Command::Gather(gather) => {
            let res = if opt.bundled {
                let mut bundle = TextBundle::load(&opt.catalog_path).unwrap();
                Catalog::from_str(bundle.take_string().unwrap())
            } else {
                Catalog::open(opt.catalog_path)
            };

            let catalog = match res {
                Ok(val) => val,
                Err(err) => {
                    match err {
                        CatalogError::Io(io) => {
                            println!("An error happened while trying to open the Catalog: {}", io)
                        }
                        CatalogError::Json(json) => {
                            println!("An error happened while trying to read the JSON: {}", json)
                        }
                        _ => (),
                    }
                    std::process::exit(1);
                }
            };

            // let bundle_id = catalog.get_internal_id_index(gather.internal_id).unwrap();
            let bundle_id = match catalog.get_internal_id_index(&gather.internal_id) {
                Some(id) => id,
                None => {
                    let search: Vec<&String> = catalog
                        .m_InternalIds
                        .iter()
                        .filter(|id| id.contains(&gather.internal_id) && id.ends_with("prefab"))
                        .collect();
                    if search.is_empty() {
                        println!("Couldn't find the index for this InternalId. Make sure you've got the spelling right.");
                        std::process::exit(1);
                        unreachable!()
                    } else {
                        let selection = dialoguer::FuzzySelect::new()
                            .with_prompt(
                                "Multiple InternalIds matching your input have been found, pick one or refine your search",
                            )
                            .items(&search)
                            .interact()
                            .unwrap();
                        catalog.get_internal_id_index(search[selection]).unwrap()
                    }
                }
            };

            let bundle_entry = catalog.get_entry_by_internal_id(bundle_id).unwrap();

            let dependencies = catalog.get_dependencies(bundle_entry).unwrap();

            let all_deps = recursive_deps(&catalog, dependencies);

            let mut paths = all_deps.iter().filter_map(|id| {
                catalog.get_entry(id.clone())
            }).flat_map(|entry| {
                catalog.get_internal_id_from_index(entry.internal_id.0 as usize)
            }).collect::<Vec<_>>();

            let abs_paths = paths.iter_mut().map(|path| {
                (path.replace("{UnityEngine.AddressableAssets.Addressables.RuntimePath}", gather.aa_path.as_str()),
                path.replace("{UnityEngine.AddressableAssets.Addressables.RuntimePath}", "")
            )
            });

            for (from, to) in abs_paths {
                let out = PathBuf::from(format!("{}/{}", gather.out_path.as_str(), to));
                std::fs::create_dir_all(&out.parent().unwrap()).unwrap();
                if let Err(err) = std::fs::copy(&from, &out) {
                    match err.kind() {
                        std::io::ErrorKind::NotFound => println!("Could not find the bundle file in the AA directory. Is the path correct?\nPath computed: {from}"),
                        _ => todo!(),
                    }

                    std::process::exit(1);
                }
            }

            println!("Bundles successfully gathered in '{}'.", gather.out_path)
        },
    }
}

pub fn recursive_deps(catalog: &Catalog, entries: impl AsRef<[EntryId]>) -> Vec<EntryId> {
    let entries = entries.as_ref();

    let deps = entries.iter().filter_map(|id| {
        let entry = catalog.get_entry(id.clone())?;
        catalog.get_dependencies(entry)
    }).flat_map(|entries| {
        recursive_deps(catalog, entries)
    });

    [entries.to_vec(), deps.collect()].concat()
}

// TODO: Move this to library
// TODO: Write actual tests
#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use catalog::lookup::KeyDataValue;

    use crate::{recursive_deps, CatalogEntries, ExtraBundles, ExtraPrefabs};

    // #[test]
    // pub fn edit_test() {
    //     let catalog = Catalog::open("./catalog_edit.json").unwrap();
    //     let bundle_id = catalog.get_internal_id_index("{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle").unwrap();
    //     let bundle_entry = dbg!(catalog.get_entry_by_internal_id(bundle_id).unwrap());
    //     let bundle_key = dbg!(catalog.get_key(bundle_entry.primary_key).unwrap());
    //     let bundle_bucket = dbg!(catalog.get_bucket(bundle_entry.primary_key).unwrap());
    //     let bundle_entry_id = dbg!(catalog.get_entry_id_by_internal_id(bundle_id).unwrap());

    //     let prefab_id = catalog.get_internal_id_index("Assets/Share/Addressables/Unit/Model/uBody/Cor0AF/c069/Prefabs/uBody_Cor0AF_c069.prefab").unwrap();
    //     let prefab_entry = dbg!(catalog.get_entry_by_internal_id(prefab_id).unwrap());
    //     let prefab_key = dbg!(catalog.get_key(prefab_entry.primary_key).unwrap());
    //     let prefab_bucket = dbg!(catalog.get_bucket(prefab_entry.primary_key).unwrap());
    //     let prefab_entry_id = dbg!(catalog.get_entry_id_by_internal_id(prefab_id).unwrap());

    //     let dependency_key = dbg!(catalog.get_key(prefab_entry.dependency_key_idx).unwrap());
    //     let dependency_buncket = dbg!(catalog.get_bucket(prefab_entry.dependency_key_idx).unwrap());
    // }

    #[test]
    pub fn test() {
        let catalog = Catalog::open("./edited_catalog.json").unwrap();
        let bundle_id = catalog.get_internal_id_index("{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_customrp/shaders/chara/charastandard.shader.bundle").unwrap();
        let bundle_entry = dbg!(catalog.get_entry_by_internal_id(bundle_id).unwrap());
        let bundle_key = dbg!(catalog.get_key(bundle_entry.primary_key).unwrap());
        let bundle_bucket = dbg!(catalog.get_bucket(bundle_entry.primary_key).unwrap());
        let bundle_entry_id = dbg!(catalog.get_entry_id_by_internal_id(bundle_id).unwrap());

        let prefab_id = catalog.get_internal_id_index("Assets/Share/CustomRP/Shaders/Chara/CharaStandard.shader").unwrap();
        let prefab_entry = dbg!(catalog.get_entry_by_internal_id(prefab_id).unwrap());
        let prefab_key = dbg!(catalog.get_key(prefab_entry.primary_key).unwrap());
        let prefab_bucket = dbg!(catalog.get_bucket(prefab_entry.primary_key).unwrap());
        let prefab_entry_id = dbg!(catalog.get_entry_id_by_internal_id(prefab_id).unwrap());

        let dependency_key = dbg!(catalog.get_key(prefab_entry.dependency_key_idx).unwrap());
        let dependency_buncket = dbg!(catalog.get_bucket(prefab_entry.dependency_key_idx).unwrap());
    }

    #[test]
    pub fn output_example_toml() {
        let entries = CatalogEntries {
            bundles: vec![
                ExtraBundles {
                    internal_id: "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle".to_string(),
                    internal_path: "fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle".to_string(),
                },
                ExtraBundles {
                    internal_id: "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle".to_string(),
                    internal_path: "fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle".to_string(),
                },
            ],
            prefabs: vec![
                ExtraPrefabs {
                    internal_id: "Assets/Share/Addressables/Unit/Model/uBody/Cor0AF/c069/Prefabs/uBody_Cor0AF_c069.prefab".to_string(),
                    internal_path: "Unit/Model/uBody/Cor0AF/c069/Prefabs/uBody_Cor0AF_c069".to_string(),
                    dependencies: vec![
                        String::from("{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/cor0af/c069/prefabs/ubody_cor0af_c069.bundle")
                    ]
                },
                ExtraPrefabs {
                    internal_id: "Assets/Share/Addressables/Unit/Model/uBody/Cor0AF/c069/Prefabs/uBody_Cor0AF_c069.prefab".to_string(),
                    internal_path: "Unit/Model/uBody/Cor0AF/c069/Prefabs/uBody_Cor0AF_c069".to_string(),
                    dependencies: vec![]
                }
            ],
        };

        std::fs::write("dump.toml", serde_toml::to_string_pretty(&entries).unwrap()).unwrap()
    }
}
