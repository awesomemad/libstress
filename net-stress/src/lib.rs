//! Loopback TCP stress — localhost only for safety.

use anyhow::Result;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use stress_core::{Intensity, run_timed};
use tracing::info;

const DEFAULT_PORT: u16 = 19456;

/// Saturate loopback with echo-style TCP traffic.
pub fn run(
    intensity: Intensity,
    duration: Duration,
    port: u16,
    threads: usize,
) -> Result<()> {
    let port = if port == 0 { DEFAULT_PORT } else { port };
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    info!(%addr, threads, "starting loopback net stress");

    let stop = Arc::new(AtomicBool::new(false));
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    let stop_srv = stop.clone();
    let srv = std::thread::spawn(move || {
        while !stop_srv.load(Ordering::Relaxed) {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_nonblocking(false);
                let _ = std::thread::spawn(move || echo_client(&mut stream));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });

    let payload_size = intensity.scale_usize(32 * 1024).max(4096);
    let buf = vec![0x5Au8; payload_size];

    run_timed(duration, |sig| {
        if sig.should_stop() {
            return;
        }
        rayon_like_clients(addr, &buf, threads.min(32).max(1));
    });

    stop.store(true, Ordering::Relaxed);
    let _ = srv.join();
    Ok(())
}

fn echo_client(stream: &mut TcpStream) {
    let mut tmp = [0u8; 8192];
    while let Ok(n) = stream.read(&mut tmp) {
        if n == 0 {
            break;
        }
        let _ = stream.write_all(&tmp[..n]);
    }
}

fn rayon_like_clients(addr: SocketAddr, buf: &[u8], threads: usize) {
    std::thread::scope(|s| {
        for _ in 0..threads {
            let buf = buf.to_vec();
            s.spawn(move || {
                if let Ok(mut stream) = TcpStream::connect(addr) {
                    for _ in 0..64 {
                        let _ = stream.write_all(&buf);
                        let mut discard = [0u8; 1024];
                        let _ = stream.read(&mut discard);
                    }
                }
            });
        }
    });
}
