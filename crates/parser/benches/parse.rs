use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_parse(c: &mut Criterion) {
    let text = include_str!(
        "/mnt/d/Program Files/Steam/steamapps/common/Team Fortress 2/tf/scripts/vscripts/cheats/aimbot.nut"
    );
    c.bench_function("parse", |b| {
        b.iter(|| sq_3_parser::Parse::new(black_box(text)))
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
