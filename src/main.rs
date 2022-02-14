use std::io;

fn main() {

    let connections = get_connections();
    let mut muteable_connections = get_muteable_connections(&connections);
    let mut effect_connections = get_effect_connections(&connections);
    let mut error_messages = Vec::new();
    
    //init and get jack client
    let active_client = get_jack_client();


    //connect all connections which should be connected on startup
    error_messages.append(&mut init_connections(&connections, active_client.as_client()));

    let mut run = true;
    #[allow(while_true)]
    while run {

        //Print connection status and error messages
        print_status(&muteable_connections, &effect_connections, &error_messages);
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

            //active_client.as_client() testen für zugriff auf ports

            //toggle muteable connections (mute-unmute)
            for i in 0..muteable_connections.len() {
                if user_input.contains(muteable_connections[i].shortcut) {
                    if muteable_connections[i].connection.connected {
                        match active_client
                        .as_client()
                        .disconnect_ports_by_name(&muteable_connections[i].connection.port_connections.audio_in, &muteable_connections[i].connection.port_connections.audio_out) {
                            Ok(()) => (),
                            Err(_err) => error_messages.push(format!("could not disconnect {}", &muteable_connections[i].connection.name))
                        }
                        muteable_connections[i].connection.connected = false;
                    } else {
                        match active_client
                        .as_client()
                        .connect_ports_by_name(&muteable_connections[i].connection.port_connections.audio_in, &muteable_connections[i].connection.port_connections.audio_out) {
                            Ok(()) => (),
                            Err(_err) => error_messages.push(format!("could not connect {}", &muteable_connections[i].connection.name))
                        }
                        muteable_connections[i].connection.connected = true;
                    }
                }
            }
    
            for i in 0..effect_connections.connections.len() {
                if user_input.contains(effect_connections.connections[i].shortcut) {
                    //disconnect if connected
                    if effect_connections.connections[i].connection.connected {
                        match active_client
                        .as_client()
                        .disconnect_ports_by_name(&effect_connections.connections[i].connection.port_connections.audio_in, &effect_connections.connections[i].connection.port_connections.audio_out) {
                            Ok(()) => (),
                            Err(_err) => error_messages.push(format!("could not disconnect {}", &effect_connections.connections[i].connection.name))
                        }
                        effect_connections.connections[i].connection.connected = false;
                        //connect standard
                        if !effect_connections.standard.connected {
                            match active_client
                            .as_client()
                            .connect_ports_by_name(&effect_connections.standard.port_connections.audio_in, &effect_connections.standard.port_connections.audio_out) {
                                Ok(()) => (),
                                Err(_err) => error_messages.push(format!("could not connect {}", &effect_connections.standard.name))
                            }
                            effect_connections.standard.connected = true;
                        }
                    //Connect if disconnected
                    } else {
                        for f in 0..effect_connections.connections.len() {
                            if !user_input.contains(effect_connections.connections[f].shortcut) {
                                //disconnect other connected effects
                                if effect_connections.connections[f].connection.connected {
                                    match active_client
                                    .as_client()
                                    .disconnect_ports_by_name(&effect_connections.connections[f].connection.port_connections.audio_in, &effect_connections.connections[f].connection.port_connections.audio_out) {
                                        Ok(()) => (),
                                        Err(_err) => error_messages.push(format!("could not disconnect {}", &effect_connections.connections[f].connection.name))
                                    }
                                    effect_connections.connections[f].connection.connected = false;
                                }
                            }
                        }
                        //disconnect standard
                        if effect_connections.standard.connected {
                            match active_client
                            .as_client()
                            .disconnect_ports_by_name(&effect_connections.standard.port_connections.audio_in, &effect_connections.standard.port_connections.audio_out) {
                                Ok(()) => (),
                                Err(_err) => error_messages.push(format!("could not disconnect {}", &effect_connections.standard.name))
                            }
                            effect_connections.standard.connected = false;
                        }

                        match active_client
                        .as_client()
                        .connect_ports_by_name(&effect_connections.connections[i].connection.port_connections.audio_in, &effect_connections.connections[i].connection.port_connections.audio_out) {
                            Ok(()) => (),
                            Err(_err) => error_messages.push(format!("could not connect {}", &effect_connections.connections[i].connection.name))
                        }
                        effect_connections.connections[i].connection.connected = true;
                    }
                }
            }
        }
    }
    

    active_client.deactivate().unwrap();
}

