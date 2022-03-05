use config_1lt::data::{
    config_group::ConfigGroup,
    config_attribute::ConfigAttribute
};


pub struct Connection {
    pub name: String,
    pub connect_init: bool,
    pub port_out: String,
    pub port_in: String
}

impl crate::config::config::New for Connection {
    ///get Connection config from config group
    fn new(cfg_group: ConfigGroup) -> Connection {
        //Connections must have 4 attributes
        if cfg_group.config_attributes().len() == 4 {
            //get indexes of attributes
            let mut iname: usize = 4;
            let mut iconnect_init: usize = 4;
            let mut iport_out: usize = 4;
            let mut iport_in: usize = 4;
            for i in 0..cfg_group.config_attributes().len() {
                match cfg_group.config_attributes()[i].name.as_str(){
                    "name" => iname = i,
                    "connect_init" => iconnect_init = i,
                    "port_out" => iport_out = i,
                    "port_in" => iport_in = i,
                    _ => ()
                }
            }

            //Connections must have a name, connect_init, port_out and port_in attribute
            if iname < 4 && iconnect_init < 4 && iport_out < 4 && iport_in < 4 {
                let vconnect_init;
                match Connection::get_connect_init(&cfg_group.config_attributes()[iconnect_init]) {
                    Ok(value) => vconnect_init = value,
                    Err(()) => panic!("{} connect_init value of config group {} must be true or false", Self::error_text_start(), cfg_group.group_name())
                }
                return Connection {
                    name: cfg_group.config_attributes()[iname].value.clone(),
                    connect_init: vconnect_init,
                    port_out: cfg_group.config_attributes()[iport_out].value.clone(),
                    port_in: cfg_group.config_attributes()[iport_in].value.clone()
                }
            }
            else {
                panic!("{} Attribute names of 'Connection' config group {} must be name, connect_init, port_out and port_in", Self::error_text_start(), cfg_group.group_name())
            }
        }
        else {
            panic!("{} There must be 4 attributes in the 'Connection' config group {}", Self::error_text_start(), cfg_group.group_name())
        }
    }

    
}

impl Connection {
    fn get_connect_init(cfg_attribute: &ConfigAttribute) -> Result<bool, ()> {
        if cfg_attribute.value == "true" {
            Ok(true)
        } else if cfg_attribute.value == "false" {
            Ok(false)
        }
        else {
            //if connect_init value isn't true or false
            Err(())
        }
    }
}