use std::{fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

fn get_request(mut stream: &TcpStream) -> Vec<String> {
    let buf_reader = BufReader::new(&mut stream);

    buf_reader
        .lines()
        .filter_map(|result| result.ok())
        .take_while(|line| !line.is_empty())
        .collect()
}

enum HttpResponse
{
    Ok(String),
    BadRequest(String),
    InternalServerError(String),
    NotFound(String)
}

impl HttpResponse {
    fn into_string(&self) -> String {
        let (status_line, contents) = match self {
            HttpResponse::Ok(contents) => ("HTTP/1.1 200 OK", contents),
            HttpResponse::BadRequest(contents) => ("HTTP/1.1 400 Bad Request", contents),
            HttpResponse::InternalServerError(contents) => ("HTTP/1.1 500 Internal Server Error", contents),
            HttpResponse::NotFound(contents) => ("HTTP/1.1 404 Not Found", contents),
        };

        let length = contents.len();
        format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: text/html\r\n\r\n{contents}")
    }
}

fn get_file_content(file_name: &str) -> Option<String> {
    let exe_path = std::env::current_exe().ok()?;
    let mut assets_path = exe_path.parent()?.to_path_buf();
    assets_path.push(file_name);

    fs::read_to_string(assets_path).ok()
}

fn get_not_found_response() ->  HttpResponse {
    match get_file_content("404.html") {
        Some(content) => HttpResponse::NotFound(content),
        _ => return get_internal_server_error_response()
    }
}

fn get_bad_request_response() -> HttpResponse {
    HttpResponse::BadRequest("Bad Request".into())
} 

fn get_internal_server_error_response() -> HttpResponse {
    HttpResponse::InternalServerError("Internal Error".into())
}

fn process_request(http_request: &[String], path: &str) -> HttpResponse {
    let request_line = match http_request.first() {
        Some(request_line) if request_line.starts_with("GET") && request_line.ends_with("HTTP/1.1") => request_line,
        _ => return get_bad_request_response() 
    };

    if !request_line.eq_ignore_ascii_case(&format!("GET {path} HTTP/1.1")) {
        return get_not_found_response();
    }

    match get_file_content("hello.html") {
        Some(contents) => HttpResponse::Ok(contents),
        _ => get_internal_server_error_response()
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let http_request = get_request(&stream);

            let response = process_request(&http_request, "/");
            stream.write_all(response.into_string().as_bytes()).unwrap();    
        }
    }
}
