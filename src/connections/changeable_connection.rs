use crate::connections::connection::Connection;

#[derive(Clone)]
pub struct ChangeableConnection {
    pub shortcut: char,
    pub connection: Connection
}
impl ChangeableConnection {
    pub fn new(pshortcut: char, pconnection: Connection) -> ChangeableConnection{
        ChangeableConnection {
            shortcut: pshortcut,
            connection: pconnection
        }
    }

    pub fn toggle_connection(&mut self, client: &jack::Client) -> Result<(), String>{
        if self.connection.connected(client) {
            match self.connection.disconnect(client) {
                Ok(()) => Ok(()),
                Err(err) => Err(format!("Connection toggle: {}", err))
            }
        }
        else {
            match self.connection.connect(client) {
                Ok(()) => Ok(()),
                Err(_err) => Err(format!("Connection toggle: could not connect {}", &self.connection.name))
            }
        }
    }

    pub fn toggle(changeable_connections: &mut Vec<ChangeableConnection>, input: &String, client: &jack::Client) -> Result<(), Vec<String>>{
        let mut error_messages = Vec::new();

        for i in 0..changeable_connections.len() {
            if input.contains(changeable_connections[i].shortcut) {
                match changeable_connections[i].toggle_connection(client) {
                    Ok(()) => (),
                    Err(err) => error_messages.push(err)
                }
            }
        }

        if error_messages.len() > 0 {
            return Err(error_messages)
        }
        else {
            return Ok(())
        }
    }
}