use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion,
};

use unq_tiny_llm::{
    loader::{
        load_data,
        DataLoadResult,
    },
    tokenizer::bpe::training
};

fn benchmark_bpe(c: &mut Criterion) {
    let text = load_data("data/dataset/books/hobbit").expect("Failed to load data for benchmarking.");
    let vocab_size = 1000;
    let min_pair_freq = 2;

    let DataLoadResult { contents, .. } = text;
    let base_content = contents.join("\n\0\n");

    c.bench_function("bpe_training", |b| {
        b.iter(|| {
            let _ = training::bpe_training(black_box(&base_content), black_box(vocab_size), black_box(min_pair_freq));
        });
    });

    c.bench_function("bpe_training_pair_count_optimized", |b| {
        b.iter(|| {
            let _ = training::bpe_training_pair_count_optimized(black_box(&base_content), black_box(vocab_size), black_box(min_pair_freq));
        });
    });
}

criterion_group!(benches, benchmark_bpe);
criterion_main!(benches);