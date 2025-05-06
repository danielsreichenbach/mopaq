use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mopaq::*;
use std::io::Cursor;

pub fn header_v1_roundtrip_benchmark(c: &mut Criterion) {
    let header = MpqHeader::new_v1();

    c.bench_function("header_v1_write", |b| {
        b.iter(|| {
            let mut buffer = Cursor::new(Vec::new());
            black_box(&header).write(&mut buffer).unwrap();
        })
    });

    c.bench_function("header_v1_read", |b| {
        let mut buffer = Cursor::new(Vec::new());
        header.write(&mut buffer).unwrap();
        buffer.set_position(0);

        b.iter(|| {
            buffer.set_position(0);
            black_box(MpqHeader::read(&mut buffer).unwrap());
        })
    });
}

pub fn header_v4_roundtrip_benchmark(c: &mut Criterion) {
    let header = MpqHeader::new_v4();

    c.bench_function("header_v4_write", |b| {
        b.iter(|| {
            let mut buffer = Cursor::new(Vec::new());
            black_box(&header).write(&mut buffer).unwrap();
        })
    });

    c.bench_function("header_v4_read", |b| {
        let mut buffer = Cursor::new(Vec::new());
        header.write(&mut buffer).unwrap();
        buffer.set_position(0);

        b.iter(|| {
            buffer.set_position(0);
            black_box(MpqHeader::read(&mut buffer).unwrap());
        })
    });
}

pub fn user_header_roundtrip_benchmark(c: &mut Criterion) {
    let user_header = MpqUserHeader::new();
    let mpq_header = MpqHeader::new_v1();

    c.bench_function("user_header_write", |b| {
        b.iter(|| {
            let mut buffer = Cursor::new(Vec::new());
            write_mpq_header(
                &mut buffer,
                Some(black_box(&user_header)),
                black_box(&mpq_header),
            )
            .unwrap();
        })
    });

    c.bench_function("user_header_read", |b| {
        let mut buffer = Cursor::new(Vec::new());
        write_mpq_header(&mut buffer, Some(&user_header), &mpq_header).unwrap();
        buffer.set_position(0);

        b.iter(|| {
            buffer.set_position(0);
            black_box(read_mpq_header(&mut buffer).unwrap());
        })
    });
}

criterion_group!(
    benches,
    header_v1_roundtrip_benchmark,
    header_v4_roundtrip_benchmark,
    user_header_roundtrip_benchmark
);
criterion_main!(benches);
