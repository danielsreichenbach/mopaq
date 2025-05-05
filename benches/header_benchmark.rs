use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mopaq::header::{MpqHeader, MpqUserDataHeader};
use std::io::Cursor;

fn benchmark_header_read_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("MPQ Headers");

    // Benchmark for MPQ header
    group.bench_function("mpq_header_write_read", |b| {
        b.iter(|| {
            let header = MpqHeader::new(1);
            let mut buffer = Vec::new();
            let mut writer = Cursor::new(&mut buffer);
            header.write(&mut writer).unwrap();

            let mut reader = Cursor::new(&buffer);
            let _read_header = MpqHeader::read(&mut reader).unwrap();
        })
    });

    // Benchmark for MPQ user header
    group.bench_function("mpq_user_header_write_read", |b| {
        b.iter(|| {
            let user_header = MpqUserDataHeader::new(1024, 0x200);
            let mut buffer = Vec::new();
            let mut writer = Cursor::new(&mut buffer);
            user_header.write(&mut writer).unwrap();

            let mut reader = Cursor::new(&buffer);
            let _read_user_header = MpqUserDataHeader::read(&mut reader).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_header_read_write);
criterion_main!(benches);
