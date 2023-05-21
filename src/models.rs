pub mod agent;
pub mod rules;

pub trait YamlDeserialize {
    fn from_yaml_file(path: &std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: std::marker::Sized;
}
