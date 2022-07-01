#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate mps_interpreter;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        print!("len:{},data:{}\n", data.len(), s)
    } else {
        print!("len:{},data:<non-ut8>,", data.len());
    }
    let mut cursor = std::io::Cursor::new(data);
    let interpreter = mps_interpreter::MpsFaye::with_stream(&mut cursor);
    for item in interpreter {
        match item {
            Err(e) => print!("err:{},", e),
            Ok(_i) => {},//print!("item:{},", i),
        }
    }
    println!("done.");
});
