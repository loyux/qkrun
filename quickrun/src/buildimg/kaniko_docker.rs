use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::StartContainerOptions;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use tracing::info;

use crate::dockerapi::runcontainerd::RunDocker;
const KANIKO_IMAGE: &str = "registry.cn-hangzhou.aliyuncs.com/clouddevs/kanico:latest";
///use kaniko to build with git
/// 利用git及其子目录进行构建，docker
// pub fn build_kaniko() {
//     let workspace_map = "/home:/workspace";
//     let config_json_map = "/home/config.json:/kaniko/.docker/config.json:ro";
//     let git_url = "git://github.com/loyurs/qkrun.git#refs/heads/master";
//     let git_subfolder = "dockerfiles/test/";
//     let dest_image = "ccr.ccs.tencentyun.com/tctd/yuxin:love";
//     Command::new("docker")
//         .args(&[
//             "run",
//             "-ti",
//             "--rm",
//             "-v",
//             workspace_map,
//             "-v",
//             config_json_map,
//             KANIKO_IMAGE,
//             "--context",
//             git_url,
//             "--context-sub-path",
//             git_subfolder,
//             "--dockerfile",
//             "Dockerfile",
//             "--destination",
//             dest_image,
//         ])
//         .stderr(Stdio::inherit())
//         .stdin(Stdio::inherit())
//         .stdout(Stdio::inherit())
//         .output()
//         .unwrap();
//     // std::fs::remove_file("/home/config.json").unwrap();
// }

///use kaniko to build with context
pub fn generaste_base64_secret(
    user: &str,
    password: &str,
    url: &str,
    generate_json_file_path: &str,
) {
    let based64 = base64::encode(format!("{}:{}", user, password));
    let pp = json!({
        "auths": {
            url: {
                "auth": based64
            }
        }
    });
    std::fs::write(generate_json_file_path, pp.to_string().as_bytes()).unwrap();
    info!("成功生成docker push key /home/config.json");
    // println!("{}", pp);
}

///利用本地dockerfile进行构建
pub fn build_with_local_dockerfile(input_str: &str) {
    let echo_child = Command::new("echo")
        .arg(input_str)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start echo process");
    let echo_out = echo_child.stdout.expect("Failed to open echo stdout");
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg("echo")
        .stdin(Stdio::from(echo_out))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn kakaka() {
    build_with_local_dockerfile("input_dockerfiles");
}

pub struct KanikoBuildInfo {
    kaniko_image: String,
    workspace_map: String,
    config_json_map: String,
    git_url: String,
    git_subfolde: String,
    dest_image: String,
    docker_registry: Image_Registry<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Image_Registry<T> {
    pub user: T,
    pub password: T,
    pub registry_url: T,
}

impl<T> Image_Registry<T> {
    fn new(user: String, password: String, registry_url: String) -> Image_Registry<String> {
        Image_Registry {
            user,
            password,
            registry_url,
        }
    }
}

impl KanikoBuildInfo {
    pub fn new(
        kaniko_image: String,
        workspace_map: String,
        //生成的docker-registry文件json地址
        config_json_map: String,
        git_url: String,
        git_subfolde: String,
        dest_image: String,
        docker_registry: Image_Registry<String>,
    ) -> Self {
        KanikoBuildInfo {
            kaniko_image,
            workspace_map,
            config_json_map,
            git_url,
            git_subfolde,
            dest_image,
            docker_registry,
        }
    }
    ///启动一个docker容器运行并开始根据提供的dockerfile 构造镜像
    pub async fn kaniko_start_build(&self, container_name: &str) -> Result<(), anyhow::Error> {
        let start_cmd = vec![
            "--context",
            &self.git_url,
            "--context-sub-path",
            &self.git_subfolde,
            "--dockerfile",
            "Dockerfile",
            "--destination",
            &self.dest_image,
        ];
        let registry: &Image_Registry<String> = &self.docker_registry;
        generaste_base64_secret(
            &registry.user,
            &registry.password,
            &registry.registry_url,
            &self.config_json_map,
        );
        let dockerrun = RunDocker::default();
        dockerrun
            .docker_run_with_volume_mount(
                "/kaniko/.docker/config.json",
                self.config_json_map.as_str(),
                container_name,
                KANIKO_IMAGE,
                start_cmd,
            )
            .await?;
        Ok(())
    }
}

///build test
pub async fn kaniko_start_build1_test() -> Result<(), anyhow::Error> {
    let start_cmd = vec![
        "--context",
        "git://github.com/loyurs/qkrun.git#refs/heads/master",
        "--context-sub-path",
        "build_images/dockerfiles/ubuntu20_ssh/Dockerfile",
        "--dockerfile",
        "Dockerfile",
        "--destination",
        "ccr.ccs.tencentyun.com/tctd/yuxin:lov11",
    ];
    generaste_base64_secret(
        "100016367772",
        "docker_registry_password",
        "ccr.ccs.tencentyun.com",
        "asd".into(),
    );
    let dockerrun = RunDocker::default();
    dockerrun
        .docker_run_with_volume_mount(
            "/kaniko/.docker/config.json",
            "/home/config.json",
            "seven",
            KANIKO_IMAGE,
            start_cmd,
        )
        .await?;
    Ok(())
}

//testcmd cargo test --package quickrun --lib -- buildimg::kaniko_docker::test_build_and_push --exact --nocapture
#[tokio::test]
async fn test_build_and_push() {
    kaniko_start_build1_test().await.unwrap();
}
//running logs
// numerating objects: 342, done.
// Counting objects: 100% (342/342), done.
// Compressing objects: 100% (166/166), done.
// Total 342 (delta 132), reused 337 (delta 127), pack-reused 0
// INFO[0002] Retrieving image manifest ccr.ccs.tencentyun.com/loyu/dev:latest
// INFO[0002] Retrieving image ccr.ccs.tencentyun.com/loyu/dev:latest from registry ccr.ccs.tencentyun.com
// INFO[0003] Built cross stage deps: map[]
// INFO[0003] Retrieving image manifest ccr.ccs.tencentyun.com/loyu/dev:latest
// INFO[0003] Returning cached image manifest
// INFO[0003] Executing 0 build triggers
// INFO[0003] Skipping unpacking as no commands require it.
// INFO[0003] CMD /bin/bash
// INFO[0003] Pushing image to ccr.ccs.tencentyun.com/tctd/yuxin:lov11
// INFO[0006] Pushed ccr.ccs.tencentyun.com/tctd/yuxin@sha256:202d42335a14facb325901259743f4d6794e11557f9ccf4b9b60785e739b8e37
