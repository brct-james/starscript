use serde::Deserialize;
use serde_yaml;

#[derive(Debug, Deserialize)]
pub enum Rule {
    StalenessRule {
        name: String,
        seconds_till_stale: u32,
    },
}

impl super::YamlDeserialize for Vec<Rule> {
    fn from_yaml_file(path: &std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let res = serde_yaml::from_reader(reader)?;
        Ok(res)
    }
}
