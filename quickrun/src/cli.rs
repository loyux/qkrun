use std::{fs, io::BufReader, path::PathBuf};

#[warn(unused)]
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "1")]
#[clap(author = "loyu loyurs@163.com")]
#[clap(version)]
#[clap(about = "make a containerd vscode-server", long_about = None)]
// quickrun build/start {}
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ///构建镜像,或生成配置模板<docker>
    KanikoDocker {
        ///读取位置配置文件，构建镜像
        #[clap(long)]
        config: Option<PathBuf>,
        ///指定位置生成模板配置文件
        #[clap(long)]
        generate: Option<PathBuf>,
    },
    ///start container with ssh ubuntu20.04<docker>
    Docker {
        #[clap(short = 'c')]
        container_name: String,
    },
    ///vscode-client remotessh install tools<vscode>
    Plugins {
        #[clap(short = 'l', default_value = "rust")]
        language: String,
    },
    /// 构建镜像,或生成配置模板<kubernetes>
    KanikoPod {
        #[clap(value_parser, long)]
        config: Option<PathBuf>,
        #[clap(value_parser, long)]
        generate: Option<PathBuf>,
    },
    ///start a statefulset <kubernetes>
    Start {
        ///利用此项进行删除
        #[clap(value_parser, long)]
        delete: Option<String>,
        ///k8s pv name, pvc name is pvname-pvc
        #[clap(value_parser, long)]
        pv: String,
        #[clap(value_parser, long)]
        pvc: String,
        ///host_path local mount point ,eg: /home
        #[clap(value_parser, long, short)]
        volume: PathBuf,
        ///container name
        #[clap(value_parser, long, short)]
        stsname: String,
        ///vscode-web password
        #[clap(value_parser, long, short)]
        passwd: String,
    },
    ///start a statefulset container quickly<kubernetes>
    Quick {
        #[clap(value_parser, long, short = 's')]
        ///statefulset名字, -s
        statefulset_name: String,
        #[clap(value_parser, long, short = 'a')]
        ///selector 的名字 -a
        app_name: String,
        #[clap(value_parser, long, short = 'p')]
        ///web url password -p
        passwd: String,
        ///use -d string to delete
        #[clap(value_parser, long, short = 'd')]
        delete: Option<String>,
    },
    ResourceQuota {
        #[clap(value_parser, long = "cr")]
        ///namespace cpu request
        cpu_cores_req: String,
        #[clap(value_parser, long = "mr")]
        ///namespace memory request <Mb>
        memory_req: String,
        #[clap(value_parser, long = "cl")]
        ///namespace cpu limit
        cpu_cores_limit: String,
        #[clap(value_parser, long = "ml")]
        ///namespace memory limits <Mb>
        memory_limit: String,

        #[clap(value_parser, long, short = 'r')]
        resource_quota_name: String,
        #[clap(value_parser, long, short = 'n')]
        namespace: String,
        #[clap(value_parser, long, short = 'd')]
        delete: bool,
    },
}
use anyhow::Error;
use tracing::{info, warn};

use crate::{
    buildimg::{
        kaniko_docker::{
            kaniko_docker_build_image_with_config_file, kaniko_docker_config_template_generate,
        },
        kaniko_pod::{self, kaniko_pod_config_template_generate},
    },
    dockerapi,
    k8sapi::apply_delete,
    templates::models::{
        self, new_docker_registry, new_kaniko_build, new_namespace_resourcequota,
        new_statefulset_codeserver, new_vscode_server_pod,
    },
    tools,
};

///starting the cli parser
pub async fn cli_run() -> Result<(), Error> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::ResourceQuota {
            cpu_cores_req,
            memory_req,
            cpu_cores_limit,
            memory_limit,
            resource_quota_name,
            namespace,
            delete
        } => {
            let ns_res = new_namespace_resourcequota(
                &resource_quota_name,
                (memory_req, memory_limit),
                (cpu_cores_req, cpu_cores_limit),
                namespace,
            )
            .unwrap();
            // apply_delete("crea", yaml)

            if delete == &false {
                apply_delete("create", &ns_res).await?;
            } else {
                apply_delete("delete", &ns_res).await?;
            }
        }
        Commands::KanikoDocker { config, generate } => {
            match config {
                Some(k) => {
                    kaniko_docker_build_image_with_config_file(k.to_path_buf())
                        .await
                        .unwrap();
                }
                None => {
                    warn!("Please use -h get help")
                }
            }
            if let Some(a) = generate {
                kaniko_docker_config_template_generate(a).unwrap();
                info!("generate config file successful");
            }
        }

        Commands::Docker { container_name } => {
            let cnd = dockerapi::runcontainerd::Cond::
            new("https://raw.githubusercontent.com/loyurs/qkrun/master/build_images/dockerfiles/ubuntu20_ssh/Dockerfile".into(),  
            "ccr.ccs.tencentyun.com/loyu/dev:latest".into(),
             container_name.to_owned(),
             "8022".into(), 
             vec!["sleep","36000000"]);
            cnd.run_container().await.unwrap();
        }
        Commands::Plugins { language } => match language.to_uppercase().as_str() {
            "RUST" => {
                tools::rust_extensions_install();
            }
            lang => {
                println!("not suppose {}", lang);
            }
        },
        Commands::Start {
            ///PersistentVolume name
            pv,
            ///PersistenVolumeClaim name
            pvc,
            ///mounted disk path
            volume,
            ///statefulset name
            stsname,
            ///container password
            passwd,
            ///optional if none .create if not none delete
            delete,
        } => {
            let yaml = new_vscode_server_pod(
                models::VSCODE_SERVER_POD,
                pv.into(),
                pvc.to_owned(),
                volume.to_str().unwrap().to_string(),
                stsname.to_owned(),
                passwd.to_owned(),
            );

            if delete.is_none() {
                apply_delete("create", &yaml).await?;
            } else {
                apply_delete("delete", &yaml).await?;
            }
        }
        Commands::KanikoPod { config, generate } => {
            //创建一个configmap
            // cargo run  build --git git://github.com/loyurs/qkrun.git#refs/heads/master --subpath build_images/dockerfiles/vscode_server_only/ --registry-api ccr.ccs.tencentyun.com --image-name ccr.ccs.tencentyun.com/tctd/k888:latest --passwd-registry  --user-registry 100016367772 --name kaka
            match config {
                Some(k) => {
                    let d = kaniko_pod::kaniko_pod_get_config_from_path(k.to_path_buf())
                        .await
                        .unwrap();
                    apply_delete("create", &d.0).await?;
                    apply_delete("create", &d.1).await?;
                }
                None => {
                    warn!("Please input the config path")
                }
            }
            if let Some(p) = generate {
                kaniko_pod_config_template_generate(p).unwrap();
                warn!("generate config file successful");
            }
        }
        Commands::Quick {
            statefulset_name,
            app_name,
            passwd,
            delete,
        } => {
            let sts_yaml = new_statefulset_codeserver(
                statefulset_name.to_owned(),
                app_name.to_owned(),
                passwd.to_owned(),
            )?;
            if delete.is_none() {
                apply_delete("create", &sts_yaml).await?;
            } else {
                apply_delete("delete", &sts_yaml).await?;
            }
        }
    };
    Ok(())
}
