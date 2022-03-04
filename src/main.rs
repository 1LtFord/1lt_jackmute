mod connections;
mod config;

use std::env;
use std::io;
use crate::config::config::Config;
use crate::connections::{
    port_connection::PortConnection, 
    connection::Connection, 
    changeable_connection::ChangeableConnection, 
    connection_switch::ConnectionSwitch
};
use crate::connections::port::{
    PortIn,
    PortOut
};

fn main() {
    //get config
    let config = Config::new(get_config_file_path());

    let mut connections = get_connections();
    let mut muteable_connections = get_muteable_connections(&connections);
    let mut effect_connections = get_effect_connections(&connections);
    let mut error_messages = Vec::new();
    
    //init and get jack client
    let active_client = get_jack_client(&config);

    //connect all connections which should be connected on startup
    error_messages.append(&mut init_connections(&mut connections, active_client.as_client()));


    let mut run = true;
    #[allow(while_true)]
    while run {

        //Print connection status and error messages
        print_status(active_client.as_client(), &muteable_connections, &effect_connections, &error_messages);
        error_messages.clear();

        //wait for user input
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).ok();

        //clear terminal
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        
        if user_input.len() == 2 {
            
            //exit program
            if user_input.contains('e') {
                run = false;
            }

            //toggle muteable connections (mute-unmute)
            match &mut ChangeableConnection::toggle(&mut muteable_connections, &user_input, active_client.as_client()) {
                Ok(()) => (),
                Err(err) => error_messages.append(err)
            }
            
            //switch switchable connections
            match &mut effect_connections.switch(active_client.as_client(), &user_input) {
                Ok(()) => (),
                Err(err) => error_messages.append(err)
            }
        }
    }
    

    active_client.deactivate().unwrap();
}

fn get_jack_client(config: &Config) -> jack::AsyncClient<Notifications, jack::ClosureProcessHandler<impl FnMut(&jack::Client, &jack::ProcessScope)-> jack::Control>> {
    //Create Client
    let (client, _status) = jack::Client::new("1lt_jackmute", jack::ClientOptions::NO_START_SERVER).unwrap();
    let process = jack::ClosureProcessHandler::new(get_jack_process_callback(&client, config));

    // Activate the client, which starts the processing.
    client.activate_async(Notifications, process).unwrap()
}


fn get_jack_process_callback(client: &jack::Client, config: &Config) -> impl FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control {
    //register Ports on client
    let ports_in = PortIn::get_ports_in(client, config.get_ports());
    let mut ports_out = PortOut::get_ports_out(client, config.get_ports());
    
    //define process callback
    move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        if ports_in.len() != ports_out.len() {
            return jack::Control::Quit
        }

        //copy audio data from in ports to out ports
        for i in 0..ports_in.len() {
            let port_out = ports_out[i].out_port.as_mut_slice(ps);
            let port_in = ports_in[i].in_port.as_slice(ps);
            port_out.clone_from_slice(port_in);
        }
        jack::Control::Continue
    }
}


fn init_connections(connections: &mut Vec<Connection>, active_client: &jack::Client) -> Vec<String> {
    let mut error_messages = Vec::new();
    for i in 0..connections.len() {
        match connections[i].init(active_client) {
            Ok(()) => (),
            Err(err) => error_messages.push(err)
        }
    }
    error_messages
}


fn print_status(client: &jack::Client, muteable_connections: &Vec<ChangeableConnection>, effect_connections: &ConnectionSwitch, error_messages: &Vec<String>) {
    //print status
    //status muteable connections
    if muteable_connections.len() > 0 {
        for i in 0..muteable_connections.len() {
            println!("{}: {}", muteable_connections[i].connection.name, 
                if muteable_connections[i].connection.connected(client)
                {"unmuted"} 
                else 
                {"muted"}
            );
        }
    }
    
    //status switchable connections
    if effect_connections.connections.len() > 0 {
        if muteable_connections.len() > 0 {
            println!("");
        }
        for i in 0..effect_connections.connections.len() {

            println!("{}: {}", effect_connections.connections[i].connection.name, 
                if effect_connections.connections[i].connection.connected(client)
                {"active"} 
                else 
                {"inactive"}
            );
        }
    }
    
    //print error messages
    if error_messages.len() > 0 {
        println!("\n errors:");
        for error in error_messages {
            println!("{}", error);
        }
    }
}



