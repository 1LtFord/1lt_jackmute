use config_1lt::data::config_group::ConfigGroup;


pub struct Ports {
    pub port_in: String,
    pub port_out: String
}

impl crate::config::config::New for Ports {
    ///get port config from config group
    fn new(cfg_group: ConfigGroup) -> Ports{
        //ports must have 2 attributes
        if cfg_group.config_attributes().len() == 2 {
            //get indexes of attributes
            let mut iin: usize = 2;
            let mut iout: usize = 2;
            for i in 0..cfg_group.config_attributes().len() {
                match cfg_group.config_attributes()[i].name.as_str(){
                    "in" => iin = i,
                    "out" => iout = i,
                    _ => ()
                }
            }
            
            //ports must have a in and a out attribute
            if iin < 2 && iout < 2 {
                return Ports {
                    port_in: cfg_group.config_attributes()[iin].value.clone(),
                    port_out: cfg_group.config_attributes()[iout].value.clone()
                }
            }
            else {
                panic!("{} Attribute names of Ports config group {} must be 'in' and 'out'", Self::error_text_start(), cfg_group.group_name());
            }
        }
        else {
            panic!("{} There must be 2 attributes in the 'Ports' config group {}", Self::error_text_start(), cfg_group.group_name());
        }
    }
}