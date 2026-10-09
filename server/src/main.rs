use std::time::Duration;

use futures::{select, FutureExt};
use futures_timer::Delay;
use matchbox_socket::{PeerState, WebRtcSocket};


#[tokio::main]
async fn main(){

// socket -> remote control
// loop_fut -> mesin yg mengurus koneksi, ga ada ini  ga ada pesan yg terkirim
    let (mut socket, loop_fut) = WebRtcSocket::new_reliable("ws://localhost:3536/spike_room");

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(100));
        futures::pin_mut!(timeout);

        loop {
            for (peer, state) in socket.update_peers() {
                match state {
                    PeerState::Connected =>{
                        println!("Peer terhubung: {peer:?}");

                        // pesan di matchbox selalu terpisah sih
                        // berupa bytes mentah (Box<[u8]>).
                        let packet = "halo dari native".as_bytes().to_vec().into_boxed_slice();
                        socket.channel_mut(0).send(packet, peer);
                    }
                    PeerState::Disconnected => {
                        println!("Peer keluar: {peer:?}");
                    }
                }
            }

            for (peer, packet) in socket.channel_mut(0).receive() {
                let text = String::from_utf8_lossy(&packet);
                println!("Pesan dari {peer:?}: {text}");
            }

            select! {
                _ = (&mut timeout).fuse() => {
                    timeout.reset(Duration::from_millis(100));
                }
                res = &mut loop_fut => {
                    println!("Socket ditutup. {res:?}");
                    break;
                }
            }
        }

}
