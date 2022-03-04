use crate::connections::port_connection::PortConnection;

#[derive(Clone)]
pub struct Connection {
    pub name: String,
    pub init_connected: bool,
    pub port_connections: PortConnection
}
impl Connection {
    pub fn new(pname: &str, pinit_connected: bool, pport_connections: PortConnection) -> Connection {
        Connection{
            name: String::from(pname),
            init_connected: pinit_connected,
            port_connections: pport_connections
        }
    }

    //returns if ports of connection are connected
    pub fn connected(&self, client: &jack::Client) -> bool {
        let port = match client.port_by_name(&self.port_connections.audio_in) {
            Some(port) => port,
            None => return false
        };
        match port.is_connected_to(&self.port_connections.audio_out) {
            Ok(connected) => connected,
            Err(_err) => false
        }
    }

    pub fn connect(&mut self, client: &jack::Client) -> Result<(), String> {
        if !self.connected(client) {
            match client.connect_ports_by_name(&self.port_connections.audio_in, &self.port_connections.audio_out) {
                Ok(()) => {
                    return Ok(());
                }
                Err(_err) => return Err(format!("could not connect {}", &self.name))
            }
        }
        else {
            Err(format!("already connected {}", &self.name))
        }
        
    }

    pub fn disconnect(&mut self, client: &jack::Client) -> Result<(), String> {
        if self.connected(client) {
            match client.disconnect_ports_by_name(&self.port_connections.audio_in, &self.port_connections.audio_out) {
                Ok(()) => {
                    return Ok(());
                }
                Err(_err) => return Err(format!("could not disconnect {}", &self.name))
            }
        }
        else {
            Err(format!("already disconnected {}", &self.name))
        }
    }

    //connets ports which should be connected on program initialisation 
    pub fn init(&mut self, client: &jack::Client) -> Result<(), String> {
        if self.init_connected {
            match self.connect(client) {
                Ok(()) => return Ok(()),
                Err(err) => {
                    return Err(format!("Init: {}", err));
                }
            }
        }
        Ok(())
    }


    pub fn find_connection_by_name(name: &str, connections: &Vec<Connection>) -> Connection {
        let mut iter = connections.iter();
        iter.find(|&x| x.name == name).unwrap().clone()
    }
}