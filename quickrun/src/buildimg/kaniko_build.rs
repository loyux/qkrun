use std::{io::BufReader, path::PathBuf};

use serde_yaml::Value;
use tracing::info;

use crate::{
    buildimg::kaniko_docker::KanikoBuildInfo,
    dockerapi::{self, runcontainerd::RunDocker},
};

use super::kaniko_docker::Image_Registry;

//利用Kaniko进行构建容器
//kaniko在docker中构建
// You will need to store your build context in a place that kaniko can access. Right now, kaniko supports these storage solutions:
// GCS Bucket
// S3 Bucket
// Azure Blob Storage
// Local Directory
// Local Tar
// Standard Input
// Git Repository
/// ```When running kaniko, use the --context flag with the appropriate prefix to specify the location of your build context:
/// Source	Prefix	Example
/// Local Directory	dir://[path to a directory in the kaniko container]	dir:///workspace
/// Local Tar Gz	tar://[path to a .tar.gz in the kaniko container]	tar://path/to/context.tar.gz
/// Standard Input	tar://[stdin]	tar://stdin
/// GCS Bucket	gs://[bucket name]/[path to .tar.gz]	gs://kaniko-bucket/path/to/context.tar.gz
/// S3 Bucket	s3://[bucket name]/[path to .tar.gz]	s3://kaniko-bucket/path/to/context.tar.gz
/// Azure Blob Storage	https://[account].[azureblobhostsuffix]/[container]/[path to .tar.gz]	https://myaccount.blob.core.windows.net/container/path/to/context.tar.gz
/// Git Repository	git://[repository url][#reference][#commit-id]	git://github.com/acme/myproject.git#refs/heads/mybranch#<desired-commit-id>
/// ```
/// ```第一步 启动容器
/// 第二部 传入命令
const IMAGE: &str = "registry.cn-hangzhou.aliyuncs.com/clouddevs/kanico:latest";

async fn kaniko_build_image() {
    let workspace_map = "/home:/workspace";
    let config_json_map = "/etc/docker/config.json:/kaniko/.docker/config.json:ro";
    let git_url = "git://github.com/loyurs/qkrun.git#refs/heads/master";
    let git_subfolder = "dockerfiles/test/";
    let dest_image = "ccr.ccs.tencentyun.com/tctd/yuxin:love";
    // dockerapi::runcontainerd::Cond::new(git_url, image_name, containerd_name, port, cmd);

    //通过api启动docker run类似命令
    let dockerrun = RunDocker::default();
    dockerrun
        .docker_run_with_volume_mount(
            "/root",
            "/vdb/containerd/",
            "seven",
            IMAGE,
            vec!["sleep", "3600"],
        )
        .await;
}

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
async fn kaniko_build_image_with_config_file(config_path: PathBuf) -> Result<(), anyhow::Error> {
    let reader_yaml = std::fs::File::open(config_path).unwrap();
    let mut reader = BufReader::new(reader_yaml);
    let conf: Value = serde_yaml::from_reader(reader).unwrap();
    // let conf = config_val.clone();
    let mut retval = String::new();
    let value_get = |x: &'static str, y: &'static str| -> &str {
        conf.get(x).unwrap().get(y).unwrap().as_str().unwrap()
    };
    let registry_docker = || -> Image_Registry<String> {
        Image_Registry {
            user: value_get("registry", "user").to_string(),
            password: value_get("registry", "user").to_string(),
            registry_url: value_get("registry", "user").to_string(),
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
    kaniko_build_info
        .kaniko_start_build("container_name")
        .await?;
    info!("start the container successful");
    Ok(())
}
