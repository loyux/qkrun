// 利用sled kv存储模板yaml文件，按照key进行调用与修改

// use crate::k8sapi::apply_delete;

// use super::models::{self, VscodeServerPod};

// use anyhow::Error;
// use tinytemplate::TinyTemplate;

//storage many yaml templates
// fn write2db_init() -> Result<(), Error> {
//     let db: sled::Db = sled::open("template_db")?;
//     let template_vec = vec![
//         "VSCODE_SERVER_POD",
//         "KANIKO_BUILD",
//         "DOCKER_REGISTRY",
//         "KANIKO_SECRET",
//     ];
//     for one in template_vec {
//         db.insert("vscode_server_pod", models::VSCODE_SERVER_POD)?;
//     }
//     Ok(())
// }
// #[test]
// fn test_write2db() -> Result<(), Error> {
//     write2db_init()?;
//     let db: sled::Db = sled::open("my_db")?;
//     let dd = db.get("vscode_server_pod")?.unwrap();
//     let kt = String::from_utf8_lossy(&dd);
//     let ppp = kt.to_string();
//     println!("{}", ppp);
//     Ok(())
// }

// ///利用数据库模板，定义资源名字
// pub async fn generate_sts_yaml() -> Result<(), Error> {
//     // write2db();
//     let db: sled::Db = sled::open("template_db").unwrap();
//     let dd = db.get("vscode_server_pod").unwrap().unwrap();
//     let kt = String::from_utf8_lossy(&dd);
//     let template_d = kt.to_string();
//     let tt = TinyTemplate::new();
//     let kaka = VscodeServerPod {
//         pv_name: "rupv".into(),
//         host_path: "/mnt/".into(),
//         pvc_name: "rupvc".into(),
//         statefulset_name: "yuxin".into(),
//         vscode_password: "li".into(),
//     };
//     let rendered = tt.render("hello", &kaka).unwrap();
//     // println!("{}",rendered);
//     apply_delete("delete", &rendered).await.unwrap();
//     Ok(())
// }

// #[test]
// fn test_generate_sts_yaml() {
//     tokio::runtime::Runtime::new()
//         .unwrap()
//         .block_on(generate_sts_yaml());
// }
