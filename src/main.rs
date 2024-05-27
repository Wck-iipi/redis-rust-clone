use std::{
    collections::HashMap,
    fmt::format,
    io::{Read, Write},
    net::TcpListener,
    time::SystemTime,
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
    hashmap: HashMap<String, (String, Option<i32>, Option<SystemTime>)>,
) -> (Option<String>, Option<(String, String)>, Option<i32>) {
    match redis_type {
        RedisTypes::BulkString(content) => {
            if content == "PING" {
                return (Some("+PONG\r\n".to_string()), None, None);
            } else {
                println!("Bulk String: {}", content);
            }
            return (None, None, None);
        }
        RedisTypes::Integer(content) => {
            println!("Integer: {}", content);
            return (None, None, None);
        }
        RedisTypes::SimpleString(content) => {
            if content == "PING" {
                return (Some("+PONG\r\n".to_string()), None, None);
            } else {
                println!("Simple String: {}", content);
            }
            return (None, None, None);
        }
        RedisTypes::Error(content) => {
            println!("Error: {}", content);
            return (None, None, None);
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

                    if content_string.to_lowercase() == "echo" {
                        if let RedisTypes::BulkString(content_echoed) = content.get(1).unwrap() {
                            println!("Echoed: {}", content_echoed);
                            return (
                                Some(format!(
                                    "${}\r\n{}\r\n",
                                    content_echoed.len(),
                                    content_echoed
                                )),
                                None,
                                None,
                            );
                        }
                        return (None, None, None);
                    } else if content_string == "SET" {
                        println!("Setting value");
                        if let RedisTypes::BulkString(key) = content.get(1).unwrap() {
                            if let RedisTypes::BulkString(value) = content.get(2).unwrap() {
                                if content.get(3).is_none() {
                                    return (
                                        Some("+OK\r\n".to_string()),
                                        Some((key.clone(), value.clone())),
                                        None,
                                    );
                                } else {
                                    if let RedisTypes::BulkString(flag) = content.get(3).unwrap() {
                                        if flag.to_lowercase() == "px" {
                                            if let RedisTypes::BulkString(int) =
                                                content.get(4).unwrap()
                                            {
                                                println!("Int: {}", int);
                                                return (
                                                    Some("+OK\r\n".to_string()),
                                                    Some((key.clone(), value.clone())),
                                                    Some(int.parse::<i32>().unwrap().clone()),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return (None, None, None);
                    } else if content_string == "GET" {
                        if let RedisTypes::BulkString(key) = content.get(1).unwrap() {
                            if let Some((value, time, system_time)) = hashmap.get(key) {
                                if time.is_none() {
                                    return (
                                        Some(format!("${}\r\n{}\r\n", value.len(), value.clone())),
                                        None,
                                        None,
                                    );
                                } else {
                                    let current_time = SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                        as i32;
                                    let system_time_at_call = system_time
                                        .unwrap()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                        as i32;
                                    println!("Time: {:?}", time);
                                    println!("Start time: {:?}", system_time_at_call);
                                    println!("current_time: {:?}", current_time);

                                    if current_time <= system_time_at_call + time.unwrap() {
                                        return (
                                            Some(format!(
                                                "${}\r\n{}\r\n",
                                                value.len(),
                                                value.clone()
                                            )),
                                            None,
                                            None,
                                        );
                                    }
                                }
                            }
                        }
                        return (Some("$-1\r\n".to_string()), None, None);
                    } else if content_string == "INFO" {
                        let replice_of = std::env::args().nth(4);
                        if Some(x) = replice_of {
                            return (Some(format!("$11\r\nrole:slave\r\n")), None, None);
                        } else {
                            return (Some(format!("$11\r\nrole:master\r\n")), None, None);
                        }
                    } else {
                        return response_redis_type(
                            RedisTypes::BulkString(content_string.clone()),
                            hashmap,
                        );
                    }
                }
                RedisTypes::List(_) => (None, None, None),
                RedisTypes::Integer(integer) => {
                    return response_redis_type(RedisTypes::Integer(*integer), hashmap)
                }
                RedisTypes::SimpleString(string) => {
                    return response_redis_type(RedisTypes::SimpleString(string.clone()), hashmap);
                }
                RedisTypes::Error(_) => (None, None, None),
            }
        }
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let port = std::env::args().nth(2).unwrap_or("6379".to_string());
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let _handle = std::thread::spawn(move || {
                    let mut global_state: HashMap<
                        String,
                        (String, Option<i32>, Option<SystemTime>),
                    > = HashMap::new();
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
                        let (response, gs, time) =
                            response_redis_type(request, global_state.clone());

                        if let Some(time) = time {
                            if let Some((key, value)) = gs {
                                global_state
                                    .insert(key, (value, Some(time), Some(SystemTime::now())));
                            }
                        } else {
                            if let Some((key, value)) = gs {
                                global_state.insert(key, (value, None, None));
                            }
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
