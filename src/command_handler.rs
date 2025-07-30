use std::net::UdpSocket;

/// Command handling operations for processing system commands
/// Processes incoming command injection requests via UDP
pub fn process_command_stream() -> Result<String, String> {
    let socket = match UdpSocket::bind("127.0.0.1:0") {
        Ok(socket) => socket,
        Err(_) => return Err("Failed to bind socket".to_string()),
    };

    let mut buffer = [0; 1024];
    
    //SOURCE
    match socket.recv(&mut buffer) {
        Ok(bytes_received) => {
            let command_data = String::from_utf8_lossy(&buffer[..bytes_received])
                .trim_matches(char::from(0)).to_string();
            
            // Process the command data through the command engine
            match crate::command_engine::handle_command_operations(command_data) {
                Ok(result) => Ok(format!("Command operation completed: {}", result)),
                Err(e) => Err(format!("Command operation failed: {}", e)),
            }
        },
        Err(e) => Err(format!("Failed to receive command data: {}", e)),
    }
} 