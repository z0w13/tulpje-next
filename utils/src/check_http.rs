fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("usage: check-http <method> <url>");
        std::process::exit(64);
    }

    let method = match args[1].to_ascii_uppercase().as_str() {
        "HEAD" => reqwest::Method::HEAD,
        "GET" => reqwest::Method::GET,
        _ => {
            println!("ERROR: Unsupported method, only HEAD/GET are accepted");
            std::process::exit(64);
        }
    };

    let url = &args[2];

    let client = reqwest::blocking::Client::new();
    let req = client
        .request(method, url)
        .build()
        .expect("error building request");

    match client.execute(req) {
        Err(err) => {
            println!("ERROR: {}, {}", url, err);
            std::process::exit(1);
        }
        Ok(result) => {
            if !result.status().is_success() {
                println!("ERROR: {}, status {}", url, result.status());
                std::process::exit(1);
            } else {
                println!("OK");
            }
        }
    }
}
