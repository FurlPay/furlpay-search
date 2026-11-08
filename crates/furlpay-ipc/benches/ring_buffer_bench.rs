use criterion::{black_box, criterion_group, criterion_main, Criterion};
use furlpay_ipc::protocol::{SearchRequestMessage, SearchResponseMessage, StatusCode};
use furlpay_ipc::ring_buffer::{RingConsumer, RingProducer, RingBufferHeader};

fn bench_ring_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("furlpay_ipc_ring_buffer");

    // 1. Serialization / struct assembly benchmark
    group.bench_function("request_message_assembly", |b| {
        b.iter(|| {
            let mut req = SearchRequestMessage::new();
            req.request_id = black_box(42);
            req.set_query(black_box("handle_payment"));
            black_box(req)
        });
    });

    group.bench_function("response_message_assembly", |b| {
        b.iter(|| {
            let mut res = SearchResponseMessage::new(black_box(42));
            res.elapsed_micros = 268;
            res.status_code = StatusCode::Success as u16;
            res.total_hits = 1;
            black_box(res)
        });
    });

    // 2. Ring buffer push and pop benchmark
    let slot_size = 512;
    let slot_count = 64;
    let capacity = slot_size * slot_count;
    let total_size = std::mem::size_of::<RingBufferHeader>() + capacity;
    let mut buffer = vec![0u8; total_size + 64];
    let base = buffer.as_mut_ptr();
    let aligned = ((base as usize + 63) & !63) as *mut u8;

    unsafe {
        let producer = RingProducer::new(aligned, capacity, slot_size);
        let consumer = RingConsumer::new(aligned as *const u8, capacity, slot_size);
        let msg = [42u8; 256];

        group.bench_function("spsc_ring_buffer_roundtrip", |b| {
            b.iter(|| {
                producer.try_write(black_box(&msg));
                let read = consumer.try_read();
                black_box(read)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_ring_buffer);
criterion_main!(benches);
