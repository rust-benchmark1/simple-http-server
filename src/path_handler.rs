use std::net::TcpStream;
use std::io::Read;

/// Path handling operations for processing file system requests
/// Processes incoming path traversal requests via TCP
pub fn process_path_stream() -> Result<String, String> {
    let mut stream = match TcpStream::connect("127.0.0.1:8080") {
        Ok(stream) => stream,
        Err(_) => return Err("Failed to connect to socket".to_string()),
    };

    let mut buffer = [0; 1024];
    
    //SOURCE
    match stream.read(&mut buffer) {
        Ok(bytes_received) => {
            let path_data = String::from_utf8_lossy(&buffer[..bytes_received])
                .trim_matches(char::from(0)).to_string();
            
            // Process the path data through the path engine
            match crate::path_engine::handle_path_operations(path_data) {
                Ok(result) => Ok(format!("Path operation completed: {}", result)),
                Err(e) => Err(format!("Path operation failed: {}", e)),
            }
        },
        Err(e) => Err(format!("Failed to receive path data: {}", e)),
    }
} 