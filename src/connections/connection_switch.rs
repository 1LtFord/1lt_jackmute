use crate::connections::{connection::Connection, changeable_connection::ChangeableConnection};

#[derive(Clone)]
pub struct ConnectionSwitch {
    pub standard: Connection,
    pub connections: Vec<ChangeableConnection>
}

impl ConnectionSwitch {
    pub fn switch(&mut self, client: &jack::Client, input: &String) -> Result<(), Vec<String>> {
        let mut error_messages = Vec::new();

        for i in 0..self.connections.len() {
            if input.contains(self.connections[i].shortcut) {

                //if already connected: disconnect switch connections and connect standard
                if self.connections[i].connection.connected(&client) {
                    //disconnect switch connections
                    for f in 0..self.connections.len() {
                        if self.connections[f].connection.connected(&client) {
                            match self.connections[f].toggle_connection(&client) {
                                Ok(()) => (),
                                Err(err) => error_messages.push(err)
                            }
                        }
                    }
                    //connect standard if not already connected
                    if !self.standard.connected(&client) {
                        match self.standard.connect(&client) {
                            Ok(()) => (),
                            Err(err) => error_messages.push(err)
                        }
                    }
                }
                //If not connected: disconnect all other switch connections and connect
                else {
                    //disconnect standard if connected
                    if self.standard.connected(&client) {
                        match self.standard.disconnect(&client) {
                            Ok(()) => (),
                            Err(err) => error_messages.push(err)
                        }
                    }
                    //disconnect switch connections if connected
                    for f in 0..self.connections.len() {
                        if self.connections[f].connection.connected(&client) {
                            match self.connections[f].toggle_connection(&client) {
                                Ok(()) => (),
                                Err(err) => error_messages.push(err)
                            }
                        }
                    }
                    //connect
                    match self.connections[i].toggle_connection(&client) {
                        Ok(()) => (),
                        Err(err) => error_messages.push(err)
                    }
                }
            }
        }

        if error_messages.len() > 0 {
            Err(error_messages)
        }
        else {
            Ok(())
        }
    }
}