#[cfg(feature = "music_library")]
mod music_lib_test {
    use mps_interpreter::music::*;

    #[test]
    fn generate_library() {
        let mut lib = MpsLibrary::new();
        lib.read_path("/home/ngnius/Music", 10).unwrap();
        println!("generated library size: {}", lib.len());
    }
}
