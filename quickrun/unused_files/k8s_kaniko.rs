use std::{
    io::Stdin,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::Error;
use serde::Serialize;
use tinytemplate::TinyTemplate;
pub static KANIKO: &'static str = r#"apiVersion: v1
kind: Pod
metadata:
  name: {kaniko_build}
spec:
  containers:
    - name: {kaniko_name}
      image: "registry.cn-hangzhou.aliyuncs.com/clouddevs/kanico:latest"
      #imagePullPolicy: IfNotPresent
      #   stdin: false
      #stdinOnce: true
      args:
        - '--dockerfile=Dockerfile'
        - '--context={context_git_url}'
        - '--context-sub-path={context_sub_path}'
        - '--destination={image_name}'
      volumeMounts:
        - name: docker-config
          mountPath: /kaniko/.docker/
  restartPolicy: Never
  volumes:
    - name: docker-config
      configMap:
        name: docker-config
"#;
//docker-config需要配置私有仓库的认证信息
// - '--destination=ccr.ccs.tencentyun.com/tctd/k8k8:latest'

#[derive(Serialize)]
pub struct KanikoBuild {
    kaniko_build: String,
    kaniko_name: String,
    context_git_url: String,
    context_sub_path: String,
    image_name: String,
}

impl KanikoBuild {
    pub fn new(
        kaniko_build: String,
        kaniko_name: String,
        context_git_url: String,
        context_sub_path: String,
        image_name: String,
    ) -> Self {
        KanikoBuild {
            kaniko_build,
            kaniko_name,
            context_git_url,
            context_sub_path,
            image_name,
        }
    }
    pub fn generate_kaniko_build_yaml(&self) {
        let mut tt = TinyTemplate::new();
        tt.add_template("hello", KANIKO).unwrap();
        let rendered = tt.render("hello", &self).unwrap();
    }
}

pub fn kaniko_build() -> Result<(), Error> {
    let mut du_output_child = Command::new("echo")
        .arg(WO)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(du_output) = du_output_child.stdout.take() {
        let sort_output_child = Command::new("xargs")
            .arg("-0")
            .arg("kubectl")
            .arg("run")
            .arg("kaniko")
            .arg("--image")
            .arg("registry.cn-hangzhou.aliyuncs.com/clouddevs/kanico:latest")
            // .arg("-ndefault")
            .arg("--overrides")
            .stdin(du_output)
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped())
            .spawn()?;

        let haout = sort_output_child.wait_with_output()?;
        du_output_child.wait()?;
        println!("{}", String::from_utf8(haout.stdout).unwrap());
    }
    Ok(())
}

static WO: &str = r#"{
  "apiVersion":"v1",
  "kind":"Pod",
  "metadata":{
      "name":"kaniko"
  },
  "spec":{
      "containers":[
          {
              "name":"kaniko",
              "image":"registry.cn-hangzhou.aliyuncs.com/clouddevs/kanico:latest",
              "args":[
                  "--dockerfile=Dockerfile",
                  "--context=git://github.com/loyurs/qkrun.git#refs/heads/master",
                  "--context-sub-path=dockerfiles/test/",
                  "--destination=ccr.ccs.tencentyun.com/tctd/k8k8:latest"
              ],
              "volumeMounts":[
                  {
                      "name":"docker-config",
                      "mountPath":"/kaniko/.docker/"
                  }
              ]
          }
      ],
      "restartPolicy":"Never",
      "volumes":[
          {
              "name":"docker-config",
              "configMap":{
                  "name":"docker-config"
              }
          }
      ]
  }
}"#;

use k8s_openapi::api::apps::v1::StatefulSet;
///job build
use k8s_openapi::api::{batch::v1::Job, core::v1::Pod};
// use k8s_openapi::api::storage::v1::StorageClass;
use k8s_openapi::api::core::v1::{PersistentVolume, PersistentVolumeClaim};
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams},
    runtime::wait::{await_condition, conditions},
    Client, Discovery,
};
use tracing::info;
///使用storageClass

pub async fn storagep() -> anyhow::Result<()> {
    Ok(())
}

