#![forbid(unsafe_code)]

use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use std::io::copy;

pub fn run_proxy(port: u32, destination: String) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

    for stream in listener.incoming() {
        let client = Arc::new(stream.unwrap());
        let server = Arc::new(TcpStream::connect(destination.as_str()).unwrap());
        handle_connection(Arc::clone(&client), Arc::clone(&server));
        handle_connection(Arc::clone(&server), Arc::clone(&client));
    }

    fn handle_connection(client: Arc<TcpStream>, server: Arc<TcpStream>) {
        thread::spawn(move || {
            copy(&mut client.as_ref(), &mut server.as_ref()).unwrap();
        });
    }
}
