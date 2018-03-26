extern crate tokio;
#[macro_use]
extern crate futures;
extern crate bytes;

use tokio::io;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude;
use futures::sync::mpsc;
use futures::future::{self, Either};
use futures::stream;
use bytes::{BytesMut, Bytes, BufMut};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

// shorthand for the transmit half of the message channel
type Tx = mpsc::UnboundedSender<Bytes>;
type Rx = mpsc::UnboundedReceiver<Bytes>;


#[derive(Debug)]
struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

fn main() {
    let state = Arc::new(Mutex::new(Shared::new())));
    let addr = "127.0.0.1".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener.incoming().for_each(move |socket| {
        // TODO: process socket
        process(socket, state.clone());
        Ok(())
    })
    .map_err(|err| {
        // handle error by printing to stdout
        println!("accept error = {:?}", err);
    });

    println!("server running on localhost:6142");

    // start the server 
    //
    // start the tokio runtime(reactor, threadpool, etc...)
    // spawns the 'server' task onto the runtime
    // blocks the current thread until the runtime becomes idle,
    // i.e. all spawned tasks have completed.
    tokio::run(server);
}

fn process(socket: TcpStream, state: Arc<Mutex<Shared>>)  {
    // define the task that processes the connection.
    let task = unimplemented!();

    // spawn the task
    tokio::spawn(task);
}

#[derive(Debug)]
struct Lines {
    socket: TcpStream,
    rd: BytesMut,
    wr: BytesMut,
}

impl Lines {
    /// create a new 'Lines' codec backed by the socket
    fn new(socket: TcpStream) -> Self {
        Lines {
            socket,
            rd: BytesMut::new(),
            wr: BytesMut::new(),
        }
    }
}

impl Stream for Lines {
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut: self) -> Result<Async<Option<Self::Item>>, Self::Error {
        // first, read any new data that might have been received
        // off the socket
        //
        // we track if the socket is closed here and will be used to 
        // inform the return value below
        let sock_closed = self.fill_read_buf()?.is_ready();

        // now, try finding lines
        let pos = self.rd.windows(2).enumerate()
            .find(|&(_, bytes)| bytes == b"\r\n")
            .map(|(i, _)| i);
        
        if let Some(pos) = pos {
            // remove the line from the read buffer and set it
            // to 'line'.
            let mut line = self.rd.split_to(pos + 2);

            // drop the trailing \r\n
            line.split_off(pos);

            // return the line
            return Ok(Async::Ready(Some(line)));
        }

        if sock_closed {
            Ok(Async::Ready(None))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl Lines {
    fn fill_read_buf(&mut self) -> Result<Async<()>, io::Error> {
        loop {
            // ensure the read buffer has capacity.
            // 
            // this might result in an internal allocation.
            self.rd.reverse(1024);

            // read data into the buffer
            //
            // the 'read_buf' fn is provided by 'AsyncRead'
            let n == try_ready!(self.socket.read_buf(&mut self.rd));

            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }
}