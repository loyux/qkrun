//利用bollard api 启动容器，构建容器，并在容器启动后注入后执行命令(一次性命令，非守护进程)
//
//

use anyhow::Result;
use bollard::{
    container::{Config, CreateContainerOptions, StartContainerOptions},
    exec::{CreateExecOptions, StartExecOptions},
    image::BuildImageOptions,
    service::{HostConfig, Mount, MountTypeEnum},
    volume::CreateVolumeOptions,
    Docker,
};
use futures::TryStreamExt;
use std::{collections::HashMap, io::Read, time::Duration};
use tracing::{info, warn};

use crate::buildimg;

pub struct Cond {
    docker_build_url: String,
    image_name: String,
    containerd_name: String,
    port: String,
    cmd: Vec<&'static str>,
}

impl Cond {
    pub fn new(
        docker_build_url: String,
        image_name: String,
        containerd_name: String,
        port: String,
        cmd: Vec<&'static str>,
    ) -> Self {
        Cond {
            docker_build_url,
            image_name,
            containerd_name,
            port,
            cmd,
        }
    }
    pub async fn run_container(&self) -> Result<()> {
        self.run_devlang_env().await?;
        //attach the container
        info!("start container");
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.exec_cmd_in_container().await?;
        info!("exec command successful");

        info!(
            "************MESSAGE************
            password: li"
        );

        Ok(())
    }

    //进入容器执行命令
    pub async fn exec_cmd_in_container(&self) -> Result<()> {
        let docker_client = Docker::connect_with_socket_defaults()?;
        let exec_config: CreateExecOptions<String> = CreateExecOptions {
            cmd: Some(vec!["service".into(), "ssh".into(), "start".into()]),
            attach_stdin: Some(true),
            ..Default::default()
        };
        let msg_id = docker_client
            .create_exec(&self.containerd_name, exec_config)
            .await?;
        let resu = docker_client
            .start_exec(&msg_id.id, None::<StartExecOptions>)
            .await?;
        info!("{:?}", resu);
        // let exec = docker_client
        //     .attach_container(&self.containerd_name, Some(attach_options))
        //     .await?;
        Ok(())
    }

    async fn build_create_and_run(&self) -> Result<()> {
        buildimg::docker_run_buildkit::git2build(&self.docker_build_url, &self.image_name);
        self.run_devlang_env().await?;
        Ok(())
    }

    pub async fn run_devlang_env(&self) -> Result<()> {
        let pc: Vec<String> = self
            .cmd
            .clone()
            .into_iter()
            .map(|sd| sd.to_string())
            .collect();

        //expose ports
        let uphash: HashMap<(), ()> = HashMap::new();
        let mut kayate = HashMap::new();
        kayate.insert("8022/TCP".to_string(), uphash.clone());
        // kayate.insert("8022/UDP".to_string(), uphash);

        let hostconfig = HostConfig {
            publish_all_ports: Some(true),
            ..Default::default()
        };

        let config: Config<String> = Config {
            image: Some(self.image_name.to_owned()),
            cmd: Some(pc),
            host_config: Some(hostconfig),
            exposed_ports: Some(kayate),
            ..Default::default()
        };

        let config1 = config.clone();
        let options = Some(CreateContainerOptions {
            name: &self.containerd_name,
            // attach_stdout: Some(true),
        });
        let docker = bollard::Docker::connect_with_socket_defaults()?;
        let start_options = Some(StartContainerOptions {
            detach_keys: "ctrl-^",
        });
        let resu = docker.create_container(options, config).await;

        match resu {
            Ok(_) => {
                docker
                    .start_container(&self.containerd_name, start_options)
                    .await?;
            }
            Err(err) => {
                warn!("{}", err);
                let container_name = "devlang1";
                let new_opt: Option<CreateContainerOptions<&str>> = Some(CreateContainerOptions {
                    name: container_name,
                    // attach_stdout: Some(true),
                });
                docker.create_container(new_opt, config1).await?;
                docker
                    .start_container(container_name, start_options)
                    .await?;
            }
        }
        Ok(())
    }
}

impl Default for Cond {
    fn default() -> Self {
        Cond {
            docker_build_url: "https://raw.githubusercontent.com/loyurs/qkrun/master/build_images/dockerfiles/ubuntu20_ssh/Dockerfile".into(),
            image_name: "dede".into(),
            containerd_name: "devlangpod".into(),
            port: "8022".into(),
            cmd: vec!["sleep".into(),"666666".into()],
        }
    }
}

