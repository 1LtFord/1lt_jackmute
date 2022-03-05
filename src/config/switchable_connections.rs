use config_1lt::data::config_group::ConfigGroup;
use crate::config::muteable_connections::MuteableConnection;
use crate::config::config::New;

pub struct SwitchableConnection {
    pub default: String,
    pub connections: Vec<MuteableConnection>
}

impl New for SwitchableConnection {
    ///get SwitchableConnection config from config group
    fn new(cfg_group: ConfigGroup) -> SwitchableConnection {
        //SwitchableConnection must have a default connection attribute + minimum of 1 connection shortcut attribute pair
        if cfg_group.config_attributes().len() > 3 && (cfg_group.config_attributes().len() % 2) == 1 {
            //get indexes of attributes
            let mut idefault: usize = 0;
            let mut bdefault = false; //has default connection
            let mut vconnections = Vec::new();
            for i in 0..cfg_group.config_attributes().len() {
                //get default connection
                if cfg_group.config_attributes()[i].name.as_str() == "default" {
                    if bdefault == false {
                        idefault = i;
                        bdefault = true;
                    }
                    else {
                        panic!("{} 'SwitchableConnection' config group {} must only have one default attribute", Self::error_text_start(), cfg_group.group_name())
                    }
                }
                else {
                    //get switchable connections
                    if cfg_group.config_attributes()[i].name.contains("connection") {
                        vconnections.push(SwitchableConnection::get_connection_shortcut_pair(&cfg_group, i));
                    }
                }
            }

            //SwitchableConnection must have a default connection and minimum of 1 connection shortcut attribute pair
            if bdefault && vconnections.len() > 0 {
                return SwitchableConnection {
                    default: cfg_group.config_attributes()[idefault].value.clone(),
                    connections: vconnections
                }
            }
            else {
                panic!("{} Attribute names of 'SwitchableConnection' config group {} must be connection and shortcut", Self::error_text_start(), cfg_group.group_name())
            }
        }
        else {
            panic!("{} There must be 2 attributes in the 'SwitchableConnection' config group {}", Self::error_text_start(), cfg_group.group_name())
        }
    }
}

impl SwitchableConnection {
    ///search for connection shortcut pair
    fn get_connection_shortcut_pair(cfg_group: &ConfigGroup, connection_index: usize) -> MuteableConnection {
        //get connection
        let cidentifier = cfg_group.config_attributes()[connection_index].name.clone().replace("connection", "");
        //connection has identifier
        if cidentifier.chars().count() > 0 {
            for f in 0..cfg_group.config_attributes().len() {
                //get shortcut
                if cfg_group.config_attributes()[f].name.contains("shortcut") {
                    let sidentifier = cfg_group.config_attributes()[f].name.clone().replace("shortcut", "");
                    //shortcut has identifier
                    if sidentifier.chars().count() > 0 {
                        if cidentifier == sidentifier {
                            return MuteableConnection {
                                connection: cfg_group.config_attributes()[connection_index].value.clone(),
                                shortcut: match MuteableConnection::get_shortcut_char(&cfg_group.config_attributes()[f].value) {
                                    Ok(value) => value,
                                    Err(()) => panic!("{} 'SwitchableConnection' config group {} shortcut values must be a single char", Self::error_text_start(), cfg_group.group_name())
                                }
                            }
                        }
                    }
                    else {
                        panic!("{} 'SwitchableConnection' config group {} shortcut attributes must have a identifier", Self::error_text_start(), cfg_group.group_name())
                    }
                }
            }
        }
        else {
            panic!("{} 'SwitchableConnection' config group {} connection attributes must have a identifier", Self::error_text_start(), cfg_group.group_name())
        }
        //if no connection shortcut pair is found
        panic!("{} 'SwitchableConnection' config group {} each connection must have a shortcut with identical identifier", Self::error_text_start(), cfg_group.group_name())
        
    }
}