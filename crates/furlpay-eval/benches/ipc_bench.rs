use criterion::{black_box, criterion_group, criterion_main, Criterion};
use furlpay_ipc::protocol::{SearchRequestMessage, SearchResponseMessage, StatusCode};
use furlpay_ipc::ring_buffer::{RingConsumer, RingProducer, RingBufferHeader};

fn bench_ipc(c: &mut Criterion) {
    let mut group = c.benchmark_group("furlpay_eval_ipc_throughput");

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
        let msg = [0x55u8; 128];

        group.bench_function("ring_buffer_zero_copy_roundtrip", |b| {
            b.iter(|| {
                producer.try_write(black_box(&msg));
                let read = consumer.try_read();
                black_box(read)
            });
        });
    }

    group.bench_function("protocol_request_creation", |b| {
        b.iter(|| {
            let mut req = SearchRequestMessage::new();
            req.request_id = black_box(101);
            req.set_query(black_box("fn process_transfer"));
            black_box(req)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_ipc);
criterion_main!(benches);
