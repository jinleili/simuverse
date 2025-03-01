use super::*;
use std::{fs::read_to_string, path::PathBuf};
use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};

const SHADER_IMPORT: &str = "#include ";
const SHADER_SEGMENT: &str = "#insert_code_snippet";

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
    let base_dir = super::application_root_dir();
    let (fold, shader_name) = if cfg!(target_arch = "wasm32") {
        ("assets/preprocessed-wgsl", shader_name.replace('/', "_"))
    } else {
        ("wgsl", shader_name.to_string())
    };
    let code = request_shader_code(&base_dir, fold, &shader_name);

    let shader_source = if cfg!(target_arch = "wasm32") {
        code
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
    let request = web_sys::XmlHttpRequest::new().unwrap();
    let url = base_dir.to_string() + fold + "/" + shader_name + ".wgsl";
    request.open_with_async("GET", &url, false).unwrap();
    request.send().unwrap();
    request.response_text().unwrap().unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
fn request_shader_code(base_dir: &str, fold: &str, shader_name: &str) -> String {
    let path = PathBuf::from(base_dir)
        .join(fold)
        .join(format!("{shader_name}.wgsl"));
    match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            panic!("Unable to read {path:?}: {e:?}")
        }
    }
}

fn parse_shader_source(source: &str, output: &mut String, base_path: &str) {
    for line in source.lines() {
        if let Some(stripped) = line.strip_prefix(SHADER_IMPORT) {
            let imports = stripped.split(',');
            // For each import, get the source, and recurse.
            for import in imports {
                if let Some(include) = get_shader_funcs(import, base_path) {
                    parse_shader_source(&include, output, base_path);
                } else {
                    log::info!("shader parse error \n can't find shader functions: {import}");
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
        Err(e) => panic!("Unable to read {path:?}: {e:?}"),
    };
    Some(shader)
}
