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
    let text = load_data("data/dataset/books/fiction/silmarillion/chapter_1.txt").expect("Failed to load data for benchmarking.");
    let vocab_size = 1000;
    let min_pair_freq = 2;

    let DataLoadResult { contents, .. } = text;
    let base_content = contents.join("\n\0\n");

    let mut group = c.benchmark_group("BPE Training");

    group.bench_function("Naive", |b| {
        b.iter(|| {
            let _ = training::naive::bpe_training(black_box(&base_content), black_box(vocab_size), black_box(min_pair_freq));
        });
    });

    group.bench_function("Optimized Pair Counting + Token Sequence Update", |b| {
        b.iter(|| {
            let _ = training::min_alloc::bpe_training(black_box(&base_content), black_box(vocab_size), black_box(min_pair_freq));
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_bpe);
criterion_main!(benches);