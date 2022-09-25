use std::hash::{Hash, Hasher};
use std::net::ToSocketAddrs;
use std::{
    collections::{self, hash_map::DefaultHasher},
    net,
    sync::atomic::{self, Ordering},
};
use std::{fs, io};

use super::level::{self};
use super::parser;

pub struct AppState {
    routes: collections::BTreeMap<u64, Balancer>,
}

impl AppState {
    pub fn new(path: &str) -> AppState {
        let file = fs::File::open(path).unwrap();
        let reader = io::BufReader::new(file);
        let parser = parser::Parser::new(reader);
        let root = parser.parse();
        let hosts: Vec<Host> = root
            .struct_list(vec!["hosts"])
            .expect("error parsing hosts");

        let mut routes = collections::BTreeMap::new();
        for host in hosts {
            routes.insert(host.0, host.1);
        }

        AppState { routes }
    }
    pub fn route(&mut self, domain: u64) -> Option<net::SocketAddr> {
        match self.routes.get_mut(&domain) {
            Some(balancer) => Some(balancer.route()),
            None => None,
        }
    }
    pub fn hash(&self, domain: &str) -> u64 {
        hash(domain)
    }
}

struct Balancer {
    counter: atomic::AtomicUsize,
    addrs: Vec<net::SocketAddr>,
}

impl Balancer {
    fn route(&mut self) -> net::SocketAddr {
        let mut counter = self.counter.fetch_add(1, Ordering::Release);
        counter %= self.addrs.len();
        self.addrs[counter]
    }
}

fn hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

struct Host(u64, Balancer);

impl TryFrom<&level::Level> for Host {
    type Error = level::Error;

    fn try_from(level: &level::Level) -> Result<Self, Self::Error> {
        let val = level.field_name(vec![])?;
        let hashed_domain = hash(val);

        let routing = level.list(vec!["routing"])?;

        let addrs: Vec<net::SocketAddr> = routing
            .into_iter()
            .map(|d| {
                let domain: String = d.try_into().unwrap();
                domain.to_socket_addrs().unwrap().next().unwrap()
            })
            .collect();

        let balancer = Balancer {
            counter: atomic::AtomicUsize::new(0),
            addrs,
        };

        Ok(Host(hashed_domain, balancer))
    }
}
