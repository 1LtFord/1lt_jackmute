mod connections;
mod config;

use std::env;
use std::io;
use std::io::{prelude::*, BufReader, BufWriter};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::os::unix::net::{UnixListener, UnixStream};

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
    let config = Config::new("1lt_jackmute", env!("CARGO_PKG_VERSION"), get_config_file_path());

    let mut args: Vec<_> = env::args().collect();
    let args = args.split_off(1);
    if args.len() >= 1 {
        client(config, args);
    }
    else {
        server(config);
    }
}

fn client(_config: Config, args: Vec<String>) {
    let mut path = env::temp_dir();
    path.push("1lt_jackmute.sock");
    let mut stream = BufWriter::new(match UnixStream::connect(path) {
        Ok(stream) => stream, 
        Err(error) => panic!("network error {}", error)}
    );

    for arg in args {
        match stream.write(arg.as_bytes()) {
            Ok(_) => (),
            Err(error) => panic!("network error {}", error)
        };
        match stream.flush() {
            Ok(()) => (),
            Err(error) => panic!("network error {}", error)
        }
    }
}

fn server(config: Config) {
    let mut connections = get_connections(&config);
    let mut muteable_connections = get_muteable_connections(&config, &connections);
    let mut switchable_connections = get_switchable_connections(&config, &connections);
    let mut error_messages = Vec::new();
    
    //init and get jack client
    let active_client = get_jack_client(&config);

    //connect all connections which should be connected on startup
    error_messages.append(&mut init_connections(&mut connections, active_client.as_client()));

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let txt = tx.clone();
    let txn = tx.clone();
    thread::spawn(move || terminal_input(txt));
    thread::spawn(move || unix_socket_input(txn));


    let mut run = true;
    #[allow(while_true)]
    while run {

        //Print connection status and error messages
        print_status(config.program_name(), config.program_version(), active_client.as_client(), &muteable_connections, &switchable_connections, &error_messages);
        error_messages.clear();

        //wait for user input
        let user_input = match rx.recv() {
            Ok(message) => message,
            Err(error) => panic!("inter process communication error: {}", error)
        };

        //clear terminal
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        
        if user_input.len() <= 2 && user_input.len() >= 1 {
            
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
            match &mut ConnectionSwitch::switch(&mut switchable_connections, active_client.as_client(), &user_input) {
                Ok(()) => (),
                Err(err) => error_messages.append(err)
            }
        }
    }
    

    active_client.deactivate().unwrap();
}

fn terminal_input(txt: Sender<String>) {
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
    match txt.send(user_input) {
        Ok(()) => (),
        Err(error) => panic!("inter process communication error: {}", error)
    };
    thread::spawn(move || terminal_input(txt));
}

fn unix_socket_input(txn: Sender<String>) {
    let mut path = env::temp_dir();
    path.push("1lt_jackmute.sock");
    if path.exists() {
        std::fs::remove_file(&path).unwrap();
    }

    let listener = match UnixListener::bind(&path) {
        Ok(listener) => listener,
        Err(error) => panic!("could not create unix socket at {} | {}", &path.display(), error)
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match txn.send(get_command_from_unix_stream(stream)) {
                Ok(()) => (),
                Err(error) => panic!("communication error: {}", error)
            },
            Err(error) => panic!("error while reading from unix socket: {}", error)
        };
    }
}

fn get_command_from_unix_stream(mut stream: UnixStream) -> String {
    let buf_reader = BufReader::new(&mut stream);
    let command = buf_reader.lines().next();
    match command {
        Some(command) => 
        match command {
            Ok(command) => command,
            Err(error) => panic!("communication error: {}", error)
        }
        None => "".to_string()
    }
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


fn print_status(program_name: String, program_version: String, client: &jack::Client, muteable_connections: &Vec<ChangeableConnection>, switchable_connections: &Vec<ConnectionSwitch>, error_messages: &Vec<String>) {
    //print status
    //status muteable connections
    println!("{} {}\n", program_name, program_version);

    if muteable_connections.len() > 0 {
        println!("Muteable:");
        for i in 0..muteable_connections.len() {
            println!("{}: {}", muteable_connections[i].connection.name, 
                if muteable_connections[i].connection.connected(client)
                {"unmuted"} 
                else 
                {"muted"}
            );
        }
        println!("");
    }
    
    //status switchable connections
    for switchable_connection in switchable_connections {
        println!("Switchable:");
        if switchable_connection.connections.len() > 0 {
            if muteable_connections.len() > 0 {
            }
            for i in 0..switchable_connection.connections.len() {
    
                println!("{}: {}", switchable_connection.connections[i].connection.name, 
                    if switchable_connection.connections[i].connection.connected(client)
                    {"active"} 
                    else 
                    {"inactive"}
                );
            }
            println!("");
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



fn get_connections(config: &Config) -> Vec<Connection> {
    let mut connections = Vec::new();
    let connections_def = config.get_connections();

    for connection_def in connections_def {
        connections.push(
            Connection::new(
                &connection_def.name,
                connection_def.connect_init,
                PortConnection::new(
                    &connection_def.port_in, 
                    &connection_def.port_out
                )
            )
        )
    }

    connections
}

fn get_muteable_connections(config: &Config, connections: &Vec<Connection>) -> Vec<ChangeableConnection> {
    let mut muteable_connections = Vec::new();
    let mcs_def = config.get_muteable_connections();

    for mc_def in mcs_def {
        muteable_connections.push(
            ChangeableConnection::new(
                mc_def.shortcut,
                Connection::find_connection_by_name(
                    &mc_def.connection, 
                    connections
                )
            )
        );
    }

    muteable_connections
}

fn get_switchable_connections(config: &Config, connections: &Vec<Connection>) -> Vec<ConnectionSwitch> {
    let mut switchable_connections = Vec::new();
    let scs_def = config.get_switchable_connections();

    for sc_def in scs_def {
        let mut muteable_connections = Vec::new();
        let mcs_def = sc_def.connections;

        for mc_def in mcs_def {
            muteable_connections.push(
                ChangeableConnection::new(
                    mc_def.shortcut,
                    Connection::find_connection_by_name(
                        &mc_def.connection, 
                        connections
                    )
                )
            );
        }
        
        switchable_connections.push(
            ConnectionSwitch{
                standard: Connection::find_connection_by_name(&sc_def.default, connections),
                connections: muteable_connections
            }
        )
    }

    switchable_connections
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