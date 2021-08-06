pub mod asteroid;
use asteroid::Asteroid;
use hotham::{App, HothamResult};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    println!("[HOTHAM_ASTEROID_ANDROID] MAIN!");
    real_main().unwrap();
}

pub fn real_main() -> HothamResult<()> {
    let program = Asteroid::new();
    let mut app = App::new(program)?;
    app.run()?;
    Ok(())
}