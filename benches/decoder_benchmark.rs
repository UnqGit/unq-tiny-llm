use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

use unq_tiny_llm::tokenizer::{
    bpe::serialization::load_tokenizer,
    decoding::decode,
    serialization::load_encoded,
};

fn benchmark_decoder(c: &mut Criterion) {
    let tokenizer = load_tokenizer("data/training/tokenizer.json")
        .expect("Unable to open tokenizer file");

    let encoded = load_encoded("data/training/encoder.json")
        .expect("Unable to open encoded data");

    c.bench_function("Decoder", |b| {
        b.iter(|| {
            let _ = decode(black_box(&encoded), black_box(&tokenizer));
        });
    });
}

criterion_group!(benches, benchmark_decoder);
criterion_main!(benches);