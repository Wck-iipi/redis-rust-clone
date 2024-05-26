use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
};

enum RedisTypes {
    BulkString(String),
    Integer(i32),
    List(Vec<RedisTypes>),
    SimpleString(String),
    Error(String),
}

fn convert_string_to_redis_types(content: String) -> RedisTypes {
    let first_char = content.chars().nth(0).unwrap();

    match first_char {
        '+' => {
            // Simple String
            let simple_string: String = content
                .strip_prefix("+")
                .unwrap()
                .strip_suffix("\r\n")
                .unwrap()
                .to_string();
            RedisTypes::SimpleString(simple_string)
        }
        '$' => {
            // String
            let string: String = content
                .strip_prefix("$")
                .unwrap()
                .split("\r\n")
                .collect::<Vec<&str>>()
                .get(1)
                .unwrap()
                .to_string();
            RedisTypes::BulkString(string)
        }
        ':' => {
            // Integer
            let integer: i32 = content
                .strip_prefix(":")
                .unwrap()
                .split("\r\n")
                .collect::<Vec<&str>>()
                .get(1)
                .unwrap()
                .parse::<i32>()
                .unwrap();
            RedisTypes::Integer(integer)
        }
        '*' => {
            // Array
            println!("Content: {}", content);
            let first_rn = content.find("\r\n").unwrap();
            // Everything to the right of the first \r\n
            let arr_content = content[first_rn + 2..].split("\r\n").collect::<Vec<&str>>();
            println!("Array Content: {:?}", arr_content);

            let mut array: Vec<RedisTypes> = vec![];

            for i in 0..arr_content.len() as i32 {
                let first_char = arr_content.get(i as usize).unwrap().chars().nth(0).unwrap();
                if first_char == ':' {
                    let integer: RedisTypes = convert_string_to_redis_types(
                        arr_content.get(i as usize).unwrap().to_string() + "\r\n",
                    );
                    array.push(integer);
                } else if first_char == '$' {
                    let total_string = arr_content.get(i as usize).unwrap().to_string()
                        + "\r\n"
                        + arr_content.get(i as usize + 1).unwrap()
                        + "\r\n";
                    println!("Total String: {}", total_string);
                    let string: RedisTypes = convert_string_to_redis_types(total_string);

                    array.push(string);
                }
            }

            RedisTypes::List(array)
        }
        _ => RedisTypes::Error("Invalid request".to_string()),
    }
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn response_redis_type(
    redis_type: RedisTypes,
    hashmap: HashMap<String, String>,
) -> (Option<String>, Option<(String, String)>) {
    match redis_type {
        RedisTypes::BulkString(content) => {
            if content == "PING" {
                return (Some("+PONG\r\n".to_string()), None);
            } else {
                println!("Bulk String: {}", content);
            }
            return (None, None);
        }
        RedisTypes::Integer(content) => {
            println!("Integer: {}", content);
            return (None, None);
        }
        RedisTypes::SimpleString(content) => {
            if content == "PING" {
                return (Some("+PONG\r\n".to_string()), None);
            } else {
                println!("Simple String: {}", content);
            }
            return (None, None);
        }
        RedisTypes::Error(content) => {
            println!("Error: {}", content);
            return (None, None);
        }
        RedisTypes::List(content) => {
            println!("Running in list");
            for r in &content {
                print_type_of(&r);
            }
            println!("Content size: {}", content.len());

            let first_element = content.get(0).unwrap();

            match first_element {
                RedisTypes::BulkString(content_string) => {
                    println!("Content String: {}", content_string);
                    if content_string == "ECHO" {
                        if let RedisTypes::BulkString(content_echoed) = content.get(1).unwrap() {
                            println!("Echoed: {}", content_echoed);
                            return (
                                Some(format!(
                                    "${}\r\n{}\r\n",
                                    content_echoed.len(),
                                    content_echoed
                                )),
                                None,
                            );
                        }
                        return (None, None);
                    } else if content_string == "SET" {
                        println!("Setting value");
                        if let RedisTypes::BulkString(key) = content.get(1).unwrap() {
                            if let RedisTypes::BulkString(value) = content.get(2).unwrap() {
                                return (
                                    Some("+OK\r\n".to_string()),
                                    Some((key.clone(), value.clone())),
                                );
                            }
                        }
                        return (None, None);
                    } else if content_string == "GET" {
                        if let RedisTypes::BulkString(key) = content.get(1).unwrap() {
                            if let Some(value) = hashmap.get(key) {
                                return (Some(format!("${}\r\n{}\r\n", value.len(), value)), None);
                            }
                        }
                        return (None, None);
                    } else {
                        return response_redis_type(
                            RedisTypes::BulkString(content_string.clone()),
                            hashmap,
                        );
                    }
                }
                RedisTypes::List(_) => (None, None),
                RedisTypes::Integer(integer) => {
                    return response_redis_type(RedisTypes::Integer(*integer), hashmap)
                }
                RedisTypes::SimpleString(string) => {
                    return response_redis_type(RedisTypes::SimpleString(string.clone()), hashmap);
                }
                RedisTypes::Error(_) => (None, None),
            }

            // if let RedisTypes::BulkString(content_string) = content.get(0).unwrap() {
            //     println!("Content String: {}", content_string);
            //     if content_string == "ECHO" {
            //         if let RedisTypes::BulkString(content_echoed) = content.get(1).unwrap() {
            //             println!("Echoed: {}", content_echoed);
            //             return (Some(format!("{}\r\n", content_echoed)), None);
            //         }
            //     } else if content_string == "SET" {
            //         println!("Setting value");
            //         if let RedisTypes::BulkString(key) = content.get(1).unwrap() {
            //             if let RedisTypes::BulkString(value) = content.get(2).unwrap() {
            //                 return (
            //                     Some("+OK\r\n".to_string()),
            //                     Some((key.clone(), value.clone())),
            //                 );
            //             }
            //         }
            //     } else if content_string == "GET" {
            //         if let RedisTypes::BulkString(key) = content.get(1).unwrap() {
            //             if let Some(value) = hashmap.get(key) {
            //                 return (Some(format!("${}\r\n{}\r\n", value.len(), value)), None);
            //             }
            //         }
            //     }
            // }
            // return (None, None);
        }
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let _handle = std::thread::spawn(move || {
                    let mut global_state: HashMap<String, String> = HashMap::new();
                    let mut request_buffer = [0; 512];
                    loop {
                        let read_count = stream.read(&mut request_buffer).expect("HTTP Request");
                        if read_count == 0 {
                            break;
                        }

                        println!(
                            "Request: {:?}",
                            String::from_utf8(request_buffer.to_vec()).unwrap()
                        );
                        let request: RedisTypes = convert_string_to_redis_types(
                            String::from_utf8(request_buffer.to_vec()).unwrap(),
                        );
                        let (response, gs) = response_redis_type(request, global_state.clone());

                        if let Some((key, value)) = gs {
                            global_state.insert(key, value);
                        }

                        stream
                            .write(
                                response
                                    .expect("Cannot get response from the server")
                                    .as_bytes(),
                            )
                            .expect("HTTP Response");
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