fn get_connections() -> Vec<Connection> {
    let mut connections = Vec::new();

    //Mikrofon
    connections.push(Connection::new("mikrofon -> 1lt_jackmute", 
                                        false, 
                                        PortConnection::new("rode:capture_1", "1lt_jackmute:mikro_in")
                                    ));
    connections.push(Connection::new("jackmute_mikrofon -> ardour_standard", 
                                        true, 
                                        PortConnection::new("1lt_jackmute:mikro_out", "ardour:RODE_Podcaster/audio_in 1")
                                    ));

    //System_L
    connections.push(Connection::new("ardour_system_L -> 1lt_jackmute", 
                                        true, 
                                        PortConnection::new("ardour:OUT-System/audio_out 1", "1lt_jackmute:system_in_l")
                                    ));
    connections.push(Connection::new("jackmute_system_L -> ardour_stream", 
                                        true, 
                                        PortConnection::new("1lt_jackmute:system_out_l", "ardour:System-Stream/audio_in 1")
                                    ));
    connections.push(Connection::new("jackmute_system_L -> ardour_master", 
                                        true, 
                                        PortConnection::new("1lt_jackmute:system_out_l", "ardour:Master/audio_in 1")
                                    ));

    //System_R
    connections.push(Connection::new("ardour_system_R -> 1lt_jackmute", 
                                        true, 
                                        PortConnection::new("ardour:OUT-System/audio_out 2", "1lt_jackmute:system_in_r")
                                    ));
    connections.push(Connection::new("jackmute_system_R -> ardour_stream", 
                                        true, 
                                        PortConnection::new("1lt_jackmute:system_out_r", "ardour:System-Stream/audio_in 2")
                                    ));
    connections.push(Connection::new("jackmute_system_R -> ardour_master", 
                                        true, 
                                        PortConnection::new("1lt_jackmute:system_out_r", "ardour:Master/audio_in 2")
                                    ));
    
    //Effects
    connections.push(Connection::new("1lt_jackmute -> ardour_effect1", 
                                        false, 
                                        PortConnection::new("1lt_jackmute:mikro_out", "ardour-01:1/audio_in 1")
                                    ));
    connections.push(Connection::new("1lt_jackmute -> ardour_effect2", 
                                        false, 
                                        PortConnection::new("1lt_jackmute:mikro_out", "ardour-01:2/audio_in 1")
                                    ));
    connections.push(Connection::new("1lt_jackmute -> ardour_effect3", 
                                        false, 
                                        PortConnection::new("1lt_jackmute:mikro_out", "ardour-01:3/audio_in 1")
                                    ));
    connections.push(Connection::new("1lt_jackmute -> ardour_effect4", 
                                        false, 
                                        PortConnection::new("1lt_jackmute:mikro_out", "ardour-01:4/audio_in 1")
                                    ));

    connections
}

fn get_muteable_connections(connections: &Vec<Connection>) -> Vec<ChangeableConnection> {
    let mut muteable_connections = Vec::new();
    
    muteable_connections.push(ChangeableConnection::new('m', Connection::find_connection_by_name("mikrofon -> 1lt_jackmute", connections)));
    muteable_connections.push(ChangeableConnection::new('s', Connection::find_connection_by_name("ardour_system_L -> 1lt_jackmute", connections)));
    muteable_connections.push(ChangeableConnection::new('s', Connection::find_connection_by_name("ardour_system_R -> 1lt_jackmute", connections)));

    muteable_connections
}

fn get_effect_connections(connections: &Vec<Connection>) -> ConnectionSwitch {
    let mut muteable_connections = Vec::new();

    muteable_connections.push(ChangeableConnection::new('1', Connection::find_connection_by_name("1lt_jackmute -> ardour_effect1", connections)));
    muteable_connections.push(ChangeableConnection::new('2', Connection::find_connection_by_name("1lt_jackmute -> ardour_effect2", connections)));
    muteable_connections.push(ChangeableConnection::new('3', Connection::find_connection_by_name("1lt_jackmute -> ardour_effect3", connections)));
    muteable_connections.push(ChangeableConnection::new('4', Connection::find_connection_by_name("1lt_jackmute -> ardour_effect4", connections)));

    ConnectionSwitch{
        standard: Connection::find_connection_by_name("jackmute_mikrofon -> ardour_standard", connections),
        connections: muteable_connections
    }
}

///get config file location
fn get_config_file_path() -> String {
    let home_path = match env::var("HOME") {
        Ok(home_path) => home_path,
        Err(error) => panic!("No home directory set. Can't find config file path: {:?}", error),
    };
    format!("{}/.local/share/1lt_software/1lt_jackmute/{}", home_path, "config")
}



struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        //println!("JACK: thread init");
    }

    fn shutdown(&mut self, _status: jack::ClientStatus, _reason: &str) {
        //println!(
        //    "JACK: shutdown with status {:?} because \"{}\"",
        //    status, reason
        //);
    }

    fn freewheel(&mut self, _: &jack::Client, _is_enabled: bool) {
        //println!(
        //    "JACK: freewheel mode is {}",
        //    if is_enabled { "on" } else { "off" }
        //);
    }

    fn sample_rate(&mut self, _: &jack::Client, _srate: jack::Frames) -> jack::Control {
        //println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, _name: &str, _is_reg: bool) {
        //println!(
        //    "JACK: {} client with name \"{}\"",
        //    if is_reg { "registered" } else { "unregistered" },
        //    name
        //);
    }

    fn port_registration(&mut self, _: &jack::Client, _port_id: jack::PortId, _is_reg: bool) {
        //println!(
        //    "JACK: {} port with id {}",
        //    if is_reg { "registered" } else { "unregistered" },
        //    port_id
        //);
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        _port_id: jack::PortId,
        _old_name: &str,
        _new_name: &str,
    ) -> jack::Control {
        //println!(
        //    "JACK: port with id {} renamed from {} to {}",
        //    port_id, old_name, new_name
        //);
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        _port_id_a: jack::PortId,
        _port_id_b: jack::PortId,
        _are_connected: bool,
    ) {
        //println!(
        //    "JACK: ports with id {} and {} are {}",
        //    port_id_a,
        //    port_id_b,
        //    if are_connected {
        //        "connected"
        //    } else {
        //        "disconnected"
        //    }
        //);
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        //println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        //println!("JACK: xrun occurred");
        jack::Control::Continue
    }
}