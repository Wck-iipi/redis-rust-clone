use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn convert_to_vector(content: String) -> Vec<String> {
    let mut vec_string: Vec<String> = Vec::new();
    let mut i = 0;
    for j in 0..content.len() {
        if content.chars().nth(j).unwrap() == '\n' {
            if i != j - 1 {
                //no empty string
                vec_string.push(content[i..j - 1].to_string()); // Ignore \r as well
            }
            i = j + 1;
        }
    }
    vec_string.push(content[i..].to_string());
    println!("vec_string: {:?}", vec_string);
    vec_string
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let _handle = std::thread::spawn(move || {
                    let mut request_buffer = [0; 512];
                    let _request_buffer_size = stream
                        .read(&mut request_buffer)
                        .expect("Could not read from stream");
                    stream.write("+PONG\r\n".as_bytes()).expect("HTTP Response");
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
