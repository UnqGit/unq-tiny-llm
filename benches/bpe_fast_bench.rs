use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main, black_box};
use unq_tiny_llm::loader::load_data;
use unq_tiny_llm::tokenizer::bpe::training::optimized;
use unq_tiny_llm::tokenizer::preprocessing::preprocess_text;

static DATA_SIZES: &[usize] = &[
    1_000_000,
    5_000_000,
    10_000_000,
    15_000_000,
    20_000_000,
];

fn extract_data(base_content: &str, size: usize) -> &str {
    let max_bytes = base_content.len().min(size);
    let boundary = base_content.floor_char_boundary(max_bytes);
    &base_content[..boundary]
}

fn fast_bpe_impl_bench(c: &mut Criterion) {
    let content = load_data("data/dataset").expect("Could not load data for benchmarking");
    let text = content.contents.join("\n\0\n");

    static VOCAB_SIZE: usize = 1000;
    static MIN_PAIR_FREQ: usize = 2;

    
    for &data_size in DATA_SIZES {
        let mut group = c.benchmark_group(format!("{data_size}"));
        group
            .sample_size(10)
            .measurement_time(Duration::from_secs(5))
            .warm_up_time(Duration::from_secs(3));
        
        let pretokens = preprocess_text(extract_data(&text, data_size));

        group.bench_function(format!("Parallel"), |b| {
            b.iter(|| {
                let _ = optimized::parallel::bpe_training(
                    black_box(&pretokens),
                    black_box(VOCAB_SIZE),
                    black_box(MIN_PAIR_FREQ)
                );
            });
        });

        group.bench_function(format!("PriorityQueue"), |b| {
            b.iter(|| {
                let _ = optimized::max_heap::bpe_training(
                    black_box(&pretokens),
                    black_box(VOCAB_SIZE),
                    black_box(MIN_PAIR_FREQ)
                );
            });
        });

        group.finish();
    }
}

criterion_group!(benches, fast_bpe_impl_bench);
criterion_main!(benches);