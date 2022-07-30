use muss_interpreter::Interpreter;
//use mps_interpreter::MpsRunner;
use std::fs::File;
use std::io::{BufReader, Read, Seek};

use criterion::{criterion_group, criterion_main, Criterion};

/*fn interpretor_benchmark(c: &mut Criterion) {
    let f = File::open("benches/lots_of_empty.mps").unwrap();
    let mut reader = BufReader::with_capacity(1024 * 1024 /* 1 MiB */, f);
    // read everything into buffer before starting
    let mut buf = Vec::with_capacity(1024 * 1024);
    reader.read_to_end(&mut buf).unwrap();
    drop(buf);
    c.bench_function("mps lots_of_empty.mps", |b| {
        b.iter(|| {
            //let f = File::open("benches/lots_of_empty.mps").unwrap();
            //let mut reader = BufReader::new(f);
            reader.rewind().unwrap();
            let mps = MpsRunner::with_stream(&mut reader);
            for item in mps {
                match item {
                    Err(e) => panic!("{}", e),
                    Ok(_) => {}
                }
            }
        })
    });
}*/

fn faye_benchmark(c: &mut Criterion) {
    let f = File::open("benches/lots_of_empty.muss").unwrap();
    let mut reader = BufReader::with_capacity(1024 * 1024 /* 1 MiB */, f);
    // read everything into buffer before starting
    let mut buf = Vec::with_capacity(1024 * 1024);
    reader.read_to_end(&mut buf).unwrap();
    drop(buf);
    c.bench_function("muss-faye lots_of_empty.muss", |b| {
        b.iter(|| {
            //let f = File::open("benches/lots_of_empty.mps").unwrap();
            //let mut reader = BufReader::new(f);
            reader.rewind().unwrap();
            let mps = Interpreter::with_stream(&mut reader);
            for item in mps {
                match item {
                    Err(e) => panic!("{}", e),
                    Ok(_) => {}
                }
            }
        })
    });
}

criterion_group!(
    parse_benches,
    /*interpretor_benchmark,*/ faye_benchmark
);
criterion_main!(parse_benches);
