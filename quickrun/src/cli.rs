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

// #[derive(Subcommand)]
// enum DevLang {
//     Rust,
//     Go,
//     Java,
//     Python,
//     C,
//     Cpp,
//     Javascript,
// }

#[derive(Subcommand)]
enum Commands {
    ///构建镜像,或生成配置模板<docker>
    KanikoDocker {
        ///读取位置配置文件，构建镜像
        #[clap(long)]
        config_path: Option<PathBuf>,
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
        config_path: Option<PathBuf>,
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
}
use anyhow::Error;
use serde_yaml::Value;
use tracing::{error, info, warn};

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
        self, new_docker_registry, new_kaniko_build, new_statefulset_codeserver,
        new_vscode_server_pod,
    },
    tools,
};

///starting the cli parser
pub async fn cli_run() -> Result<(), Error> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::KanikoDocker {
            config_path,
            generate,
        } => {
            match config_path {
                Some(k) => {
                    kaniko_docker_build_image_with_config_file(k.to_path_buf())
                        .await
                        .unwrap();
                }
                None => {
                    error!("Please use -h get help")
                }
            }
            if let Some(a) = generate {
                kaniko_docker_config_template_generate(a).unwrap();
                warn!("generate config file successful");
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
        Commands::KanikoPod {
            config_path,
            generate,
        } => {
            //创建一个configmap
            // cargo run  build --git git://github.com/loyurs/qkrun.git#refs/heads/master --subpath build_images/dockerfiles/vscode_server_only/ --registry-api ccr.ccs.tencentyun.com --image-name ccr.ccs.tencentyun.com/tctd/k888:latest --passwd-registry  --user-registry 100016367772 --name kaka
            match config_path {
                Some(k) => {
                    let d = kaniko_pod::kaniko_pod_get_config_from_path(k.to_path_buf())
                        .await
                        .unwrap();
                    apply_delete("create", &d.0).await?;
                    apply_delete("create", &d.1).await?;
                }
                None => {
                    error!("Please input the config path")
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