fn get_jack_client() -> jack::AsyncClient<Notifications, jack::ClosureProcessHandler<impl FnMut(&jack::Client, &jack::ProcessScope)-> jack::Control>> {
    //Create Client
    let (client, _status) = jack::Client::new("1lt_jackmute", jack::ClientOptions::NO_START_SERVER).unwrap();
    let process = jack::ClosureProcessHandler::new(get_jack_process_callback(&client));

    // Activate the client, which starts the processing.
    client.activate_async(Notifications, process).unwrap()
}


fn get_jack_process_callback(client: &jack::Client) -> impl FnMut(&jack::Client, &jack::ProcessScope) -> jack::Control {
     // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let mikro_in = client
        .register_port("mikro_in", jack::AudioIn::default())
        .unwrap();
    let system_in_l = client
        .register_port("system_in_l", jack::AudioIn::default())
        .unwrap();
    let system_in_r = client
        .register_port("system_in_r", jack::AudioIn::default())
        .unwrap();
    let mut mikro_out = client
        .register_port("mikro_out", jack::AudioOut::default())
        .unwrap();
    let mut system_out_l = client
        .register_port("system_out_l", jack::AudioOut::default())
        .unwrap();
    let mut system_out_r = client
        .register_port("system_out_r", jack::AudioOut::default())
        .unwrap();

    move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        //mikro
        let mikro_out_p = mikro_out.as_mut_slice(ps);
        let mikro_in_p = mikro_in.as_slice(ps);
        mikro_out_p.clone_from_slice(mikro_in_p);

        //system_l
        let system_out_p_l = system_out_l.as_mut_slice(ps);
        let system_in_p_l = system_in_l.as_slice(ps);
        system_out_p_l.clone_from_slice(system_in_p_l);

        //system_r
        let system_out_p_r = system_out_r.as_mut_slice(ps);
        let system_in_p_r = system_in_r.as_slice(ps);
        system_out_p_r.clone_from_slice(system_in_p_r);


        jack::Control::Continue
    }
}

fn init_connections(connections: &Vec<Connection>, active_client: &jack::Client) -> Vec<String> {
    let mut error_messages = Vec::new();
    for connection in connections {
        if connection.connected == true {
            match active_client
            .connect_ports_by_name(&connection.port_connections.audio_in, &connection.port_connections.audio_out) {
                Ok(()) => (),
                Err(_err) => error_messages.push(format!("could not connect {}", &connection.name))
            }
        }
    }
    error_messages
}


fn print_status(muteable_connections: &Vec<ChangeableConnection>, effect_connections: &ConnectionSwitch, error_messages: &Vec<String>) {
    //print status
    if muteable_connections.len() > 0 {
        for i in 0..muteable_connections.len() {
            println!("{}: {}", muteable_connections[i].connection.name, 
                if muteable_connections[i].connection.connected 
                {"unmuted"} 
                else 
                {"muted"}
            );
        }
    }
    
    if effect_connections.connections.len() > 0 {
        if muteable_connections.len() > 0 {
            println!("");
        }
        for i in 0..effect_connections.connections.len() {

            println!("{}: {}", effect_connections.connections[i].connection.name, 
                if effect_connections.connections[i].connection.connected 
                {"active"} 
                else 
                {"inactive"}
            );
        }
    }
    
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



#[derive(Clone)]
struct PortConnection {
    audio_out: String,
    audio_in: String
}
impl PortConnection {
    fn new(paudio_in: &str, paudio_out: &str) -> PortConnection {
        PortConnection{
            audio_in: String::from(paudio_in),
            audio_out: String::from(paudio_out)
        }
    }
}

#[derive(Clone)]
struct Connection {
    name: String,
    connected: bool,
    port_connections: PortConnection
}
impl Connection {
    fn new(pname: &str, pconnected: bool, pport_connections: PortConnection) -> Connection {
        Connection{
            name: String::from(pname),
            connected: pconnected,
            port_connections: pport_connections
        }
    }

    fn find_connection_by_name(name: &str, connections: &Vec<Connection>) -> Connection {
        let mut iter = connections.iter();
        iter.find(|&x| x.name == name).unwrap().clone()
    }
}

#[derive(Clone)]
struct ChangeableConnection {
    shortcut: char,
    connection: Connection
}
impl ChangeableConnection {
    fn new(pshortcut: char, pconnection: Connection) -> ChangeableConnection{
        ChangeableConnection {
            shortcut: pshortcut,
            connection: pconnection
        }
    }
}

#[derive(Clone)]
struct ConnectionSwitch {
    standard: Connection,
    connections: Vec<ChangeableConnection>
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