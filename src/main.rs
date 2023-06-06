use std::net::{UdpSocket, Ipv4Addr};
use std::net::{SocketAddr, TcpStream};
use std::sync::{mpsc,Arc,Mutex};
use std::str::FromStr;

use rayon::ThreadPoolBuilder;


use colored::Colorize;



fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let target = dns_resolver(args[1].to_string());
    let start_port = args[2].clone().parse::<u16>().unwrap();
    let end_port = args[3].clone().parse::<u16>().unwrap();
    let n_threads = args[4].clone().parse::<u16>().unwrap();

    let target_arc = Arc::new(Mutex::new(target.clone()));

    println!(
        "{} Scan {} from port {} to {} in {} threads.",
        "[*]".yellow(),
        target,
        start_port,
        end_port,
        n_threads
    );

    let thread_ranges = split_ports(start_port, end_port, n_threads.clone());

    let (sender, receiver) = mpsc::channel();
    let sender = Arc::new(Mutex::new(sender));

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(n_threads.into())
        .build()
        .unwrap();

    thread_ranges.into_iter().for_each(|(start, end)| {
        let sender = Arc::clone(&sender);
        let target_arc = Arc::clone(&target_arc);
        thread_pool.spawn(move || {
            req(target_arc,start, end, sender);
        });
    });

    drop(sender);

    for port in receiver {
        println!("  {} OPEN: {}", "[+]".green(), port);
    }
}

fn split_ports(start_port: u16, end_port: u16, num_threads: u16) -> Vec<(u16, u16)> {
    let total_ports = end_port - start_port + 1;
    let ports_per_thread = total_ports / num_threads as u16;

    (0..num_threads)
        .map(|i| {
            let start = start_port + (i * ports_per_thread);
            let end = start + ports_per_thread - 1;
            (start, end)
        })
        .chain(std::iter::once((
            start_port as u16 + (num_threads * ports_per_thread),
            end_port,
        )))
        .collect()
}

fn req(target:Arc<Mutex<String>>,start: u16, end: u16, sender: Arc<Mutex<mpsc::Sender<u16>>>) {
    let target = target.lock().unwrap().clone();

    for port in start..=end {
        let addr = format!("{}:{}", target, port);

        let addr = SocketAddr::from_str(&addr).unwrap_or_else(|_| panic!("Failed to convert address: {}", addr)); // change parse from here to the main function

        let res = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1));
        if res.is_ok() {
            sender.lock().unwrap().send(port).unwrap();
        }
    }
}

fn dns_resolver(dns: String) -> String {
    

    if Ipv4Addr::from_str(dns.as_str()).is_ok(){
        return dns
    }    
    
    // DNS server address and port
    let server_address = "8.8.8.8";
    let server_port = 53;

    // Create a UDP socket to the DNS server
    let dns_server = format!("{}:{}", server_address, server_port);
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(err) => {
            panic!("Failed to bind UDP socket: {}", err);
        }
    };


    let dns_vec: Vec<&str> = dns.split(".").collect();

    let target = dns_vec[0];
    let target_len = target.len();

    let extension = dns_vec[1];
    let extension_len = extension.len();

    let query: Vec<u8> = Vec::from_iter(
        [
            &[
                0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ][..],
            &[target_len as u8],
            target.as_bytes(),
            &[extension_len as u8],
            extension.as_bytes(),
            &[0x00, 0x00, 0x01, 0x00, 0x01],
        ]
        .concat(),
    );

    // Send the DNS query to the server
    if let Err(err) = socket.send_to(query.as_slice(), dns_server) {
        panic!("Failed to send DNS query: {}", err);
    }

    // Receive the response from the server
    let mut response = [0; 1024];
    let (response_length, _) = match socket.recv_from(&mut response) {
        Ok((length, _)) => (length, ()),
        Err(err) => {
            panic!("Failed to receive DNS response: {}", err);
        }
    };

    let dns_response = &response[..response_length];

    // Extractt IP
    let addr_u8 = &dns_response[dns_response.len() - 4..];
    let mut addr: String = addr_u8.iter().map(|&byte| format!("{}.", byte)).collect();
    addr.pop();

    addr
}
