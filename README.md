[![Build](https://github.com/1LtFord/1lt_jackmute/actions/workflows/build.yml/badge.svg)](https://github.com/1LtFord/1lt_jackmute/actions/workflows/build.yml)
# 1lt_jackmute
1Lt_Jackmute is a JACK client which enables you to toggle and switch JACK connections via shortcuts.  
I developed this application for my live streaming audio setup, to be able to mute my microphone and system audio via my keyboard whenever i need and to redirect my microphone to different ports for preset voice manipulation.  
![Screenshot_20221015_003857](https://user-images.githubusercontent.com/2715819/195953941-9df133e0-c18c-4bd9-8edb-8f402d648393.png)
  
## How to build this software
1. Download the source code
2. Install **rustup** via your distributions package manager or find instructions [here](https://www.rust-lang.org/tools/install)
3. Build the application with the command `cargo build --release`

## Configuration
### Ports
On the first start of the application, an empty config file is created at ~/.local/share/1lt_software/1lt_jackmute/config  
First, define the Ports of the client.  
Example:  

    [ports1]  
    in = mikro_in  
    out = mikro_out

    [ports2]  
    in = system_in_l  
    out = system_out_l

    [ports3]  
    in = system_in_r  
    out = system_out_r
    

This creates 3 in and 3 out Ports for your client:  
![Screenshot_20221014_211938](https://user-images.githubusercontent.com/2715819/195925033-1a4ce7e2-52bf-4e71-8bba-a41da7a8bd0f.png)

### Connections
Now you need to define every connection between ports your application shall be able to create.  
Example:  

    [connections1]
    name = mikrofon -> 1lt_jackmute
    connect_init = false
    port_out = rode:capture_1
    port_in = 1lt_jackmute:mikro_in
    
    [connections2]
    name = jackmute_mikrofon -> ardour_standard
    connect_init = true
    port_out = 1lt_jackmute:mikro_out
    port_in = ardour:RODE_Podcaster/audio_in 1
    
    [connections3]
    name = ardour_system_L -> 1lt_jackmute
    connect_init = true
    port_out = ardour:OUT-System/audio_out 1
    port_in = 1lt_jackmute:system_in_l
    
*You can find all 12 connections I create in the [config file](https://github.com/1LtFord/1lt_jackmute/blob/main/config) I added to this repository (row 15 to 85)*  
  
`name =` is the name of the connection. We reference them for muting and swithing the connections. They can be whatever you want, but i suggest you choose something describing, like `name of out port` -> `name of in port`
  
`connect_init =` is the default state of the connection. If you want to have this connection active on application startup choose `true` else choose `false`  
  
`port_out =` is the outgoing port of your connection. This is where the audio is comming from. The first part of the port name is the name of the JACK client. If you use QjackCtl you can find the names of the clients at the top of the client boxes in the Graph overview  
![Screenshot_20221014_222012](https://user-images.githubusercontent.com/2715819/195936013-58326263-77b0-4d9f-8584-2fba546fe910.png)  
The second part is the name of the port itself  
![Screenshot_20221014_222459](https://user-images.githubusercontent.com/2715819/195937358-edff1e46-f385-4736-885d-1d52ab3b73de.png)  
Both parts are seperated by a colon. The port_out value of the port of the exaple pictures above would be `1lt_jackmute:mikro_out`  
  
`port_in =` is the ingoing port of your connection. This is where the audio is going.  
For example, if I use the `port_out` value from above and set the value of `port_in` to `ardour:RODE_Podcaster/audio_in 1` the result would be connection with outgoing audio from the `1lt_jackmute` `mirko_out` port to an `ardour` client on port `RODE_Podcaster/audio_in 1`  
![Screenshot_20221014_223446](https://user-images.githubusercontent.com/2715819/195938884-d46a6356-93b6-4bcf-ad46-5894da149d0e.png)
  
  
### Muteable connections
Muteable connections are connections which you can toggle through entering and sending a charakter on the application window.
  
    [muteable_connection1]
    connection = mikrofon -> 1lt_jackmute
    shortcut = m
    
    [muteable_connection2]
    connection = ardour_system_L -> 1lt_jackmute
    shortcut = s
    
    [muteable_connection3]
    connection = ardour_system_R -> 1lt_jackmute
    shortcut = s
  
`connection =` is the name of the connection you defiened earlier  
  
`shortcut =` is the charakter you need to enter into the application to toggle the connection  

For comfort reasons, you might want to add global keyboard shortcuts to your system which sends the shortcut keystrokes to the application window. Some distributions does not support this in default configuration and might need additional software like [xdotool](https://www.semicomplete.com/projects/xdotool/)  
  
  
### Switchable connections
switchable connections are connections you can switch between being active. It has a default connection it fallbacks on if no alternative connection is active and as much alternative connections as you want.
  
    [switchable_connection1]
    default = jackmute_mikrofon -> ardour_standard
    connection1 = 1lt_jackmute -> ardour_effect1
    shortcut1 = 1
    connection2 = 1lt_jackmute -> ardour_effect2
    shortcut2 = 2
    connection3 = 1lt_jackmute -> ardour_effect3
    shortcut3 = 3
    connection4 = 1lt_jackmute -> ardour_effect4
    shortcut4 = 4
  
`default =` is the default connection and active if none of the alternative connections defined under it are active.  
  
alternative connections are defined by a pair of `connectionX =` and `shortcutX =` \(replace X with a number\) almost like a muteable connection. By entering the shortcut charakter you deactivate the default or other alternative connections inside the switchable connection and activate the connection with the same number.  
If the alternative connection you entered the shortcut for is already active, the alternative connection will get deactivated and the default connection goes active.  
  
## Full example configuration
You can find a full example config I actively use on my system [here](https://github.com/1LtFord/1lt_jackmute/blob/main/config)

