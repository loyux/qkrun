use crate::dockerapi::{self, runcontainerd::RunDocker};

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