// use futures::{stream::StreamExt, TryStreamExt};
///build a docker container
pub async fn build_image(git_url: &str, image_name: &str) -> Result<()> {
    let client = bollard::Docker::connect_with_socket_defaults()?;
    let build_options = BuildImageOptions {
        dockerfile: "Dockerfile",
        t: "testworld:latest",
        remote: "https://raw.githubusercontent.com/loyurs/qkrun/master/build_images/dockerfiles/tda/Dockerfile",
        ..Default::default()
    };

    // stream.try_for_each_concurrent(MAX_CONCURRENT_JUMPERS, |num| async move {
    //     jump_n_times(num).await?;
    //     report_n_jumps(num).await?;
    //     Ok(())
    // }).await?;
    let mut file = std::fs::File::open("/root/qkrun/tarball.tar.gz").unwrap();
    let mut contents = Vec::new();
    // file.read_buf(buf)
    // file.read_to_end(buf);
    use futures::Stream;
    file.read_to_end(&mut contents).unwrap();
    let streamsss = client.build_image(build_options, None, Some(contents.into()));
    // println!("{:?}", streamsss);
    Ok(())
}

mod test_build {
    use super::{build_image, Cond};

    #[tokio::test]
    async fn test_build_image() {
        // build_image("", "biyuxin");
        let pd = Cond::default();
        build_image(&pd.docker_build_url, "biyuxin").await.unwrap();
    }
}

//可以挂载文件，可以绑定挂载点
#[derive(Default)]
pub struct RunDocker {}

///docker run -v -> Mount结构体 mount type -> BIND
///docker run --mount -> CreateOptions -> volume
impl RunDocker {
    pub async fn docker_run_with_volume_mount(
        &self,
        target_container_mount_point: &str,
        source_host_mount_point: &str,
        container_name: &str,
        image_name: &str,
        start_cmd: Vec<&str>,
    ) -> Result<(), anyhow::Error> {
        let mount_opt = Mount {
            target: Some(target_container_mount_point.to_string()),
            source: Some(source_host_mount_point.to_string()),
            typ: Some(MountTypeEnum::BIND),
            ..Default::default()
        };
        let hostcfg = HostConfig {
            mounts: Some(vec![mount_opt]),
            ..Default::default()
        };
        let docker = bollard::Docker::connect_with_socket_defaults()?;
        let config: Config<&str> = Config {
            image: Some(image_name),
            // cmd: Some(create_cmd),
            host_config: Some(hostcfg),
            cmd: Some(start_cmd),
            ..Default::default()
        };
        let options = Some(CreateContainerOptions {
            name: container_name,
            // ..Default::default()
        });
        docker.create_container(options, config).await?;
        let start_options = Some(StartContainerOptions {
            detach_keys: "ctrl-^",
        });
        //启动容器，并附带启动参数
        docker
            .start_container(container_name, start_options)
            .await?;
        Ok(())
    }
    pub async fn docker_run_with_created_volume(
        &self,
        volume_name: &str,
        container_name: &str,
    ) -> Result<(), anyhow::Error> {
        // let volume_created =
        let docker = Docker::connect_with_socket_defaults()?;
        let myvolume_opt: CreateVolumeOptions<&str> = CreateVolumeOptions {
            name: volume_name,
            ..Default::default()
        };
        docker.create_volume(myvolume_opt).await?;
        let mut volume_mount = HashMap::new();
        let mut ku: HashMap<(), ()> = HashMap::new();
        volume_mount.insert(volume_name.to_string(), ku.clone());
        let config: Config<String> = Config {
            image: Some("ubuntu:20.04".into()),
            // cmd: Some(create_cmd),
            volumes: Some(volume_mount),
            cmd: Some(vec!["sleep".to_string(), "3600".to_string()]),
            ..Default::default()
        };
        let options = Some(CreateContainerOptions {
            name: container_name.to_string(),
        });
        docker.create_container(options, config).await?;
        let start_options = Some(StartContainerOptions {
            detach_keys: "ctrl-^",
        });
        docker
            .start_container(container_name, start_options)
            .await?;
        Ok(())
    }
}

//可接受str和String
fn wock<T: Into<String> + std::fmt::Debug>(haha: T) {
    let pd = haha;
    println!("{:?}", pd);
}

#[test]
fn test_wock() {
    wock("111".to_string());
}

#[tokio::test]
async fn test_docker_run_with_created_volume() {
    let p = RunDocker::default();
    p.docker_run_with_created_volume("haha", "bidaxin")
        .await
        .unwrap();
}
