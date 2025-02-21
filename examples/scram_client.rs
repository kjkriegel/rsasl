use rsasl::mechname::Mechname;
use rsasl::property::{AuthId, Password};
use rsasl::session::Step::{Done, NeedsMore};
use rsasl::SASL;

use std::io;
use std::io::Cursor;
use std::sync::Arc;

// A SCRAM-SHA-1 authentication exchange.
//
// Run both this and the `scram_server` example to pass data to and fro

pub fn main() {
    // Create an untyped SASL because we won't store/retrieve information in the context since
    // we don't use callbacks.
    let sasl = SASL::new();

    // Usually you would first agree on a mechanism with the server, for demostration purposes
    // we directly start a SCRAM-SHA-1 "exchange"
    let mut session = sasl
        .client_start(Mechname::new(b"SCRAM-SHA-1").unwrap())
        .unwrap();

    // Read the "authcid" from stdin
    let mut username = String::new();
    println!("Enter username to encode for SCRAM-SHA-1 auth:");
    if let Err(error) = io::stdin().read_line(&mut username) {
        println!("error: {}", error);
        return;
    }
    username.pop(); // Remove the newline char at the end of the string

    // Read the "password" from stdin
    println!("\nEnter password to encode for SCRAM-SHA-1 auth:");
    let mut password = String::new();
    if let Err(error) = io::stdin().read_line(&mut password) {
        println!("error: {}", error);
        return;
    }
    password.pop(); // Remove the newline char at the end of the string
    print!("\n");

    // Set the username that will be used in the SCRAM-SHA-1 authentication
    session.set_property::<AuthId>(Arc::new(username));

    // Now set the password that will be used in the SCRAM-SHA-1 authentication
    session.set_property::<Password>(Arc::new(password));

    let mut data: Option<Box<[u8]>> = None;
    loop {
        let mut out = Cursor::new(Vec::new());
        // Do an authentication step. In a SCRAM-SHA-1 exchange there is only one step, with no data.
        let step_result = session.step64(data.as_ref(), &mut out);

        match step_result {
            Ok(Done(Some(_len))) => {
                let buffer = out.into_inner();
                let s = std::str::from_utf8(&buffer);
                println!("Done: {:?}", s);
                break;
            }
            Ok(Done(None)) => {
                println!("Done, but mechanism wants to send no further data to the other party");
            }
            Ok(NeedsMore(Some(_len))) => {
                let buffer = out.into_inner();
                let s = std::str::from_utf8(&buffer);
                println!("Data to send: {:?}", s.as_ref());

                let mut in_data = String::new();
                if let Err(error) = io::stdin().read_line(&mut in_data) {
                    println!("error: {}", error);
                    return;
                }
                in_data.pop(); // Remove the newline char at the end of the string

                data = Some(in_data.into_boxed_str().into_boxed_bytes());
            }
            Ok(NeedsMore(None)) => {
                println!("Needs more data, but mechanism wants to send no further data to the other party");
            }
            Err(e) => {
                println!("{}", e);
                return;
            }
        }
    }
}
