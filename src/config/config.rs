use std::cell;
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

#[derive(Debug)]
pub struct AppState {
    routes: collections::BTreeMap<u64, Balancer>,
    pub addr: String,
    pub thread: usize,
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

        let addr: String = root
            .value(vec!["server", "addr"])
            .unwrap()
            .try_into()
            .unwrap();
        let thread: i64 = root
            .value(vec!["server", "thread"])
            .unwrap()
            .try_into()
            .unwrap();
        let thread: usize = thread.try_into().unwrap();

        let mut routes = collections::BTreeMap::new();
        for host in hosts {
            routes.insert(host.0, host.1);
        }

        AppState {
            routes,
            addr,
            thread,
        }
    }
    pub fn route(&self, domain: u64) -> Option<net::SocketAddr> {
        match self.routes.get(&domain) {
            Some(balancer) => Some(balancer.route()),
            None => None,
        }
    }
    pub fn hash(&self, domain: &str) -> u64 {
        hash(domain)
    }
}

#[derive(Debug)]
struct Balancer {
    counter: atomic::AtomicUsize,
    addrs: Vec<net::SocketAddr>,
    #[cfg(debug_assertions)]
    domain:String
}

impl Balancer {
    fn route(&self) -> net::SocketAddr {
        let mut counter = self.counter.fetch_add(1, Ordering::Release);
        counter %= self.addrs.len();
        self.addrs[counter]
    }
}

pub fn hash<T>(obj: T) -> u64
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
        let hashed_domain = hash(val.as_bytes());

        let routing = level.list(vec!["routing"])?;

        let addrs: Vec<net::SocketAddr> = routing
            .into_iter()
            .map(|d| {
                let domain: String = d.try_into().unwrap();
                domain
                    .to_socket_addrs()
                    .expect(&format!("fail parsing domain {:?}", domain))
                    .next()
                    .unwrap()
            })
            .collect();

        let balancer = Balancer {
            counter: atomic::AtomicUsize::new(0),
            addrs,
            #[cfg(debug_assertions)]
            domain:val.to_string()
        };

        Ok(Host(hashed_domain, balancer))
    }
}
