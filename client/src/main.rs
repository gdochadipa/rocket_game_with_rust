// Langkah 4b/4c: client yang sama untuk native DAN browser (WASM).
//
// Satu-satunya perbedaan antar platform ada di struct `Net`:
//   - native : mesin koneksi Matchbox jalan di thread tokio (seperti sebelumnya)
//   - WASM   : tidak ada thread, jadi future Matchbox di-poll manual tiap frame
//              (Matchbox sendiri menyebut cara ini valid: "polled manually,
//              i.e. once per frame")
//
// Game loop di main() IDENTIK di kedua platform.
//
// CATATAN: bagian WASM ini hipotesis yang belum teruji. Itulah gunanya spike.

use macroquad::logging::{error, info};
use macroquad::prelude::*;
use matchbox_socket::{PeerState, WebRtcSocket};

#[cfg(target_arch = "wasm32")]
use matchbox_socket::MessageLoopFuture;

// Pakai 127.0.0.1 (bukan localhost) untuk menghindari masalah IPv6 di macOS.
// Semua client (native dan browser) harus memakai URL room yang sama persis.
const ROOM_URL: &str = "ws://127.0.0.1:3536/spike_room";

struct Net {
    socket: WebRtcSocket,
    alive: bool,
    #[cfg(target_arch = "wasm32")]
    loop_fut: MessageLoopFuture,
}

impl Net {
    // ---------- native ----------
    #[cfg(not(target_arch = "wasm32"))]
    fn new(room_url: &str) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let url = room_url.to_string();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("gagal membuat runtime tokio");
            rt.block_on(async move {
                let (socket, loop_fut) = WebRtcSocket::new_unreliable(url);
                tx.send(socket).expect("thread game sudah berhenti");

                match loop_fut.await {
                    Ok(()) => println!("Message loop selesai (socket ditutup)"),
                    Err(e) => eprintln!("Message loop ERROR: {e:?}"),
                }
            });
        });

        let socket = rx.recv().expect("gagal menerima socket dari thread jaringan");
        Net { socket, alive: true }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn pump(&mut self) {
        // Tidak ada yang perlu dilakukan: thread tokio sudah menjalankan mesinnya.
    }

    // ---------- WASM ----------
    #[cfg(target_arch = "wasm32")]
    fn new(room_url: &str) -> Self {
        let (socket, loop_fut) = WebRtcSocket::new_unreliable(room_url);
        Net { socket, alive: true, loop_fut }
    }

    #[cfg(target_arch = "wasm32")]
    fn pump(&mut self) {
        use std::future::Future;
        use std::task::{Context, Poll};

        if !self.alive {
            return;
        }
        // Waker "kosong": kita tidak menunggu dibangunkan, karena kita
        // memang akan mem-poll lagi di frame berikutnya.
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        if let Poll::Ready(result) = self.loop_fut.as_mut().poll(&mut cx) {
            error!("Message loop berhenti: {:?}", result);
            self.alive = false;
        }
    }
}

/// Format paket: 8 byte = x (f32) + y (f32), little-endian.
fn encode_position(pos: Vec2) -> Box<[u8]> {
    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&pos.x.to_le_bytes());
    bytes.extend_from_slice(&pos.y.to_le_bytes());
    bytes.into_boxed_slice()
}

fn decode_position(packet: &[u8]) -> Option<Vec2> {
    if packet.len() != 8 {
        return None;
    }
    let x = f32::from_le_bytes(packet[0..4].try_into().ok()?);
    let y = f32::from_le_bytes(packet[4..8].try_into().ok()?);
    Some(vec2(x, y))
}

#[macroquad::main("Spike Matchbox")]
async fn main() {
    let mut net = Net::new(ROOM_URL);

    let mut my_pos = vec2(100.0, 100.0);
    let mut other_pos: Option<Vec2> = None;
    let mut send_timer = 0.0_f32;
    const SEND_INTERVAL: f32 = 1.0 / 20.0; // 20 Hz

    loop {
        let dt = get_frame_time();

        // --- Input lokal ---
        let speed = 200.0 * dt;
        if is_key_down(KeyCode::Right) { my_pos.x += speed; }
        if is_key_down(KeyCode::Left)  { my_pos.x -= speed; }
        if is_key_down(KeyCode::Down)  { my_pos.y += speed; }
        if is_key_down(KeyCode::Up)    { my_pos.y -= speed; }

        // --- Jaringan ---
        net.pump(); // WASM: jalankan mesin koneksi satu langkah. Native: no-op.

        if net.alive {
            match net.socket.try_update_peers() {
                Ok(changes) => {
                    for (peer, state) in changes {
                        match state {
                            PeerState::Connected => info!("Peer terhubung: {:?}", peer),
                            PeerState::Disconnected => {
                                info!("Peer keluar: {:?}", peer);
                                other_pos = None;
                            }
                        }
                    }

                    for (_peer, packet) in net.socket.channel_mut(0).receive() {
                        if let Some(pos) = decode_position(&packet) {
                            other_pos = Some(pos);
                        }
                    }

                    send_timer += dt;
                    if send_timer >= SEND_INTERVAL {
                        send_timer = 0.0;
                        let peers: Vec<_> = net.socket.connected_peers().collect();
                        for peer in peers {
                            net.socket.channel_mut(0).send(encode_position(my_pos), peer);
                        }
                    }
                }
                Err(_) => {
                    error!("Socket sudah tertutup");
                    net.alive = false;
                    other_pos = None;
                }
            }
        }

        // --- Gambar ---
        clear_background(BLACK);
        draw_rectangle(my_pos.x, my_pos.y, 30.0, 30.0, GREEN);
        if let Some(p) = other_pos {
            draw_rectangle(p.x, p.y, 30.0, 30.0, RED);
        }
        let status = if !net.alive {
            "Terputus dari jaringan"
        } else if other_pos.is_some() {
            "Terhubung"
        } else {
            "Menunggu pemain lain..."
        };
        draw_text(status, 10.0, 20.0, 20.0, WHITE);

        next_frame().await;
    }
}
