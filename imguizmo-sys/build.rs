extern crate cc;

use std::{path::PathBuf, fs::File, io::Write};

use git2::Repository;

const IMGUIZMO_REPO_URL: &str = "https://github.com/CedricGuillemet/ImGuizmo.git";
const IMGUI_REPO_URL: &str = "https://github.com/imgui-rs/imgui-rs.git";

struct Paths {
    pub temp_path: PathBuf,
    pub imguizmo_repo: PathBuf,
    pub imgui_repo: PathBuf,
}

fn download_repo(repo_url: &str, path: &std::path::Path) {
    if path.exists() {
        return;
    }

    _ = match Repository::clone(repo_url, path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
}

fn download_imgui(paths: &Paths) {
    let imgui_rs_repo = paths.temp_path.clone().join("imgui-rs");
    download_repo(IMGUI_REPO_URL, &imgui_rs_repo);

    // TODO: assumes you are using the docking feature
    fs_extra::dir::move_dir(imgui_rs_repo.join("imgui-sys/third-party/imgui-docking/imgui"), &paths.temp_path, &fs_extra::dir::CopyOptions::new()).unwrap();
    std::fs::remove_dir_all(imgui_rs_repo).unwrap();
}

fn compile_imguizmo(paths: &Paths) {
    download_repo(IMGUIZMO_REPO_URL, &paths.imguizmo_repo);

    let mut build = cc::Build::new();
    build.cpp(true);
    
    for entry in std::fs::read_dir(&paths.imguizmo_repo).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "cpp" {
                build.file(path);
            }
        }
    }

    build.include(&paths.imgui_repo);

    build.compile("imguizmo");
}

fn generate_bindings(paths: &Paths) {
    println!("cargo:rerun-if-changed=wrapper.h");

    let mut wrapper_header = String::new();
    
    for entry in std::fs::read_dir(&paths.imguizmo_repo).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "h" {
                wrapper_header.push_str(&format!("#include \"{}\"\n", path.to_str().unwrap()));
            }
        }
    }

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let wrapper_path = out_path.join("wrapper.hpp");
    let mut file = File::create(&wrapper_path).unwrap();
    file.write_all(wrapper_header.as_bytes()).unwrap();

    let bindings = bindgen::Builder::default()
        .header(wrapper_path.to_str().unwrap())
        .clang_arg(format!("-I{}", paths.imgui_repo.to_str().unwrap()))
        .trust_clang_mangling(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let bindings_out = out_path.join("bindings.rs");
    bindings
        .write_to_file(bindings_out)
        .expect("Couldn't write bindings!");
}

fn main() {
    let temp_path = std::env::var("TEMP").unwrap();
    let temp_path = std::path::Path::new(&temp_path).join("imguizmo-sys-tmp");

    let imguizmo_repo = temp_path.clone().join("ImGuizmo");
    let imgui_repo = temp_path.clone().join("imgui");

    let paths = Paths {
        temp_path,
        imguizmo_repo,
        imgui_repo,
    };

    // TODO: have a reuse system, this is not fast.
    if paths.temp_path.exists() {
        std::fs::remove_dir_all(&paths.temp_path).unwrap();
    }

    std::fs::create_dir(&paths.temp_path).unwrap();

    download_imgui(&paths);
    compile_imguizmo(&paths);
    println!("cargo:rustc-link-lib=imguizmo");
    generate_bindings(&paths);

    std::fs::remove_dir_all(&paths.temp_path).unwrap();
}