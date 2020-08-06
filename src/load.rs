//! Responsible for loading all specified yamls
use anyhow::Context as _;
use serde::Deserialize;
use std::path::{Path, PathBuf};

fn find_yamls_in_directories(directories: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    if directories.is_empty() {
        return Ok(Vec::new());
    }
    let mut builder = ignore::WalkBuilder::new(&directories[0]);
    for p in directories.iter().skip(1) {
        builder.add(p);
    }
    let mut types = ignore::types::TypesBuilder::new();
    types.add_defaults();
    types.negate("all");
    types.select("yaml");
    builder.types(types.build()?);
    let walk = builder.build();
    let mut paths = Vec::new();
    for item in walk {
        let item = item?;
        let is_file = item.file_type().map_or(false, |ty| ty.is_file());
        if !is_file {
            continue;
        }
        paths.push(item.path().to_path_buf());
    }
    Ok(paths)
}

/// Takes slice of paths, and returns list of paths to all yaml files
/// in these prefixes
fn resolve_paths(paths: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    let mut resolved = Vec::new();
    let mut dirs = Vec::new();
    for path in paths {
        if path.is_file() {
            resolved.push(path.clone());
        } else {
            dirs.push(path.clone());
        }
    }
    resolved.append(&mut find_yamls_in_directories(&dirs)?);
    Ok(resolved)
}

fn load_documents_in_path(path: &Path) -> anyhow::Result<Vec<serde_yaml::Value>> {
    let data = std::fs::read_to_string(path)?;
    let rust_yamls = yaml_rust::YamlLoader::load_from_str(&data).context("invalid yaml")?;
    let mut serde_values = Vec::new();
    for rust_yaml in rust_yamls {
        let mut repr = String::new();
        let mut emitter = yaml_rust::YamlEmitter::new(&mut repr);
        emitter.dump(&rust_yaml)?;
        let serde_value = serde_yaml::from_str(&repr)?;
        serde_values.push(serde_value);
    }
    Ok(serde_values)
}

pub enum ObjectKind {
    Deployment(k8s_openapi::api::apps::v1::DeploymentSpec),
    //Service(k8s_openapi::api::core::v1::ServiceSpec),
    Job(k8s_openapi::api::batch::v1::JobSpec),
}

pub struct Object {
    pub name: String,
    pub kind: ObjectKind,
}

#[derive(Deserialize)]
struct RawObject {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    metadata: RawObjectMeta,
    #[serde(default)]
    spec: serde_yaml::Value,
}

#[derive(Deserialize)]
struct RawObjectMeta {
    name: String,
}

impl Object {
    ///Tries to parse object from YAML
    fn from_yaml(val: &serde_yaml::Value) -> anyhow::Result<Option<Object>> {
        let raw: RawObject = serde_yaml::from_value(val.clone())?;

        Ok(match (raw.api_version.as_str(), raw.kind.as_str()) {
            ("apps/v1", "Deployment") => {
                println!("Found deployment {}", raw.metadata.name);
                Some(Object {
                    name: raw.metadata.name.clone(),
                    kind: ObjectKind::Deployment(serde_yaml::from_value(raw.spec)?),
                })
            }
            /*("v1", "Service") => {
                println!("Found service {}", raw.metadata.name);
                Some(Object {
                    name: raw.metadata.name.clone(),
                    kind: ObjectKind::Service(serde_yaml::from_value(raw.spec)?),
                })
            }*/
            ("batch/v1", "Job") => {
                println!("Fount job {}", raw.metadata.name);
                Some(Object {
                    name: raw.metadata.name.clone(),
                    kind: ObjectKind::Job(serde_yaml::from_value(raw.spec)?),
                })
            }
            (_, kind) => {
                println!(
                    "Ignoring object {} of unknown kind {}",
                    raw.metadata.name, kind
                );
                None
            }
        })
    }
}

pub fn load(files: &[PathBuf]) -> anyhow::Result<Vec<Object>> {
    let paths = resolve_paths(files)?;
    let mut res = Vec::new();
    for path in paths {
        let mut documents = load_documents_in_path(&path)?;
        res.append(&mut documents);
    }
    let mut objects = Vec::new();
    for doc in res {
        let obj = Object::from_yaml(&doc)?;
        if let Some(obj) = obj {
            objects.push(obj);
        }
    }
    Ok(objects)
}
