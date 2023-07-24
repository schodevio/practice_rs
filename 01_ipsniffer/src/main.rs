use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::ops::RangeInclusive;
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Instant;

use clap::Parser;

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s.parse().map_err(|_| format!("`{s}` isn't a port number"))?;

    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(String::from("invalid port"))
    }
}

fn scan(tx: Sender<u16>, start_port: u16, address: IpAddr, threads: u16) {
    let mut port: u16 = start_port + 1;

    loop {
        match TcpStream::connect((address, port)) {
            Ok(_) => {
                print!(".");

                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            },
            Err(_) => {}
        };

        if (65535 - port) <= threads { break }

        port += threads;
    }
}

#[derive(Parser, Debug)]
#[clap(author, version)]
pub struct Args {
    #[arg(short('a'), long("address"))]
    address: IpAddr,

    #[arg(short('s'), long("start"), default_value="1", value_parser = port_in_range)]
    start_port: Option<u16>,

    #[arg(short('e'), long("end"), default_value="65535", value_parser = port_in_range)]
    end_port: Option<u16>,

    #[arg(short('t'), long("threads"), default_value="4")]
    threads: u16,
}

fn main() {
    let args = Args::parse();
    let time = Instant::now();

    let threads = args.threads;
    let address = args.address;

    let (tx, rx) = channel();

    for i in 0..threads {
        let tx = tx.clone();

        thread::spawn(move || {
            scan(tx, i, address, threads)
        });
    }

    let mut out = vec![];
    drop(tx);

    for p in rx {
        out.push(p);
    }

    out.sort();
    println!("");

    for v in out {
        println!("{} is open", v);
    }

    println!("Elapsed: {:?}", time.elapsed());
}
