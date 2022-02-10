
//! Takes 2 audio inputs and outputs them to 2 audio outputs.
//! All JACK notifications are also printed out.
use std::io;

fn main() {
    // Create client
    let (client, _status) =
        jack::Client::new("1lt_jackmute", jack::ClientOptions::NO_START_SERVER).unwrap();

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

    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
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
    };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    //mikro
    active_client
        .as_client()
        .connect_ports_by_name("rode:capture_1", "1lt_jackmute:mikro_in")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("1lt_jackmute:mikro_out", "ardour:RODE_Podcaster/audio_in 1")
        .unwrap();
    
    //system_l
    active_client
        .as_client()
        .connect_ports_by_name("ardour:OUT-System/audio_out 1", "1lt_jackmute:system_in_l")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("1lt_jackmute:system_out_l", "ardour:System-Stream/audio_in 1")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("1lt_jackmute:system_out_l", "ardour:Master/audio_in 1")
        .unwrap();
    //system_r
    active_client
        .as_client()
        .connect_ports_by_name("ardour:OUT-System/audio_out 2", "1lt_jackmute:system_in_r")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("1lt_jackmute:system_out_r", "ardour:System-Stream/audio_in 2")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("1lt_jackmute:system_out_r", "ardour:Master/audio_in 2")
        .unwrap();


    // Wait for user input to quit
    println!("Press m to mute microphone channel");
    println!("Press s to mute system channel");
    let mut mikro_connected = true;
    let mut system_connected = true;
    let mut mikro_telefon_connected = false;

    #[allow(while_true)]
    while true {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).ok();

        //mikro
        if user_input.contains("m") {
            if mikro_connected && !mikro_telefon_connected {
                active_client
                    .as_client()
                    .disconnect_ports_by_name("rode:capture_1", "1lt_jackmute:mikro_in")
                    .unwrap();
                mikro_connected = false;
            } else {
                active_client
                    .as_client()
                    .connect_ports_by_name("rode:capture_1", "1lt_jackmute:mikro_in")
                    .unwrap();
                mikro_connected = true;
            }
        }


        //system
        if user_input.contains("s") {
            if system_connected {
                //system_l
                active_client
                    .as_client()
                    .disconnect_ports_by_name("ardour:OUT-System/audio_out 1", "1lt_jackmute:system_in_l")
                    .unwrap();
                //system_r
                active_client
                    .as_client()
                    .disconnect_ports_by_name("ardour:OUT-System/audio_out 2", "1lt_jackmute:system_in_r")
                    .unwrap();
                system_connected = false;
            } else {
                //system_l
                active_client
                    .as_client()
                    .connect_ports_by_name("ardour:OUT-System/audio_out 1", "1lt_jackmute:system_in_l")
                    .unwrap();
                //system_r
                active_client
                    .as_client()
                    .connect_ports_by_name("ardour:OUT-System/audio_out 2", "1lt_jackmute:system_in_r")
                    .unwrap();
                system_connected = true;
            }
        }

        //effect Telefon
        if user_input.contains("t") {
            if mikro_telefon_connected {
                active_client
                    .as_client()
                    .disconnect_ports_by_name("1lt_jackmute:mikro_out", "ardour-01:Telefon/audio_in 1")
                    .unwrap();
                active_client
                    .as_client()
                    .connect_ports_by_name("1lt_jackmute:mikro_out", "ardour:RODE_Podcaster/audio_in 1")
                    .unwrap();

                mikro_telefon_connected = false;
            } else {
                active_client
                    .as_client()
                    .disconnect_ports_by_name("1lt_jackmute:mikro_out", "ardour:RODE_Podcaster/audio_in 1")
                    .unwrap();
                active_client
                    .as_client()
                    .connect_ports_by_name("1lt_jackmute:mikro_out", "ardour-01:Telefon/audio_in 1")
                    .unwrap();

                mikro_telefon_connected = true;
            }
        }

        
        
    }
    

    //active_client.deactivate().unwrap();
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }
}