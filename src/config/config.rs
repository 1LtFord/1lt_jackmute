use config_1lt::data::{
    config_file::ConfigFile,
    config_group::ConfigGroup,
    config_attribute::ConfigAttribute
};
use config_1lt::file::{
    read::read_config_file,
    write::write_config_file
};

pub struct Config{
    file: ConfigFile
}

impl Config {
    ///read and parse config file
    pub fn new(file_path: String) -> Config {
        let config = Config {
            file: match read_config_file(file_path) {
                Ok(file) => file,
                Err(err) => panic!("Error while reading config file: {}", err)
            }
        };
        config
    }

    ///get ports config from config file
    pub fn get_ports(&self) -> Vec<crate::config::ports::Ports> {
        //get all config groups which names contain "ports"
        let cgs = self.get_config_groups_contains("ports");
        let mut ports = Vec::new();
        for cg in cgs {
            ports.push(match crate::config::ports::Ports::new(cg) {
                Ok(ports) => ports,
                Err(err) => panic!("Error while parsing ports from config file: {}", err) 
            })
        }
        ports
    }

    pub fn get_connections(&self) -> Vec<ConfigGroup> {
        self.get_config_groups_contains("connections")
    }

    pub fn get_muteable_connections(&self) -> Vec<ConfigGroup> {
        self.get_config_groups_contains("muteable_connection")
    }

    pub fn get_switchable_connections(&self) -> Vec<ConfigGroup> {
        self.get_config_groups_contains("switchable_connection")
    }

    ///get all config groups from config file which contain specific pattern
    pub fn get_config_groups_contains(&self, name_contains: &str) -> Vec<ConfigGroup> {
        let mut config_groups = Vec::new();
        for config_group in &self.file.config_groups {
            if config_group.group_name().contains(&name_contains) {
                config_groups.push(config_group.clone());
            }
        }

        config_groups
    }
}