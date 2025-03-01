use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec::Vec,
};

pub mod matrix_helper;

mod buffer;
use std::path::PathBuf;

pub use buffer::BufferObj;

pub mod load_texture;
pub use load_texture::AnyTexture;

pub mod shader;
pub mod vertex;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MVPUniform {
    pub mvp_matrix: [[f32; 4]; 4],
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn application_root_dir() -> String {
    let host = web_sys::window().unwrap().location().host().unwrap();
    if host.contains("localhost") {
        String::from("http://localhost:8000/")
    } else {
        String::from("https://") + &host + "/simuverse/"
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn application_root_dir() -> String {
    use std::env;
    use std::fs;

    match env::var("PROFILE") {
        Ok(_) => String::from(env!("CARGO_MANIFEST_DIR")),
        Err(_) => {
            let mut path = env::current_exe().expect("Failed to find executable path.");
            while let Ok(target) = fs::read_link(path.clone()) {
                path = target;
            }
            if cfg!(any(
                target_os = "macos",
                target_os = "windows",
                target_os = "linux"
            )) {
                path = path.join("../../../assets/").canonicalize().unwrap();
            }

            String::from(path.to_str().unwrap())
        }
    }
}

#[allow(unused)]
pub(crate) fn get_texture_file_path(name: &str) -> PathBuf {
    PathBuf::from(application_root_dir()).join(name)
}
