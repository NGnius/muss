#[cfg(feature = "music_library")]
mod music_lib_test {
    use muss_interpreter::music::*;

    #[test]
    fn generate_library() {
        let mut lib = Library::new();
        lib.read_path("/home/ngnius/Music", 10).unwrap();
        println!("generated library size: {}", lib.len());
    }
}
