use std::{cmp, time::Duration};

use criterion::{Criterion, SamplingMode, black_box, criterion_group, criterion_main};

use unq_tiny_llm::{
    loader::{DataLoadResult, load_data}, tokenizer::{bpe::training, preprocessing::{PreTokens, preprocess_text}},
};

static DATA_SIZES: &[usize] = &[
    1_000,
    10_000,
    100_000,
    250_000,
    500_000,
    1_000_000,
];

static VOCAB_SIZES: &[usize] = &[
    300,
    375,
    500,
    625,
    750,
    875,
    1000,
];

fn extract_data(base_content: &str, size: usize) -> &str {
    let max_bytes = cmp::min(base_content.len(), size);
    let boundary = base_content.floor_char_boundary(max_bytes);
    &base_content[..boundary]
}

fn benchmark_bpe(c: &mut Criterion) {
    let text = load_data("data/dataset/books/fiction/game_of_thrones/1 - A Game of Thrones.txt")
        .expect("Failed to load data for benchmarking.");
    let min_pair_freq = 2;

    let DataLoadResult { contents, .. } = text;
    let base_content = contents.join("\n\0\n");

    let pretokens_sequence: Vec<PreTokens> = DATA_SIZES
        .iter()
        .map(|&size| preprocess_text(extract_data(&base_content, size)))
        .collect();

    
    for (&data_size, pretokens) in DATA_SIZES.iter().zip(pretokens_sequence.iter()) {
        let mut group = c.benchmark_group(format!("BPE Training/{data_size}B"));
        group
            .sample_size(20)
            .sampling_mode(SamplingMode::Auto)
            .measurement_time(Duration::from_secs(5))
            .warm_up_time(Duration::from_secs(3));

        for &vocab_size in VOCAB_SIZES {
            group.bench_function(format!("Naive/{vocab_size}"), |b| {
                b.iter(|| {
                    let _ = training::naive::bpe_training(
                        black_box(&pretokens),
                        black_box(vocab_size),
                        black_box(min_pair_freq),
                    );
                });
            });

            group.bench_function(format!("Optimized/{vocab_size}"), |b| {
                b.iter(|| {
                    let _ = training::optimized::min_alloc::bpe_training(
                        black_box(&pretokens),
                        black_box(vocab_size),
                        black_box(min_pair_freq),
                    );
                });
            });

            group.bench_function(format!("OptimizedStore/{vocab_size}"), |b| {
                b.iter(|| {
                    let _ = training::optimized::change_store::bpe_training(
                        black_box(&pretokens),
                        black_box(vocab_size),
                        black_box(min_pair_freq),
                    );
                });
            });

            group.bench_function(format!("OptimizedPar/{vocab_size}"), |b| {
                b.iter(|| {
                    let _ = training::optimized::parallel::bpe_training(
                        black_box(&pretokens),
                        black_box(vocab_size),
                        black_box(min_pair_freq),
                    );
                });
            });

            group.bench_function(format!("OptimizedPriorityQueue/{vocab_size}"), |b| {
                b.iter(|| {
                    let _ = training::optimized::max_heap::bpe_training(
                        black_box(&pretokens),
                        black_box(vocab_size),
                        black_box(min_pair_freq),
                    );
                });
            });
        }
        
        group.finish();
    }

}

criterion_group!(benches, benchmark_bpe);
criterion_main!(benches);
