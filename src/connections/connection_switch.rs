use crate::connections::{connection::Connection, changeable_connection::ChangeableConnection};

#[derive(Clone)]
pub struct ConnectionSwitch {
    pub standard: Connection,
    pub connections: Vec<ChangeableConnection>
}

impl ConnectionSwitch {
    pub fn switch(switchable_connections: &mut Vec<ConnectionSwitch>, client: &jack::Client, input: &String) -> Result<(), Vec<String>> {
        let mut error_messages = Vec::new();

        for sc in switchable_connections {
            for i in 0..sc.connections.len() {
                if input.contains(sc.connections[i].shortcut) {
    
                    //if already connected: disconnect switch connections and connect standard
                    if sc.connections[i].connection.connected(&client) {
                        //disconnect switch connections
                        for f in 0..sc.connections.len() {
                            if sc.connections[f].connection.connected(&client) {
                                match sc.connections[f].toggle_connection(&client) {
                                    Ok(()) => (),
                                    Err(err) => error_messages.push(err)
                                }
                            }
                        }
                        //connect standard if not already connected
                        if !sc.standard.connected(&client) {
                            match sc.standard.connect(&client) {
                                Ok(()) => (),
                                Err(err) => error_messages.push(err)
                            }
                        }
                    }
                    //If not connected: disconnect all other switch connections and connect
                    else {
                        //disconnect standard if connected
                        if sc.standard.connected(&client) {
                            match sc.standard.disconnect(&client) {
                                Ok(()) => (),
                                Err(err) => error_messages.push(err)
                            }
                        }
                        //disconnect switch connections if connected
                        for f in 0..sc.connections.len() {
                            if sc.connections[f].connection.connected(&client) {
                                match sc.connections[f].toggle_connection(&client) {
                                    Ok(()) => (),
                                    Err(err) => error_messages.push(err)
                                }
                            }
                        }
                        //connect
                        match sc.connections[i].toggle_connection(&client) {
                            Ok(()) => (),
                            Err(err) => error_messages.push(err)
                        }
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