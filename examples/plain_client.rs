use rsasl::mechname::Mechname;
use rsasl::property::{AuthId, Password};
use rsasl::session::Step::{Done, NeedsMore};
use rsasl::SASL;
use std::io;
use std::io::Cursor;
use std::sync::Arc;

pub fn main() {
    // Create an untyped SASL because we won't store/retrieve information in the context since
    // we don't use callbacks.
    let sasl = SASL::new();

    // Usually you would first agree on a mechanism with the server, for demostration purposes
    // we directly start a PLAIN "exchange"
    let mut session = sasl.client_start(Mechname::new(b"PLAIN").unwrap()).unwrap();

    // Read the "authcid" from stdin
    let mut username = String::new();
    println!("Enter username to encode for PLAIN auth:");
    if let Err(error) = io::stdin().read_line(&mut username) {
        println!("error: {}", error);
        return;
    }

    // Read the "password" from stdin
    println!("\nEnter password to encode for PLAIN auth:");
    let mut password = String::new();
    if let Err(error) = io::stdin().read_line(&mut password) {
        println!("error: {}", error);
        return;
    }
    print!("\n");

    // Set the username that will be used in the PLAIN authentication
    session.set_property::<AuthId>(Arc::new(username));

    // Now set the password that will be used in the PLAIN authentication
    session.set_property::<Password>(Arc::new(password));

    // Do an authentication step. In a PLAIN exchange there is only one step, with no data.
    let mut out = Cursor::new(Vec::new());
    let data: Option<&[u8]> = None;
    let step_result = session.step(data, &mut out).unwrap();

    match step_result {
        Done(Some(_)) => {
            let buffer = out.into_inner();
            println!("Encoded bytes: {:?}", buffer);
            println!("As string: {:?}", std::str::from_utf8(&buffer.as_ref()));
        }
        Done(None) => {
            panic!("PLAIN exchange produced no output")
        }
        NeedsMore(_) => assert!(false, "PLAIN exchange took more than one step"),
    }
}
