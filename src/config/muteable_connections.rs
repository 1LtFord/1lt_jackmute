use config_1lt::data::config_group::ConfigGroup;


pub struct MuteableConnection {
    pub connection: String,
    pub shortcut: char,
}

impl crate::config::config::New for MuteableConnection {
    ///get MuteableConnection config from config group
    fn new(cfg_group: ConfigGroup) -> MuteableConnection {
        //MuteableConnection must have 2 attributes
        if cfg_group.config_attributes().len() == 2 {
            //get indexes of attributes
            let mut iconnection: usize = 2;
            let mut ishortcut: usize = 2;
            for i in 0..cfg_group.config_attributes().len() {
                match cfg_group.config_attributes()[i].name.as_str(){
                    "connection" => iconnection = i,
                    "shortcut" => ishortcut = i,
                    _ => ()
                }
            }

            //MuteableConnection must have a connection and shortcut attribute
            if iconnection < 2 && ishortcut < 2 {
                return MuteableConnection {
                    connection: cfg_group.config_attributes()[iconnection].value.clone(),
                    shortcut: match Self::get_shortcut_char(&cfg_group.config_attributes()[ishortcut].value) {
                        Ok(value) => value,
                        Err(()) => panic!("{} 'MuteableConnection' config group {} shortcut value must be a single char", Self::error_text_start(), cfg_group.group_name())
                    }
                }
            }
            else {
                panic!("{} Attribute names of 'MuteableConnection' config group {} must be connection and shortcut", Self::error_text_start(), cfg_group.group_name())
            }
        }
        else {
            panic!("{} There must be 2 attributes in the 'MuteableConnection' config group {}", Self::error_text_start(), cfg_group.group_name())
        }
    }

    
}

impl MuteableConnection {
    pub fn get_shortcut_char(shortcut: &String) -> Result<char, ()> {
        if shortcut.chars().count() == 1 as usize {
            match shortcut.chars().nth(0) {
                Some(value) => Ok(value),
                None => Err(())
            }
        }
        else {
            Err(())
        }
    }
}