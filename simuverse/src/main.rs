pub fn main() -> Result<(), impl std::error::Error> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Info).expect("无法初始化日志库");
    }
    simuverse::app_handler::run()
}
