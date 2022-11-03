use std::path::PathBuf;

#[warn(unused)]
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "1")]
#[clap(author = "loyu loyurs@163.com")]
#[clap(version = "0.0.1")]
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
    KanikoBuild {},

    ///start a docker container with ssh ubuntu:20.04 && devenv
    Docker {
        #[clap(short = 'c')]
        container_name: String,
    },
    ///vscode-client remotessh install tools
    Devlang {
        #[clap(short = 'l', default_value = "rust")]
        language: String,
    },
    /// use giturl to build images(cri)
    Build {
        #[clap(value_parser, long)]
        name: String,
        #[clap(value_parser, long)]
        git: String,
        #[clap(value_parser, long)]
        subpath: String,
        #[clap(value_parser, long, short)]
        user_registry: String,
        #[clap(value_parser, long)]
        passwd_registry: String,
        #[clap(value_parser, long)]
        registry_api: String,
        #[clap(value_parser, long)]
        image_name: String,
    },
    ///start a statefulset by use kubectl
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
    ///start a statefulset container quickly
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

use crate::{
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
        Commands::KanikoBuild {} => {}
        Commands::Docker { container_name } => {
            let cnd = dockerapi::runcontainerd::Cond::
            new("https://raw.githubusercontent.com/loyurs/qkrun/master/build_images/dockerfiles/ubuntu20_ssh/Dockerfile".into(),  
            "ccr.ccs.tencentyun.com/loyu/dev:latest".into(),
             container_name.to_owned(),
             "8022".into(), 
             vec!["sleep","36000000"]);
            cnd.run_container().await.unwrap();
        }
        Commands::Devlang { language } => match language.to_uppercase().as_str() {
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
        Commands::Build {
            name,
            git,
            subpath,
            user_registry,
            passwd_registry,
            registry_api,
            image_name,
        } => {
            //创建一个configmap
            // cargo run  build --git git://github.com/loyurs/qkrun.git#refs/heads/master --subpath build_images/dockerfiles/vscode_server_only/ --registry-api ccr.ccs.tencentyun.com --image-name ccr.ccs.tencentyun.com/tctd/k888:latest --passwd-registry  --user-registry 100016367772 --name kaka
            let registry_configmap = new_docker_registry(
                user_registry.as_str(),
                passwd_registry.as_str(),
                registry_api.to_string(),
                models::DOCKER_REGISTRY,
                "docker-reg".into(),
                "default".into(),
            );
            let build_yaml = new_kaniko_build(
                models::KANIKO_BUILD,
                name.to_string(),
                git.to_owned(),
                subpath.to_owned(),
                image_name.to_owned(),
                "docker-reg".to_string(),
                "default".to_string(),
            );
            // println!("{}",build_yaml);
            apply_delete("create", &registry_configmap).await?;
            apply_delete("create", &build_yaml).await?;
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
