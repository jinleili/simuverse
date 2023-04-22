use std::error::Error;
use std::fs::read_to_string;
use std::io::prelude::*;
use std::path::PathBuf;

const OUTPUT: &str = "../assets/preprocessed-wgsl";
const SHADER_PATH_INPUT: &str = "../assets/wgsl";

pub fn preprocess_wgsl() -> Result<(), Box<dyn Error>> {
    let shader_files: Vec<&str> = vec![
        "lbm/init",
        "lbm/collide_stream",
        "lbm/trajectory_present",
        "lbm/present",
        "lbm/particle_update",
        "lbm/blend_img",
        "lbm/boundary",
        "lbm/curl_update",
        "egui_layer_compose",
        "trajectory_update",
        "present",
        "field_setting",
        "noise/3d_noise_tex",
        "noise/sphere_tex",
        "pbd/cloth_display",
        "pbd/cloth_external_force",
        "pbd/xxpbd/cloth_bending_solver",
        "pbd/xxpbd/cloth_predict",
        "pbd/xxpbd/cloth_stretch_solver",
    ];

    // 创建目录
    std::fs::create_dir_all(OUTPUT)?;
    for name in shader_files {
        let _ = regenerate_shader(name);
    }
    Ok(())
}

fn regenerate_shader(shader_name: &str) -> Result<(), Box<dyn Error>> {
    let base_dir = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(&base_dir)
        .join(SHADER_PATH_INPUT)
        .join(format!("{}.wgsl", shader_name));
    let mut out_path = OUTPUT.to_string();
    out_path += &format!("/{}.wgsl", shader_name.replace('/', "_"));

    let code = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => {
            panic!("Unable to read {:?}: {:?}", path, e)
        }
    };

    let mut shader_source = String::new();
    parse_shader_source(&code, &mut shader_source, base_dir);

    let mut f = std::fs::File::create(std::path::Path::new(base_dir).join(&out_path))?;
    f.write_all(shader_source.as_bytes())?;

    Ok(())
}

fn parse_shader_source(source: &str, output: &mut String, base_path: &str) {
    let include: &str = "#include ";
    for line in source.lines() {
        if let Some(stripped_line) = line.strip_prefix(include) {
            let imports = stripped_line.split(',');
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
            // 移除注释
            let need_delete = match line.find("//") {
                Some(_) => {
                    let segments: Vec<&str> = line.split("//").collect();
                    segments.len() > 1 && segments.first().unwrap().trim().is_empty()
                }
                None => false,
            };
            if !need_delete {
                output.push_str(line);
                output.push('\n');
            }
        }
    }
}

fn get_shader_funcs(key: &str, base_path: &str) -> Option<String> {
    let path = PathBuf::from(base_path)
        .join(SHADER_PATH_INPUT)
        .join(key.replace('"', ""));
    let shader = match read_to_string(&path) {
        Ok(code) => code,
        Err(e) => panic!("Unable to read {:?}: {:?}", path, e),
    };
    Some(shader)
}
