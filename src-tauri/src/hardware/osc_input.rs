use std::net::UdpSocket;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Clone, Debug, PartialEq)]
pub struct OscLearnEvent {
    pub address: String,
    pub args: Vec<rosc::OscType>,
}

#[derive(Debug)]
pub struct OscInputManager {
    thread_handle: Option<thread::JoinHandle<()>>,
    shutdown_tx: Option<Sender<()>>,
}

impl OscInputManager {
    pub fn new() -> Self {
        Self {
            thread_handle: None,
            shutdown_tx: None,
        }
    }

    pub fn start_listening(
        &mut self,
        bind_host: &str,
        port: u16,
    ) -> Result<Receiver<OscLearnEvent>, String> {
        self.stop_listening();

        let bind_addr = format!("{}:{}", bind_host, port);
        let socket = UdpSocket::bind(&bind_addr)
            .map_err(|e| format!("Failed to bind OSC listener on {}: {}", bind_addr, e))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set nonblocking: {}", e))?;

        let (tx, rx) = channel();
        let (shutdown_tx, shutdown_rx) = channel::<()>();

        let mut buf = [0u8; 65535];
        let handle = thread::Builder::new()
            .name("osc-input-listener".to_string())
            .spawn(move || loop {
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }
                match socket.recv_from(&mut buf) {
                    Ok((size, _addr)) => {
                        if let Ok((_rest, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                            let events = extract_osc_events(&packet);
                            for event in events {
                                if tx.send(event).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            })
            .map_err(|e| format!("Failed to spawn OSC thread: {}", e))?;

        self.thread_handle = Some(handle);
        self.shutdown_tx = Some(shutdown_tx);

        Ok(rx)
    }

    pub fn stop_listening(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn is_listening(&self) -> bool {
        self.thread_handle.is_some()
    }
}

fn extract_osc_events(packet: &rosc::OscPacket) -> Vec<OscLearnEvent> {
    match packet {
        rosc::OscPacket::Message(msg) => {
            vec![OscLearnEvent {
                address: msg.addr.clone(),
                args: msg.args.clone(),
            }]
        }
        rosc::OscPacket::Bundle(bundle) => {
            let mut events = Vec::new();
            for sub_packet in &bundle.content {
                events.extend(extract_osc_events(sub_packet));
            }
            events
        }
    }
}

impl Default for OscInputManager {
    fn default() -> Self {
        Self::new()
    }
}
