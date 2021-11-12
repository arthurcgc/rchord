use rouille::router;

pub fn new_server(hostname: &str, port: u16){
    let mut full_addr = String::from(hostname); full_addr.push_str(":"); full_addr.push_str(&port.to_string());
    // The `start_server` starts listening forever on the given address.
    rouille::start_server(full_addr, move |request| {
        
        router!(request,

            (GET) (/{id: u32}) => {
                println!("u32 {:?}", id);

                // For the same of the example we return an empty response with a 400 status code.
                rouille::Response::text(id.to_string())
            },

            // The code block is called if none of the other blocks matches the request.
            // We return an empty response with a 404 status code.
            _ => rouille::Response::empty_404()
        )
    });
}