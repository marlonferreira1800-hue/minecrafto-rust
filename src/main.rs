use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use minecrafto_rust::world::{CHUNKS_X, CHUNKS_Z, CHUNK_SIZE, WORLD_X, WORLD_Y, WORLD_Z};

fn handle_client(mut stream: std::net::TcpStream) {
    let mut buf = [0u8; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("/");

    if path == "/minecraft_rust.wasm" || path.starts_with("/minecraft_rust.wasm?") {
        if let Ok(wasm_bytes) = fs::read("web/minecraft_rust.wasm") {
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/wasm\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
                wasm_bytes.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&wasm_bytes);
            return;
        }
    }

    if path == "/api/status" {
        let json = format!(
            "{{\"engine\":\"minecrafto-rust\",\"target\":\"wasm32-unknown-unknown + WebGL2\",\"chunks\":\"{}x{}\",\"chunk_size\":{},\"world_dimensions\":[{},{},{}],\"total_voxels\":{}}}",
            CHUNKS_X,
            CHUNKS_Z,
            CHUNK_SIZE,
            WORLD_X,
            WORLD_Y,
            WORLD_Z,
            WORLD_X * WORLD_Y * WORLD_Z
        );
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
            json.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(json.as_bytes());
        return;
    }

    if let Ok(html_bytes) = fs::read("web/index.html") {
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-cache\r\n\r\n",
            html_bytes.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(&html_bytes);
    } else {
        let msg = b"Minecraft Rust WebGL2 Engine Loading...";
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
            msg.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(msg);
    }
}

fn main() {
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).expect("Failed to bind to port 8080");
    println!("============================================================");
    println!("  MINECRAFTO-RUST (Java Edition Reverse-Engineered Engine)  ");
    println!("  Target: Rust -> WebAssembly (wasm32) + WebGL2             ");
    println!("  Listening on http://localhost:8080                        ");
    println!("============================================================");

    for stream in listener.incoming().flatten() {
        thread::spawn(|| handle_client(stream));
    }
}
