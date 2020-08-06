//! Defines Docker-Compose typings
use serde::{Serialize, Serializer};
use linked_hash_map::LinkedHashMap;
/// Top-level struct, represents the whole `docker-compose.yaml`
#[derive(Serialize)]
pub struct Compose {
    pub version: Version,
    pub services: LinkedHashMap<String, Service>,
}

impl Default for Compose {
    fn default() -> Self {
        Compose {
            version: Version::V2,
            services: LinkedHashMap::new(),
        }
    }
}

#[derive(Serialize)]
pub enum Version {
    #[serde(rename = "3.8")]
    V2,
}

#[derive(Serialize)]
pub struct Service {
    /// Docker image
    pub image: String,
    /// List of exposed ports
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<Port>,
    /// Dependencies
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    /// Docker's COMMAND
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// Docker's ENTRYPOINT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<Vec<String>>,
    /// Environment variables
    #[serde(skip_serializing_if = "LinkedHashMap::is_empty")]
    pub environment: LinkedHashMap<String, String>,
}

pub struct Port(pub u16);

impl Serialize for Port {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let me = self.0.to_string();
        me.serialize(serializer)
    }
}
