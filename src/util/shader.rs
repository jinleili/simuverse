use std::{borrow::Cow, fs::read_to_string, path::PathBuf};
use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};

const SHADER_IMPORT: &str = "#include ";
const SHADER_SEGMENT: &str = "#insert_code_snippet";

#[cfg(target_arch = "wasm32")]
pub fn application_root_dir() -> String {
    let host = web_sys::window().unwrap().location().host().unwrap();
    "http://".to_string() + &host
}

#[cfg(not(target_arch = "wasm32"))]
pub fn application_root_dir() -> String {
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
                path = path
                    .join("../../../assets/")
                    .canonicalize()
                    .unwrap();
            }

            String::from(path.to_str().unwrap())
        }
    }
}

#[allow(dead_code)]
pub fn create_shader_module(
    device: &wgpu::Device,
    shader_name: &'static str,
    label: Option<&str>,
) -> ShaderModule {
    insert_code_then_create(device, shader_name, None, label)
}

#[allow(dead_code)]
pub fn insert_code_then_create(
    device: &wgpu::Device,
    shader_name: &'static str,
    code_snippet: Option<&str>,
    label: Option<&str>,
) -> ShaderModule {
    // env!("CARGO_MANIFEST_DIR") 是编译时执行的，得到的是当前所编辑的库的所在路径，而不是项目的路径
    // std::env::var("CARGO_MANIFEST_DIR") 在 xcode debug 时不存在
    // std::env::current_dir() 在 xcode debug 时只能获得相对路径： “/”
    let base_dir = application_root_dir();
    let (fold, shader_name) = if cfg!(any(target_os = "ios", target_arch = "wasm32")) {
        ("preprocessed-wgsl", shader_name.replace("/", "_"))
    } else {
        ("wgsl", shader_name.to_string())
    };
    let code = request_shader_code(&base_dir, fold, &shader_name);

    let shader_source = if cfg!(any(target_os = "ios", target_arch = "wasm32")) {
        code.to_string()
    } else {
        let mut shader_source = String::new();
        parse_shader_source(&code, &mut shader_source, &base_dir);
        shader_source
    };

    let final_source = if let Some(segment) = code_snippet {
        let mut output = String::new();
        for line in shader_source.lines() {
            if line.contains(SHADER_SEGMENT) {
                output.push_str(segment);
            } else {
                output.push_str(line);
            }
            output.push_str("\n ");
        }
        output
    } else {
        shader_source
    };

    device.create_shader_module(ShaderModuleDescriptor {
        label,
        source: ShaderSource::Wgsl(Cow::Borrowed(&final_source)),
    })
}

#[cfg(target_arch = "wasm32")]
fn request_shader_code(base_dir: &str, fold: &str, shader_name: &str) -> String {
    // 主线程中同步的 XMLHttpRequest 已不赞成使用(2021/05/07)
    let mut request = web_sys::XmlHttpRequest::new().unwrap();
    request.set_response_type(web_sys::XmlHttpRequestResponseType::None);
    let url = base_dir.to_string() + "/" + &shader_name + ".wgsl";
    request.open_with_async("get", &url, false);
    request.send();
    request.response_text().unwrap().unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
fn request_shader_code(base_dir: &str, fold: &str, shader_name: &str) -> String {
    let path = PathBuf::from(base_dir)
        .join(fold)
        .join(format!("{shader_name}.wgsl"));
    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            panic!("Unable to read {:?}: {:?}", path, e)
        }
    };
    code
}

fn parse_shader_source(source: &str, output: &mut String, base_path: &str) {
    for line in source.lines() {
        if line.starts_with(SHADER_IMPORT) {
            let imports = line[SHADER_IMPORT.len()..].split(',');
            // For each import, get the source, and recurse.
            for import in imports {
                if let Some(include) = get_shader_funcs(import, base_path) {
                    parse_shader_source(&include, output, base_path);
                } else {
                    println!("shader parse error -------");
                    println!("can't find shader functions: {}", import);
                    println!("--------------------------");
                }
            }
        } else {
            output.push_str(line);
            output.push_str("\n ");
            // 移除注释
            // let need_delete = match line.find("//") {
            //     Some(_) => {
            //         let segments: Vec<&str> = line.split("//").collect();
            //         segments.len() > 1 && segments.first().unwrap().trim().is_empty()
            //     }
            //     None => false,
            // };
            // if !need_delete {
            //     output.push_str(line);
            //     output.push_str("\n");
            // }
        }
    }
}

fn get_shader_funcs(key: &str, base_path: &str) -> Option<String> {
    let path = PathBuf::from(base_path)
        .join("wgsl")
        .join(key.replace('"', ""));
    let shader = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };
    Some(shader)
}
