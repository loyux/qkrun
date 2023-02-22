use clap::{arg, command, Arg, Command};

pub async fn build_cli_run(myapi:MyApiForApplyDelete) {
    let matches = command!() // requires `cargo` feature
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("generate")
                .about("generate Dockerfile or yaml files")
                .subcommand_required(true)
                .subcommand(Command::new("dockerfile").about("build dockerfile"))
                .subcommand(
                    Command::new("resource_quota")
                        .about("namespace资源限制配置文件")
                        .arg(arg!(--resquota [NAME]  "hess"))
                        .arg(arg!(--cpu_req [CORS] "cpu cores request"))
                        .arg(arg!(--cpu_limiy [CORS_LIMIT] "cpu cores limit"))
                )
                .subcommand(
                        Command::new("sts_code_server")
                        .about("code-server创建配置文件")
                        .arg(arg!(--resquota [NAME] "hess"))
                        .arg(Arg::new("hellow").short('c').help("szadsad"))
                        .arg(Arg::new("workld").short('w').help("sad")),)
                )
        .subcommand(
            Command::new("start")
            .about("run pod or container quickly")
                .subcommand_required(true)
                .subcommand(
                    Command::new("code-server")
                        .about("start a statefulset pod in k8s")
                        .arg(arg!(--operate [OPERATE] "create or delete"))
                        .arg(arg!(--stsname [NAME] "a required file for the configuration and no short"))
                        .arg(arg!(--pvc [PVC] "pvc name"))
                        .arg(arg!(--pv [PV] "pv name"))
                        .arg(arg!(--volume [VOLUME] "hostpath volume"))
                        .arg(arg!(--password [PASSWORD] "container root password"))
                )
                .subcommand(
                    Command::new("ubuntu-server")
                            .about("start a ubuntu-server with ssh 8022 port")
                            .arg(arg!(--pd "helo")))
        )
        .get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => match sub_matches.subcommand() {
            Some(("code-server", sub_sub_matches)) => {
                let k = new_vscode_server_pod(
                    &VSCODE_SERVER_POD,
                    sub_sub_matches.get_one::<String>("pv").unwrap(),
                    sub_sub_matches.get_one::<String>("pvc").unwrap(),
                    sub_sub_matches.get_one::<String>("volume").unwrap(),
                    sub_sub_matches.get_one::<String>("stsname").unwrap(),
                    sub_sub_matches.get_one::<String>("password").unwrap(),
                );
                println!("{k}");
                if sub_sub_matches.get_one::<String>("operate").unwrap().to_uppercase() ==  "CREATE".to_string() {
                    myapi.apply(&k).await.unwrap();
                }
                else if sub_sub_matches.get_one::<String>("operate").unwrap().to_uppercase() ==  "DELETE".to_string() {
                    myapi.delete_from_yaml(&k).await.unwrap();
                } else {
                    panic!("Error, you shold define create or delete!");
                }
            }
            _ => {
                println!("111")
            }
        },
        // sub_matches.get_one::<String>("NAME")
        Some(("build", sub_matches)) => match sub_matches.subcommand() {
            Some(("pod", sub_sub_matches)) => {
                println!("{sub_sub_matches:?}")
            }
            _ => {
                println!("111")
            }
        },
        Some(("generate", sub_matches)) => match sub_matches.subcommand() {
            Some(("k8syaml", sub_sub_matches)) => {
                println!("{sub_sub_matches:?}");
                let d1 = sub_matches.get_one::<String>("NAME");
            }
            _ => {
                println!("111")
            }
        },

        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

use rand::distributions::Alphanumeric;
///创建随机数
use rand::{thread_rng, Rng};

use crate::k8sapply::MyApiForApplyDelete;
use crate::models::yaml::{new_vscode_server_pod, VSCODE_SERVER_POD};

pub fn random_string() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    // println!("{}", rand_string);
    rand_string.to_lowercase()
}

#[test]
fn rand_string_test() {
    random_string();
}
