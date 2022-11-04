use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::Error;
use tracing::{info, warn};

///直接通过内置dockerfile构造
///quickrun::usebuildkit::dockerd_buildkit_build(
///      "asd",
///      "https://github.com/loyurs/qkrun.git#master:dockerfiles/code_server_with_ssh/",
///      "/home/:/home",
///  );
pub fn build_rust_dev_image_with_extensions(image_name_and_tag: &str) {
    let dockerfile = r#"FROM registry.cn-hangzhou.aliyuncs.com/clouddevs/vscode-extensions:rust AS builder
FROM registry.cn-hangzhou.aliyuncs.com/clouddevs/code-server:ubuntu20-ssh
COPY --from=builder  /opt/extensions /config/extensions
RUN mkdir -p /root/.vscode-server && ln -s /config/extensions /root/.vscode-server/extensions"#;
    let mut child = Command::new("docker")
        .env("DOCKER_BUILDKIT", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(&["build", "-", "-t", image_name_and_tag])
        .spawn()
        .expect("Failed to spawn child process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(dockerfile.as_bytes())
            .expect("Failed to write to stdin");
    });
    let output = child.wait_with_output().expect("Failed to read stdout");
    println!("{:?}", String::from_utf8_lossy(&output.stdout));
    println!("****Done****");
}

///通过github文件进行构造
pub fn git2build(git_urls: &str, image_name_and_tag: &str) {
    info!(
        "Start to build {} for git_url: {}",
        image_name_and_tag, git_urls
    );
    warn!("Docker ssh passwd is li");
    Command::new("docker")
        .args(&["build", git_urls, "-t", image_name_and_tag])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Build use git error");
}

///通过github url 的dockerfile, 使用docker自带构建
pub fn giturl_branch_and_folder(
    git_url_branch_folder: &str,
    container_name: &str,
) -> Result<(), Error> {
    info!("fetch {} ,start to build", git_url_branch_folder);
    Command::new("docker")
        .args(&["build", git_url_branch_folder, "--tag", container_name])
        .stdout(Stdio::inherit())
        .output()?;
    Ok(())
}

///利用一次性构建的buildkit进行构建,需要带有inherite()，流式输入输出与错误
///输出的类型有，tar文件(import 导入)，oci，注册表类型；
/// git_url -> "context=https://github.com/loyurs/qkrun.git#master:docker/rust_code_server/"
/// tarfile_path_map-> "/home:/home"
/// name -> ""
pub fn dockerd_buildkit_build(name: &str, git_url: &str, tarfile_path_map: &str) {
    // let volume_map = "/home:/home";
    let mut struct_url = String::from("context=");
    struct_url.push_str(git_url);
    info!("Start to build use buildkit");
    Command::new("docker")
        .args(&[
            "run",
            "-it",
            "--rm",
            "--privileged",
            "-v",
            tarfile_path_map,
            "--entrypoint",
            "buildctl-daemonless.sh",
            "moby/buildkit:latest",
            "build",
            "--frontend",
            "dockerfile.v0",
            "--opt",
            "source=docker/dockerfile",
            "--opt",
            struct_url.as_str(),
            "--output",
            // "type=tar,dest=/home/geneout.tar", //need to updates
            "type=image,name=docker.io/lidatong/aini, push=true",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    info!("Ok");
}
