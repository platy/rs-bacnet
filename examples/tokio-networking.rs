extern crate tokio_core;
extern crate env_logger;
extern crate futures;
extern crate bacnet;

use std::net::SocketAddr;
use std::str;
use std::io;

use futures::{Future, Stream, Sink};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use bacnet::bacnet_ip::{BipCodec, VLLFrame};
use bacnet::network;
use bacnet::serialise;
use bacnet::ast::ApduHeader;

fn main() {
    drop(env_logger::init());

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr1: SocketAddr = "127.0.0.1:47808".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:47809".parse().unwrap();

    // Bind both our sockets and then figure out what ports we got.
    let a = UdpSocket::bind(&addr1, &handle).unwrap();
    let b = UdpSocket::bind(&addr2, &handle).unwrap();
    let b_addr = b.local_addr().unwrap();

    // We're parsing each socket with the `BipCodec`, and then we
    // `split` each socket into the sink/stream halves.
    let (a_sink_t, a_stream) = a.framed(BipCodec).split();
    let (b_sink_t, b_stream) = b.framed(BipCodec).split();

    let a_sink = a_sink_t.with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[a] sending: {:?}\n", &msg);
        Ok((addr, VLLFrame::OriginalUnicastNPDU(msg)))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[a] sending: {:?}", &msg);
        let mut buf = Vec::new();
        network::encode(msg, &mut buf);
        Ok((addr, buf))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[a] sending: {:?}", &msg);
        Ok((addr, network::NetworkRequest::expect_reply(msg)))
    }).with(move |(addr, apdu_header)| -> Result<_, io::Error> {
        println!("[a] sending: {:?}", &apdu_header);
        let mut buf = vec![];
        serialise::write_apdu_header(&mut buf, apdu_header);
        Ok((addr, buf))
    });

    let b_sink = b_sink_t.with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[b] sending: {:?}\n", &msg);
        Ok((addr, VLLFrame::OriginalUnicastNPDU(msg)))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[b] sending: {:?}", &msg);
        let mut buf = Vec::new();
        network::encode(msg, &mut buf);
        Ok((addr, buf))
    }).with(move |(addr, msg)| -> Result<_, io::Error> {            
        println!("[b] sending: {:?}", &msg);
        Ok((addr, network::NetworkRequest::expect_reply(msg)))
    }).with(move |(addr, apdu_header)| -> Result<_, io::Error> {
        println!("[b] sending: {:?}", &apdu_header);
        let mut buf = vec![];
        serialise::write_apdu_header(&mut buf, apdu_header);
        Ok((addr, buf))
    });;  
    // Start off by sending a ping from a to b, afterwards we just print out
    // what they send us and continually send pings
    // let pings = stream::iter((0..5).map(Ok));
    let a = a_sink.send((b_addr, ApduHeader::UnconfirmedReq { service: 0 })).and_then(|a_sink| {
        let mut i = 0;
        let a_stream = a_stream.take(4).map(move |(addr, msg)| {
            i += 1;
            println!("[a] recv: {:?}", &msg);
            (addr, ApduHeader::UnconfirmedReq { service: 0 })
        });
        a_sink.send_all(a_stream)
    });

    // The second client we have will receive the pings from `a` and then send
    // back pongs.
    let b_stream = b_stream.and_then(|(addr, msg)| {
        println!("[b] recv: {:?}", &msg);
        if let VLLFrame::OriginalUnicastNPDU(data) = msg {
            Ok((addr, try!(network::decode(data.as_slice()))))
        } else {
            panic!("Unsupported frame {:?}", msg);
        }
    }).map(|(addr, msg)| {
        println!("[b] recv: {:?}", &msg);
        (addr, ApduHeader::UnconfirmedReq { service: 1 })
    }).or_else(|err| {
        println!("[b] error: {}", err);
        Err(err)
    });
    let b = b_sink.send_all(b_stream);

    // Spawn the sender of pongs and then wait for our pinger to finish.
    handle.spawn(b.then(|_| Ok(())));
    drop(core.run(a));
}

