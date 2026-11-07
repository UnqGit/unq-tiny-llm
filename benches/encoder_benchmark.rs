use criterion::{Criterion, black_box, criterion_group, criterion_main};

use unq_tiny_llm::{
    loader::{DataLoadResult, load_data},
    tokenizer::{bpe::serialization::load_tokenizer, encoding::encode},
};

fn benchmark_encoder(c: &mut Criterion) {
    let text = load_data("data/dataset/books/fiction/silmarillion/chapter_1.txt")
        .expect("Unable to load data for encoding");

    let DataLoadResult { contents, .. } = text;
    let base_content = contents.join("\n\0\n");
    let tokenizer =
        load_tokenizer("data/training/tokenizer.json").expect("Unable to open tokenizer file");

    c.bench_function("Encoder", |b| {
        b.iter(|| {
            let _ = encode(black_box(&base_content), black_box(&tokenizer));
        });
    });
}

criterion_group!(benches, benchmark_encoder);
criterion_main!(benches);
