use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
    net::{SocketAddr, ToSocketAddrs},
};

pub struct Config {
    pub domain_mapping: HashMap<Vec<u8>, SocketAddr>,
}

impl Config {
    pub fn new() -> Config {
        let file = File::open("./config.properties").unwrap();
        let reader = io::BufReader::new(file);

        let mut mapping = HashMap::new();

        let comment_chars = ['#', '!'];

        for i in reader.lines().map(|l| l.unwrap()) {
            let first_char = i.chars().next().unwrap();
            if comment_chars.contains(&first_char) {
                continue;
            }

            let i = i.replace(" ", "");

            let mut iter = i.split("=");

            let domain = iter.next().unwrap();
            let socket_address = iter.next().unwrap();

            let socket_addr = socket_address.to_socket_addrs().unwrap().next().unwrap();

            mapping.insert(domain.as_bytes().to_vec(), socket_addr);
        }

        Config {
            domain_mapping: mapping,
        }
    }
}
