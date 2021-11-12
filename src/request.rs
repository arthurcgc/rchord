use reqwest;

pub fn get(host: String, port: u16, key: String) -> String{
    let mut address = String::from("http://");address.push_str(&host);
    // &(BASE_PORT+nearest_node).to_string()
    address.push_str(":"); address.push_str(&port.to_string());
    let mut endpoint = String::from(address); endpoint.push_str("/"); endpoint.push_str(&key);
    // println!("Node #{}: Making request: {}", self.id, endpoint);
    // client connect to address...
    match reqwest::blocking::get(endpoint) {
        Ok(resp) => {
            match resp.text() {
                Ok(body) => {
                    // println!("Node #{}: Asking node #{} for the key...\n response: {}", self.id, nearest_node, body);
                    return body;
                },
                _ => {
                    return String::from("error while parsing body");
                }
            }
        }
        Err(s) => {
            println!("error: {}", s);
            return String::from("error while making request");
        },
    }
}