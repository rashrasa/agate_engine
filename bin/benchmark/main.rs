fn main() {
    let mut app = agate_engine::App::new(1920, 1080, 0);

    agate_engine::App::start(&mut app); // TODO(agate_engine): Implement programmatic stopping, benchmark mode
}
