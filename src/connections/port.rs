use crate::config::ports::Ports;

pub struct PortIn {
    pub in_port: jack::Port<jack::AudioIn>,
}

impl PortIn {
    ///register in port on client and return port
    pub fn new(client: &jack::Client, pin_name: String) -> PortIn {
        PortIn {
            in_port: client.register_port(&pin_name, jack::AudioIn::default()).unwrap(),
            
        }
    }
    ///get portnames from config file and register in ports on client
    pub fn get_ports_in(client: &jack::Client, ports: Vec<Ports>) -> Vec<PortIn> {
        let mut ports_in =  Vec::new();
        for p in ports {
            ports_in.push(PortIn::new(&client, p.port_in));
        }
    
        ports_in
    }
}

pub struct PortOut {
    pub out_port: jack::Port<jack::AudioOut>,
}

impl PortOut {
    ///register out port on client and return port
    pub fn new(client: &jack::Client, pout_name: String) -> PortOut {
        PortOut{
            out_port: client.register_port(&pout_name, jack::AudioOut::default()).unwrap()
        }
    }
    ///get portnames from config file and register out ports on client
    pub fn get_ports_out(client: &jack::Client, ports: Vec<Ports>) -> Vec<PortOut> {
        let mut ports_out =  Vec::new();
        for p in ports {
            ports_out.push(PortOut::new(&client, p.port_out));
        }
        
        ports_out
    }
}




