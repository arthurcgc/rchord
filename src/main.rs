use nix::unistd::ForkResult::{Parent};
use nix::unistd::{fork};
use nix::unistd::Pid;
use nix::sys::wait::waitpid;
use std::io;
mod server;

const BASE_PORT: u16 = 8000;
const N: i64 = 16;
const HOST: &str = "0.0.0.0";

struct Node {
    id: i64,
    port: u16,
    max_peers: i64,
    finger_table: Vec<String>,
}

fn start_node(node: Node){
    println!("Starting node #{} at {}:{}...", node.id, HOST, node.port);

    server::new_server(HOST, node.port);
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
                start_node(node.clone());
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

fn create_node(id: i64) -> Node{
    return node(id, BASE_PORT + id as u16);
}

fn node(id: i64, port: u16) -> Node{
    let mut max_peers: f64 = N as f64;
    max_peers = max_peers.log2();
    let mut finger_table: Vec<String> = Vec::new();
    for i in 0..max_peers as i64 {
        let x = (id + (2^i)) % N ;
        let nbr = x as u16 + BASE_PORT; // nbr = neighboor
        finger_table.push(nbr.to_string());
    }
    return Node {
        id,
        finger_table,
        port,
        max_peers: max_peers as i64,
    };
}

impl Node {
    fn am_i_the_node(&self, id: i64) -> bool{
        return self.id == id
    }

    fn clone(&self) -> Self {
        return Node{
            id: self.id,
            port: self.port,
            max_peers: self.max_peers,
            finger_table: self.finger_table.clone(),
        };
    }
}

fn main() {
    start_network();
}