///创建pv
pub async fn pvpv() -> anyhow::Result<()> {
    //  let client = Client::try_default().await?;
    //  let pods: Api<Pod> = Api::namespaced(client, "apps");
    //  let patch = serde_json::json!({
    //     "apiVersion": "v1",
    //     "kind": "PersistentVolume",
    //     "metadata": {
    //       "name": "rust-pv",
    //       "labels": {
    //         "type": "local"
    //       }
    //     },
    //     "spec": {
    //       "storageClassName": "rust-pv-config",
    //       "capacity": {
    //         "storage": "20Gi"
    //       },
    //       "accessModes": [
    //         "ReadWriteOnce"
    //       ],
    //       "persistentVolumeReclaimPolicy": "Retain",
    //       "hostPath": {
    //         "path": "/mnt/rust/rust-config"
    //       }
    //     }
    //   });
    //  let params = PatchParams::apply("myapp");
    //  let patch = Patch::Apply(&patch);
    //  let o_patched = pods.patch("blog", &params, &patch).await?;

    tracing_subscriber::fmt::init();
    info!("start");
    let client = Client::try_default().await?;
    // let pv: Api<PersistentVolume> = Api::default_namespaced(client);
    info!("middle");
    let kaya = serde_json::from_value(serde_json::json!({
      "apiVersion": "v1",
      "kind": "PersistentVolume",
      "metadata": {
        "name": "rust-pv",
        "labels": {
          "type": "local"
        }
      },
      "spec": {
        "storageClassName": "rust-pv-config",
        "capacity": {
          "storage": "20Gi"
        },
        "accessModes": [
          "ReadWriteOnce"
        ],
        "persistentVolumeReclaimPolicy": "Retain",
        "hostPath": {
          "path": "/mnt/rust/rust-config"
        }
      }
    }))?;
    let pv =
        k8s_openapi::api::core::v1::PersistentVolume::create(&kaya, Default::default()).unwrap();

    // let pp = pv.delete(&PostParams::default(),&kaya).await?;
    // let cp = pv.delete("rust-pv", &DeleteParams::default()).await?;
    // pv.create(&PostParams::default(), &kaya).await?;
    // let ppp = String::from_utf8_lossy([..]);
    println!("{:?}", &pv.0);
    // println!("{:?}", String::from_utf8_lossy(&pv.1));
    Ok(())
}

///创建pvc
pub async fn pvcpvc() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let pvc: Api<PersistentVolumeClaim> = Api::default_namespaced(client);
    let kaya = serde_json::from_value(serde_json::json!(
        {
            "apiVersion":"v1",
            "kind":"PersistentVolumeClaim",
            "metadata":{
                "name":"rust-pvc-config"
            },
            "spec":{
                "storageClassName":"rust-pv-config",
                "accessModes":[
                    "ReadWriteOnce"
                ],
                "resources":{
                    "requests":{
                        "storage":"20Gi"
                    }
                }
            }
        }
    ))?;

    pvc.create(&PostParams::default(), &kaya).await?;
    Ok(())
}

///创建sts
pub async fn coding_platform() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let sts: Api<StatefulSet> = Api::default_namespaced(client);
    let data = serde_json::from_value(serde_json::json!({}))?;
    sts.create(&PostParams::default(), &data).await?;

    Ok(())
}

///创建pv
pub async fn create_pv(client: Client) -> anyhow::Result<()> {
    Ok(())
}
///创建pvc
///创建service

///通过k8s api控制
pub async fn kaniko_job() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let client = Client::try_default().await?;
    let jobs: Api<Job> = Api::default_namespaced(client);
    info!("Creating job");
    let name = "empty-job";
    let data = serde_json::from_value(serde_json::json!({
        "apiVersion":"batch/v1",
        "kind":"Job",
        "metadata":{
            "name":"kaniko"
        },
        "spec":{
            "template": {
            "metadata": {
                "name": "empty-job-pod"
            },
            "spec": {
            "containers":[
                {
                    "name":"kaniko",
                    "image":"registry.cn-hangzhou.aliyuncs.com/clouddevs/kanico:latest",
                    "args":[
                        "--dockerfile=Dockerfile",
                        "--context=git://github.com/loyurs/qkrun.git#refs/heads/master",
                        // "--context-sub-path=dockerfiles/test/",
                        "--destination=ccr.ccs.tencentyun.com/tctd/k8k88:latest"
                    ],
                    "volumeMounts":[
                        {
                            "name":"docker-config",
                            "mountPath":"/kaniko/.docker"
                        }
                    ]
                }
            ],
            "restartPolicy":"Never",
            "volumes":[
                {
                    "name":"docker-config",
                    "configMap":{
                        "name":"docker-config"
                    }
                }
            ]
        }
    }
    }
    }))?;
    jobs.create(&PostParams::default(), &data).await?;

    info!("Waiting for job to complete");
    let cond = await_condition(jobs.clone(), name, conditions::is_job_completed());
    let _ = tokio::time::timeout(std::time::Duration::from_secs(20), cond).await?;
    info!("Cleaning up job record");
    jobs.delete(name, &DeleteParams::background()).await?;
    Ok(())
}

//直接调用请求
// async fn applykaya(client: Client, discovery: &Discovery) -> Result<(),Error> {
//     let ssapply = PatchParams::apply("kubectl-light").force();
//     let yaml = std::fs::read_to_string("/home").unwrap();
//     for doc in multidoc_deserialize(&yaml)? {
//         let obj: DynamicObject = serde_yaml::from_value(doc)?;
//         let gvk = if let Some(tm) = &obj.types {
//             GroupVersionKind::try_from(tm)?
//         } else {
//             bail!("cannot apply object without valid TypeMeta {:?}", obj);
//         };
//         let name = obj.name_any();
//         if let Some((ar, caps)) = discovery.resolve_gvk(&gvk) {
//             let api = dynamic_api(ar, caps, client.clone(), &self.namespace, false);
//             trace!("Applying {}: \n{}", gvk.kind, serde_yaml::to_string(&obj)?);
//             let data: serde_json::Value = serde_json::to_value(&obj)?;
//             let _r = api.patch(&name, &ssapply, &Patch::Apply(data)).await?;
//             info!("applied {} {}", gvk.kind, name);
//         } else {
//             warn!("Cannot apply document for unknown {:?}", gvk);
//         }
//     }
//     Ok(())
// }
