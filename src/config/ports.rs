use config_1lt::data::{
    config_group::ConfigGroup,
};

pub struct Ports {
    pub port_in: String,
    pub port_out: String
}

impl Ports {
    ///get port config from config group
    pub fn new(cfg_group: ConfigGroup) -> Result<Ports, String> {
        //ports must have 2 attributes
        if cfg_group.config_attributes().len() == 2 {
            //ports must have a in and a out attribute
            if cfg_group.config_attributes()[0].name == "in" && cfg_group.config_attributes()[1].name == "out" {
                return Ok(Ports {
                    port_in: cfg_group.config_attributes()[0].value.clone(),
                    port_out: cfg_group.config_attributes()[1].value.clone()
                })
            }
            else {
                return Err(format!("Attribute names of Ports need to be 'in' and 'out'"))
            }
        }
        else {
            return Err(format!("There must be 2 attributes in a 'Ports' config group"))
        }
    }
}