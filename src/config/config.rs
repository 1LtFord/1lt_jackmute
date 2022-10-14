use config_1lt::data::{
    config_file::ConfigFile,
    config_group::ConfigGroup,
};
use config_1lt::file::{
    read::read_config_file,
};

pub trait New {
    fn new(cfg_group: ConfigGroup) -> Self;

    fn error_text_start() -> String {
        format!("Error while parsing ports from config file:") 
    }
}

pub struct Config{
    program_name: String,
    program_version: String,
    file: ConfigFile
}

impl Config {
    ///read and parse config file
    pub fn new(program_name: &str, program_version: &str, file_path: String) -> Config {
        //create configfile if not exists
        if !std::path::Path::new(&file_path).exists() {
            Config::create_config_file(&file_path);
        }

        //read config
        let config = Config {
            program_name: String::from(program_name),
            program_version: String::from(program_version),
            file: match read_config_file(file_path) {
                Ok(file) => file,
                Err(err) => panic!("Error while reading config file: {}", err)
            }
        };
        config
    }

    fn create_config_file(file_path: &String) {
        let path = std::path::Path::new(&file_path);
        let folder = path.parent().unwrap();
        match std::fs::create_dir_all(folder) {
            Ok(()) => match std::fs::File::create(&file_path) {
                Ok(_file) => (),
                Err(error) => panic!("create config file error: {}", error)
            },
            Err(error) => panic!("create config file directory error: {}", error)
        }
    }

    pub fn program_name(&self) -> String {
        self.program_name.clone()
    }

    pub fn program_version(&self) -> String {
        self.program_version.clone()
    }

    ///get ports config from config file
    pub fn get_ports(&self) -> Vec<crate::config::ports::Ports> {
        //get all config groups which names contain "ports"
        self.parse("ports")
    }

    ///get connections config from config file
    pub fn get_connections(&self) -> Vec<crate::config::connection::Connection> {
        //get all config groups which names contain "connections"
        self.parse("connections")
    }

    ///get muteable connections config from config file
    pub fn get_muteable_connections(&self) -> Vec<crate::config::muteable_connections::MuteableConnection> {
        self.parse("muteable_connection")
    }

    ///get switchable connections config from config file
    pub fn get_switchable_connections(&self) -> Vec<crate::config::switchable_connections::SwitchableConnection> {
        self.parse("switchable_connection")
    }

    fn parse<T: New>(&self,  name: &str) -> Vec<T> {
        let cgs = self.get_config_groups_contains(&name);
        let mut parsed = Vec::new();
        for cg in cgs {
            parsed.push(T::new(cg))
        }
        parsed
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