use nix::unistd::ForkResult::{Parent};
use nix::unistd::{fork};
use nix::unistd::Pid;
use nix::sys::wait::waitpid;
use std::io;
use rouille::router;
use std::collections::HashMap;
mod request;

const BASE_PORT: u16 = 8000;
const N: u16 = 16;
const HOST: &str = "0.0.0.0";

struct Node {
    id: u16,
    port: u16,
    max_peers: i64,
    data: HashMap<String,String>,
    finger_table: Vec<(u16, String)>,
}

impl Node {
    fn to_string(&self) -> String{
        let mut node_str = String::new();
        node_str.push_str("id: "); node_str.push_str(&self.id.to_string());
        node_str.push_str("\n");
        node_str.push_str("port: "); node_str.push_str(&self.port.to_string());
        node_str.push_str("\n");
        node_str.push_str("finger_table: ");
        for (id, port) in &self.finger_table {
            node_str.push_str("(id: "); node_str.push_str(&id.to_string()); node_str.push_str(" ");
            node_str.push_str("port: "); node_str.push_str(port); node_str.push_str(") ");
            node_str.push_str(" ");
        }
        node_str.push_str("\n");

        return node_str;
    }

    fn am_i_the_node(&self, port: u16) -> bool{
        // println!("DEBUG: comparing-> self.id = {}, searching for: {} result = {}", self.id, id, self.id == id);
        return self.port == port
    }

    fn clone(&self) -> Self {
        return Node{
            id: self.id,
            port: self.port,
            max_peers: self.max_peers,
            data: self.data.clone(),
            finger_table: self.finger_table.clone(),
        };
    }

    fn new_server(&self, hostname: &str){
        let mut full_addr = String::from(hostname); full_addr.push_str(":"); full_addr.push_str(&self.port.to_string());
        println!("Starting node #{} on {}", self.id,  full_addr);
        let node_clone = self.clone();
        // The `start_server` starts listening forever on the given address.
        rouille::start_server(full_addr, move |request| {
            router!(request,
                (GET) (/{key: String}) => {
                    let target_port = node_clone.lookup(key.to_string());
                    if target_port < 0 {
                        return rouille::Response::text("not found\n");
                    }
                    if node_clone.am_i_the_node(target_port as u16) {
                        println!("hey you found me! Here are my specs:\n{}", &node_clone.to_string());
                        match node_clone.data.get(&key.to_string()) {
                            Some(value) => {
                                return rouille::Response::text(value);
                            },
                            _ => {
                                let mut resp_text = String::from("key should be inside node with port: ");
                                resp_text.push_str(&target_port.to_string());
                                resp_text.push_str(" but was not found\n");
                                return rouille::Response::text("not found\n");
                            },
                        }      
                    }

                    println!("making last request: http://{}:{}/{}", HOST, target_port, key);
                    let result = request::get(HOST.to_string(), target_port as u16, key);
                    if result == "error while parsing body" || result == "error while making request" {
                        println!("{}", result);
                        return rouille::Response::text("not found\n")
                    }

                    rouille::Response::text(&*result)
                },

                _ => rouille::Response::empty_404()
            )
        });
    }

    // lookup returns the target port that is closer or is the data holder of the key
    fn lookup(&self, key: String) -> i16{
        println!("Node #{}: calculating which node is holding the {} key...", self.id, key);
        let hash = gen_hash(key.clone());
        let node_id: u16 = hash.parse().unwrap();
        println!("Node #{}: generated hash = {}; target node_id = {}", self.id, hash, node_id);

        if node_id == self.id {
            return self.port as i16
        }

        // println!("Node #{}: I'm not the node you're looking for\nHere are my specs: {}\n searching for the right node...", self.id, self.to_string());
        let mut nearest_node = 0;
        let base: u16 = 2;
        for i in 0.. self.finger_table.len(){
            let n = (self.id + base.pow(i as u32)) % N;
            println!("Node #{}: n = {}",self.id, n);

            if n == node_id {
                let port = (self.finger_table[i].1).parse().unwrap();
                println!("Node #{}: Found the node inside my finger table!\ntarget port: {}", self.id, port);
                return port;
            }

            if n > node_id {
                break;
            }

            nearest_node = n;
        }

        let result = request::get(HOST.to_string(), BASE_PORT+nearest_node, key);
        if result == "error while parsing body" || result == "error while making request" {
            println!("{}", result);
            return -1
        }

        match result.parse() {
            Ok(num) => {
                return num
            },
            _ => return -1
        }

        // let mut address = String::from("http://");address.push_str(HOST);
        // address.push_str(":"); address.push_str(&(BASE_PORT+nearest_node).to_string());
        // let mut endpoint = String::from(address); endpoint.push_str("/"); endpoint.push_str(&key.to_string());
        // println!("Node #{}: Making request: {}", self.id, endpoint);
        // // client connect to address...
        // match reqwest::blocking::get(endpoint) {
        //     Ok(resp) => {
        //         match resp.text() {
        //             Ok(body) => {
        //                 println!("Node #{}: Asking node #{} for the key...\n response: {}", self.id, nearest_node, body);
        //                 return body.parse().unwrap();
        //             },
        //             _ => {}
        //         }
        //     }
        //     Err(s) => {
        //         println!("Node #{}: Asking node #{} for the key...\n response: {}", self.id, nearest_node, s);
        //     },
        // }
    }
}

fn gen_hash(key: String) -> String{
    let mut byte_sum: u16 = 0;
    for byte in key.bytes(){ byte_sum+=byte as u16; }
    let hash = byte_sum % (2^N);
    return hash.to_string();
}


fn start_network() {
    println!("Starting the chord network with {} nodes...", N);
    let mut jobs: Vec<Pid> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();

    for i in 0..N {
        let node = create_node(i);
        nodes.push(node.clone());
        let job = fork();
        match job.expect("Fork failed: Unable to create child process"){
            Parent { child } => {
                jobs.push(child);
                continue;
            },
            _ => { 
                // child
                node.new_server(HOST);
            },
        }
    }

    loop {
        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        let trimmed_command = command.trim();
        if trimmed_command.eq("quit") {
            break;
        }
    }

    println!("Waiting for children to terminate...");
    for j in jobs {
        waitpid(j, None).unwrap();
    }
}

fn create_node(id: u16) -> Node{
    return node(id, BASE_PORT + id as u16);
}

fn node(id: u16, port: u16) -> Node{
    let mut max_peers: f64 = N as f64;
    max_peers = max_peers.log2();
    let mut finger_table: Vec<(u16, String)> = Vec::new();
    let base: u16 = 2;
    for i in 0..max_peers as u16 {
        let x = (id + base.pow(i as u32) ) % N ;
        let nbr = x as u16 + BASE_PORT; // nbr = neighboor
        finger_table.push((x as u16, nbr.to_string()));
    }
    let empty_data: HashMap<String, String> = HashMap::new();
    return Node {
        id,
        finger_table,
        port,
        data: empty_data,
        max_peers: max_peers as i64,
    };
}

fn main() {
    start_network();
}