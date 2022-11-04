use std::{fs, io::BufReader, path::PathBuf};

use crate::{
    buildimg::kaniko_docker::KanikoBuildInfo,
    dockerapi::{self, runcontainerd::RunDocker},
    templates::models::{self, new_docker_registry, new_kaniko_build},
};
use anyhow::Error;
use serde_yaml::Value;
use tracing::info;

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
        .await
        .unwrap();
}

///从配置文件->执行通过kubernetes pod进行镜像构造
pub async fn kaniko_pod_get_config_from_path(paths: PathBuf) -> Result<(String, String), Error> {
    //获取配置文件信息
    let file = fs::File::open(paths)?;
    let mut reader = BufReader::new(file);
    let conf: Value = serde_yaml::from_reader(reader)?;
    dbg!(&conf);
    let value_get = |x: &'static str, y: &'static str| -> &str {
        conf.get(x).unwrap().get(y).unwrap().as_str().unwrap()
    };
    let registry_configmap = new_docker_registry(
        value_get("registry", "user"),
        value_get("registry", "password"),
        value_get("registry", "url").to_string(),
        models::DOCKER_REGISTRY,
        value_get("build_message", "configmap_name").to_string(),
        value_get("build_message", "namespace").to_string(),
    );
    // println!("{}", registry_configmap);
    let build_yaml = new_kaniko_build(
        models::KANIKO_BUILD,
        value_get("build_message", "name").to_string(),
        value_get("build_message", "git").to_string(),
        value_get("build_message", "subpath").to_string(),
        value_get("build_message", "image_name").to_string(),
        value_get("build_message", "configmap_name").to_string(),
        value_get("build_message", "namespace").to_string(),
    );
    // println!("{}", build_yaml);
    Ok((registry_configmap, build_yaml))
}

///生成kaniko_pod 运行配置模板文件
pub fn kaniko_pod_config_template_generate(paths: &PathBuf) -> Result<(), anyhow::Error> {
    let yaml_temp = format!(
        "{}",
        r#"  build_message:
    image_name: String #镜像名字
    name: String #pod名字
    git: String #git content链接
    subpath: String #git子目录
    configmap_name: sdsad #用户挂载push secret的configmap
    namespace: sad
  registry:
    user: String
    password: String
    url: String"#
    );
    fs::write(paths, &yaml_temp)?;
    Ok(())
}
