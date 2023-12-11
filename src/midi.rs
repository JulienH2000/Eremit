use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::result::Result as StdResult;

pub fn _setup_midi() -> StdResult<MidiOutputConnection, Box<dyn Error>> {
    let midi_out = MidiOutput::new("My Test Output")?;
    // Get an output port (read from console if multiple are available)
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => return Err("no output port found".into()),
        1 => {
            println!(
                "Choosing the only available output port: {}",
                midi_out.port_name(&out_ports[0]).unwrap()
            );
            &out_ports[0]
        }
        _ => {
            println!("\nAvailable output ports:");
            for (i, p) in out_ports.iter().enumerate() {
                println!("{}: {}", i, midi_out.port_name(p).unwrap());
            }
            print!("Please select output port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            out_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid output port selected")?
        }
    };
    println!("\nOpening connection");
    let conn_out = midi_out.connect(out_port, "midir-test")?;
    // Return the conn_out
  Ok(conn_out)
}

pub fn setup_midi_connection() -> MidiOutputConnection {
    loop {
        match _setup_midi() {
            Ok(connection) => return connection,
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}