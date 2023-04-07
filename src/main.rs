use camino::Utf8PathBuf;
use catalog::lookup::{ExtraId, KeyDataValue};
use dialoguer::Select;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

use astra_formats::{Bundle, TextBundle};

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
    /// Append new entries to catalog
    Add(Add),
    /// Output dependencies for a prefab
    Dependencies(Dependencies),
    /// Extract the JSON from a bundle file
    Extract(Extract),
    /// Output a file addition compliant file for an existing Catalog entry
    Dump(Dump),
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
        Command::Add(args) => {
            // Get a Catalog instance depending on the opening method
            let res = if opt.bundled {
                let mut bundle = TextBundle::load(&opt.catalog_path).unwrap();

                catalog::catalog::Catalog::from_str(bundle.take_string().unwrap())
            } else {
                catalog::catalog::Catalog::open(&opt.catalog_path)
            };

            // Check for any obvious error
            let mut catalog = match res {
                Ok(val) => val,
                Err(err) => {
                    match err {
                        catalog::catalog::CatalogError::Io(io) => {
                            println!("An error happened while trying to open the Catalog: {}", io)
                        }
                        catalog::catalog::CatalogError::Json(json) => {
                            println!("An error happened while trying to read the JSON: {}", json)
                        }
                        _ => (),
                    }

                    std::process::exit(1);
                }
            };

            // Get the entries to add from the provided json
            let entries: CatalogEntries =
                serde_toml::from_str(&std::fs::read_to_string(args.toml_path).unwrap()).unwrap();

            // We're being lazy here and just getting a copy of an existing metadata for the entries we're about to add
            let extra = catalog
                .get_extra(ExtraId(200))
                .expect("Couldn't get ExtraId")
                .to_owned();

            // Add bundle entries beforehand, as prefab entries will most likely depend on them.
            entries.bundles.iter().for_each(|bundle| {
                catalog
                    .add_bundle(
                        bundle.internal_id.to_owned(),
                        bundle.internal_path.to_owned(),
                        extra.clone(),
                    )
                    .unwrap();
            });

            // Add prefab entries
            entries.prefabs.iter().for_each(|prefab| {
                catalog
                    .add_prefab(
                        prefab.internal_id.to_owned(),
                        prefab.internal_path.to_owned(),
                        &prefab.dependencies,
                    )
                    .unwrap();
            });

            // Save the file to the output path
            if opt.bundled {
                let mut bundle = TextBundle::load(&opt.catalog_path).unwrap();
                bundle
                    .replace_string(serde_json::to_string(&catalog).unwrap())
                    .unwrap();
                bundle.save(args.out_path).unwrap();
            } else {
                std::fs::write(args.out_path, serde_json::to_string(&catalog).unwrap()).unwrap();
            };
        }
        Command::Dependencies(args) => {
            let res = if opt.bundled {
                let mut bundle = TextBundle::load(&opt.catalog_path).unwrap();

                catalog::catalog::Catalog::from_str(bundle.take_string().unwrap())
            } else {
                catalog::catalog::Catalog::open(opt.catalog_path)
            };

            let catalog = match res {
                Ok(val) => val,
                Err(err) => {
                    match err {
                        catalog::catalog::CatalogError::Io(io) => {
                            println!("An error happened while trying to open the Catalog: {}", io)
                        }
                        catalog::catalog::CatalogError::Json(json) => {
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
                        let selection = Select::new()
                            .with_prompt(
                                "Some InternalIds matching your input have been found, pick one",
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

                catalog::catalog::Catalog::from_str(bundle.take_string().unwrap())
            } else {
                catalog::catalog::Catalog::open(&opt.catalog_path)
            };

            // Check for any obvious error
            let mut catalog = match res {
                Ok(val) => val,
                Err(err) => {
                    match err {
                        catalog::catalog::CatalogError::Io(io) => {
                            println!("An error happened while trying to open the Catalog: {}", io)
                        }
                        catalog::catalog::CatalogError::Json(json) => {
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
                        let selection = Select::new()
                            .with_prompt(
                                "Some InternalIds matching your input have been found, pick one",
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

            std::fs::write(args.out_path, serde_toml::to_string_pretty(&entries).unwrap()).unwrap()

        }
    }
}

// TODO: Move this to library
// TODO: Write actual tests
#[cfg(test)]
mod test {
    use catalog::lookup::KeyDataValue;

    use crate::{CatalogEntries, ExtraBundles, ExtraPrefabs};

    // #[test]
    // pub fn edit_test() {
    //     let catalog = catalog::catalog::Catalog::open("./catalog_edit.json").unwrap();
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

    // #[test]
    // pub fn test() {
    //     let catalog = catalog::catalog::Catalog::open("./catalog.json").unwrap();
    //     let bundle_id = catalog.get_internal_id_index("{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch/fe_assets_unit/model/ubody/byl0am/c535/prefabs/ubody_byl0am_c535.bundle").unwrap();
    //     let bundle_entry = dbg!(catalog.get_entry_by_internal_id(bundle_id).unwrap());
    //     let bundle_key = dbg!(catalog.get_key(bundle_entry.primary_key).unwrap());
    //     let bundle_bucket = dbg!(catalog.get_bucket(bundle_entry.primary_key).unwrap());
    //     let bundle_entry_id = dbg!(catalog.get_entry_id_by_internal_id(bundle_id).unwrap());

    //     let prefab_id = catalog.get_internal_id_index("Assets/Share/Addressables/Unit/Model/uBody/Byl0AM/c535/Prefabs/uBody_Byl0AM_c535.prefab").unwrap();
    //     let prefab_entry = dbg!(catalog.get_entry_by_internal_id(prefab_id).unwrap());
    //     let prefab_key = dbg!(catalog.get_key(prefab_entry.primary_key).unwrap());
    //     let prefab_bucket = dbg!(catalog.get_bucket(prefab_entry.primary_key).unwrap());
    //     let prefab_entry_id = dbg!(catalog.get_entry_id_by_internal_id(prefab_id).unwrap());

    //     let dependency_key = dbg!(catalog.get_key(prefab_entry.dependency_key_idx).unwrap());
    //     let dependency_buncket = dbg!(catalog.get_bucket(prefab_entry.dependency_key_idx).unwrap());
    // }

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
