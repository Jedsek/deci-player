use itertools::Itertools;
use rodio::{Decoder, OutputStream, Sink};
use rustfft::{num_complex::Complex, FftPlanner};
use std::{fs::File, io::BufReader, path::Path};

pub type TheSource = Decoder<BufReader<File>>;

pub fn new_sink() -> Sink {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    Box::leak(Box::new(stream));
    Sink::try_new(&stream_handle).unwrap()
}

pub fn new_source(path: impl AsRef<Path>) -> TheSource {
    let path = path.as_ref();
    let source = File::open(path).unwrap();
    let source = BufReader::new(source);
    Decoder::new(source).unwrap()
}

#[allow(unused)]
pub fn sample(source: TheSource) {
    let to_complex = |x| Complex::new(x as i64, 0);
    let to_mod = |x: Complex<i64>| x.re * x.re + x.im * x.im;

    let chunk_size = 1024;
    let fft = FftPlanner::new().plan_fft_forward(chunk_size);

    for window in &source.map(to_complex).chunks(chunk_size) {
        let mut window = window.collect_vec();

        if window.len() < 1024 {
            continue;
        }

        fft.process(&mut window);
    }

    std::process::exit(1);
}
