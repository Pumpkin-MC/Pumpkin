use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::io::{self};

use client::interrupted;
use configuration::BasicConfiguration;
use server::Server;

// Setup some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(0);

pub mod client;
pub mod configuration;
pub mod entity;
pub mod protocol;
pub mod server;
pub mod util;
pub mod world;

#[cfg(not(target_os = "wasi"))]
fn main() -> io::Result<()> {
    use std::{collections::HashMap, rc::Rc};

    use client::Client;
    use configuration::AdvancedConfiguration;

    let basic_config = BasicConfiguration::load("configuration.toml");

    let advanced_configuration = AdvancedConfiguration::load("features.toml");

    simple_logger::SimpleLogger::new().init().unwrap();

    // Create a poll instance.
    let mut poll = Poll::new()?;
    // Create storage for events.
    let mut events = Events::with_capacity(128);

    // Setup the TCP server socket.

    let addr = format!(
        "{}:{}",
        basic_config.server_address, basic_config.server_port
    )
    .parse()
    .unwrap();

    let mut listener = TcpListener::bind(addr)?;

    // Register the server with poll we can receive events for it.
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    // Unique token for each incoming connection.
    let mut unique_token = Token(SERVER.0 + 1);

    let mut connections: HashMap<Token, Client> = HashMap::new();

    log::info!("You now can connect to the server");

    let mut server = Server::new((basic_config, advanced_configuration));

    loop {
        if let Err(err) = poll.poll(&mut events, None) {
            if interrupted(&err) {
                continue;
            }
            return Err(err);
        }

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    // Received an event for the TCP server socket, which
                    // indicates we can accept an connection.
                    let (mut connection, address) = match listener.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // If we get a `WouldBlock` error we know our
                            // listener has no more incoming connections queued,
                            // so we can return to polling and wait for some
                            // more.
                            break;
                        }
                        Err(e) => {
                            // If it was any other kind of error, something went
                            // wrong and we terminate with an error.
                            return Err(e);
                        }
                    };

                    log::info!("Accepted connection from: {}", address);

                    let token = next(&mut unique_token);
                    poll.registry().register(
                        &mut connection,
                        token,
                        Interest::READABLE.add(Interest::WRITABLE),
                    )?;

                    connections.insert(token, Client::new(Rc::new(token), connection));
                },

                token => {
                    // Maybe received an event for a TCP connection.
                    let done = if let Some(client) = connections.get_mut(&token) {
                        client.poll(&mut server, event).unwrap();
                        client.closed
                    } else {
                        // Sporadic events happen, we can safely ignore them.
                        false
                    };
                    if done {
                        if let Some(mut client) = connections.remove(&token) {
                            poll.registry().deregister(&mut client.connection)?;
                        }
                    }
                }
            }
        }
    }
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

#[cfg(target_os = "wasi")]
fn main() {
    panic!("can't bind to an address with wasi")
}
