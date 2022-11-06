//利用docker的api进行构建, 运行kaniko容器并进行构建

use crate::dockerapi::runcontainerd::RunDocker;
use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::LogsOptions;
use bollard::container::RemoveContainerOptions;
use bollard::container::StartContainerOptions;
use bollard::exec::CreateExecOptions;
use bollard::exec::StartExecResults;
use bollard::image::CreateImageOptions;
use bollard::service::HostConfig;
use bollard::Docker;
use futures::Stream;
// use futures::TryStreamExt;
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;
use serde_json::json;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use tracing::info;
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

    // pull镜像,利用create 来pull 基础构造镜像
    pub async fn autopull(&self) -> Result<(), anyhow::Error> {
        let docker = Docker::connect_with_socket_defaults().unwrap();
        //此项将会将镜像拉取下来
        docker
            .create_image(
                Some(CreateImageOptions {
                    from_image: KANIKO_IMAGE,
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await?;

        // let hostconfig = HostConfig {
        //     auto_remove: Some(false),
        //     ..Default::default()
        // };
        let alpine_config = Config {
            image: Some(KANIKO_IMAGE),
            tty: Some(true),
            // host_config: Some(hostconfig),
            ..Default::default()
        };

        let id = docker
            .create_container::<&str, &str>(None, alpine_config)
            .await?
            .id;
        docker.start_container::<String>(&id, None).await?;
        // non interactive
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
        dbg!(&start_cmd);
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
                KANIKO_IMAGE,
                start_cmd,
            )
            .await?;

        let listlog_opt: LogsOptions<&str> = LogsOptions {
            follow: true,
            stdout: true,
            stderr: true,
            timestamps: true,
            ..Default::default()
        };

        let mut output_stream = docker.logs(&id, Some(listlog_opt));
        while let Some(Ok(msg)) = output_stream.next().await {
            print!("{}", msg);
        }
        // let exec = docker
        //     .create_exec(
        //         &id,
        //         CreateExecOptions {
        //             attach_stdout: Some(true),
        //             attach_stderr: Some(true),
        //             cmd: Some(start_cmd),
        //             ..Default::default()
        //         },
        //     )
        //     .await?
        //     .id;
        // if let StartExecResults::Attached { mut output, .. } =
        // docker.start_exec(&exec, None).await?
        // {
        //     while let Some(Ok(msg)) = output.next().await {
        //         print!("{}", msg);
        //     }
        // } else {
        //     unreachable!();
        // }

        docker
            .remove_container(
                &id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        Ok(())
    }

    ///启动一个docker容器运行并开始根据提供的dockerfile 构造镜像
    pub async fn kaniko_start_build(&self) -> Result<(), anyhow::Error> {
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
                KANIKO_IMAGE,
                start_cmd,
            )
            .await?;
        Ok(())
    }
}

///从path config中解析数据并执行kaniko docker 构建
///use kaniko to build with config_file.yaml
/// ```
/// build_message:
///     kaniko_image: love #构建基础镜像名字
///     workspace_map: config #构建地址
///     config_json_map: sadas #docker-registr生成的配置文件地址   /temp/config.json
///     git_url: asjdjasjdj #要构建的镜像git地址git://github.com/loyurs/qkrun.git#refs/heads/master
///     git_subfolder: sadsad #子文件夹  形如：dockerfiles/test/
///     dest_image: asdasda #ccr.ccs.tencentyun.com/tctd/yuxin:love
/// registry:
///     user: asdsad
///     password: asdss
///     registry_url: assdss
/// ```
pub async fn kaniko_docker_build_image_with_config_file(
    config_path: PathBuf,
) -> Result<(), anyhow::Error> {
    let reader_yaml = std::fs::File::open(config_path).unwrap();
    let mut reader = BufReader::new(reader_yaml);
    let conf: Value = serde_yaml::from_reader(reader).unwrap();
    let mut retval = String::new();
    let value_get = |x: &'static str, y: &'static str| -> &str {
        conf.get(x).unwrap().get(y).unwrap().as_str().unwrap()
    };
    let registry_docker = || -> Image_Registry<String> {
        Image_Registry {
            user: value_get("registry", "user").to_string(),
            password: value_get("registry", "password").to_string(),
            registry_url: value_get("registry", "registry_url").to_string(),
        }
    };
    let kaniko_build_info = KanikoBuildInfo::new(
        value_get("build_message", "kaniko_image").to_string(),
        value_get("build_message", "workspace_map").to_string(),
        value_get("build_message", "config_json_map").to_string(),
        value_get("build_message", "git_url").to_string(),
        value_get("build_message", "git_subfolder").to_string(),
        value_get("build_message", "dest_image").to_string(),
        registry_docker(),
    );
    kaniko_build_info.kaniko_start_build().await?;
    info!("complete");
    Ok(())
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
            // "seven",
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

pub fn kaniko_docker_config_template_generate(paths: &PathBuf) -> Result<(), anyhow::Error> {
    let yaml_temp = format!(
        "{}",
        r#"  build_message:
    kaniko_image: ubuntu:20.04 #构建基础镜像名字
    workspace_map: config #构建地址
    config_json_map: /tmp/config.json #docker-registr生成的配置文件地址，push镜像需要将其挂载到kaniko容器内 /temp/config.json
    git_url: git://github.com/loyurs/qkrun.git#refs/heads/master #要构建的镜像git地址git://github.com/loyurs/qkrun.git#refs/heads/master
    git_subfolder: build_images/dockerfiles/tda/Dockerfile #子文件夹  形如：dockerfiles/test/";
    dest_image: ccr.ccs.tencentyun.com/tctd/yuxin:love1 #ccr.ccs.tencentyun.com/tctd/yuxin:love
  registry: #docker镜像仓库地址，用户名和密码
    user: demo #100016367772
    password: *****
    registry_url: ccr.ccs.tencentyun.com
"#
    );
    fs::write(paths, &yaml_temp)?;
    Ok(())
}
#[test]
fn kaniko_docker_config_template_generate_test() {
    kaniko_docker_config_template_generate(&PathBuf::from("sad.yaml")).unwrap();
}
