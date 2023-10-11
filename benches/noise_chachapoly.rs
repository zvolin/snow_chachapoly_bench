// TOML:
//
// [package]
// name = "snow_bench"
// version = "0.1.0"
// edition = "2021"
//
// [dependencies]
// anyhow = "1.0"
// snow = "0.9"
//
// [dev-dependencies]
// criterion = "0.5"
//
// [[bench]]
// name = "noise_chachapoly"
// harness = false

use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use snow::{params::NoiseParams, Builder, TransportState};

const MAX_MSG_SIZE: usize = u16::MAX as usize - 1;

fn handshake() -> Result<(TransportState, TransportState)> {
    let noise_config: NoiseParams = "Noise_NN_25519_ChaChaPoly_SHA256".parse()?;
    let mut initiator = Builder::new(noise_config.clone()).build_initiator()?;
    let mut responder = Builder::new(noise_config).build_responder()?;

    let mut i_buf = [0u8; MAX_MSG_SIZE];
    let mut r_buf = [0u8; MAX_MSG_SIZE];

    // initiator -> responder
    let written = initiator.write_message(&[], &mut i_buf)?;
    responder.read_message(&i_buf[..written], &mut r_buf)?;

    // responder -> initiator
    let written = responder.write_message(&[], &mut r_buf)?;
    initiator.read_message(&r_buf[..written], &mut i_buf)?;

    let initiator = initiator.into_transport_mode()?;
    let responder = responder.into_transport_mode()?;

    Ok((initiator, responder))
}

pub fn chacha(c: &mut Criterion) {
    let (mut peer1, _) = handshake().unwrap();

    let data = vec![0u8; 5 * 1024 * 1024];

    c.bench_function("chachapoly", move |b| {
        b.iter(|| {
            let mut buf = [0u8; MAX_MSG_SIZE];
            // 16 bytes for noise data
            for chunk in black_box(data.clone()).chunks(MAX_MSG_SIZE - 16) {
                peer1.write_message(chunk, &mut buf).unwrap();
            }
        })
    });
}

criterion_group!(benches, chacha);
criterion_main!(benches);
