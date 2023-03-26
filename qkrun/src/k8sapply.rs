//利用k8s的api进行资源的创建与删除，集成对yaml文件资源的处理;

use anyhow::{bail, Context, Result};
use kube::{
    api::{Api, DeleteParams, DynamicObject, Patch, PatchParams, ResourceExt},
    core::GroupVersionKind,
    discovery::{ApiCapabilities, ApiResource, Discovery, Scope},
    Client, Resource,
};
use std::path::PathBuf;
use tracing::*;

///利用yaml操作类型进行操作
/// create / delete
pub async fn apply_delete(opearate_type: &str, yaml: &str) -> Result<()> {
    let client = Client::try_default().await?;
    // discovery (to be able to infer apis from kind/plural only)
    let discovery = Discovery::new(client.clone()).run().await?;

    let myapi = MyApiForApplyDelete::new(client, discovery).await;
    if let "create" = opearate_type {
        myapi.apply(yaml).await?;
    }
    if let "delete" = opearate_type {
        myapi.delete_from_yaml(yaml).await?;
    }
    Ok(())
}

pub struct MyApiForApplyDelete {
    pub client: Client,
    pub discovery: Discovery,
}

impl MyApiForApplyDelete {
    pub async fn new(client: Client, discovery: Discovery) -> Self {
        MyApiForApplyDelete { client, discovery }
    }
    //通过路径读取yaml文件
    // pub async fn read_yaml(file_path: PathBuf) -> String {
    //     std::fs::read_to_string(file_path).expect("Read yaml file error")
    // }
    pub async fn apply(&self, yamls: &str) -> Result<(), anyhow::Error> {
        let ssapply = PatchParams::apply("kubectl-light").force();
        for doc in multidoc_deserialize(yamls)? {
            println!("{doc:?}");
            let obj: DynamicObject = serde_yaml::from_value(doc)?;
            let namespace = obj.namespace();
            let gvk = if let Some(tm) = &obj.types {
                GroupVersionKind::try_from(tm)?
            } else {
                bail!("cannot apply object without valid TypeMeta {:?}", obj);
            };
            let name = obj.name_any();
            if let Some((ar, caps)) = self.discovery.resolve_gvk(&gvk) {
                let api = dynamic_api(ar, caps, self.client.clone(), &namespace, false);
                trace!("Applying {}: \n{}", gvk.kind, serde_yaml::to_string(&obj)?);
                let data: serde_json::Value = serde_json::to_value(&obj)?;
                let _r = api.patch(&name, &ssapply, &Patch::Apply(data)).await?;
                info!("applied {} {}", gvk.kind, name);
            } else {
                warn!("Cannot apply document for unknown {:?}", gvk);
            }
        }
        Ok(())
    }

    ///利用namespace
    /// :resource - persistentvolume/Pod/Deployment/StatefulSet/Job and so on
    /// :namespace - kubernetes namespace
    /// :delete_name - 需要删除的资源名字
    pub async fn delete(
        &self,
        resource: String,
        namespace: Option<String>,
        delete_name: String,
    ) -> Result<(), anyhow::Error> {
        let dynamic_api_closure = |ar: ApiResource,
                                   caps: ApiCapabilities,
                                   client: Client,
                                   ns: &Option<String>,
                                   all: bool|
         -> Api<DynamicObject> {
            if caps.scope == Scope::Cluster || all {
                Api::all_with(client, &ar)
            } else if let Some(namespace) = ns {
                Api::namespaced_with(client, namespace, &ar)
            } else {
                Api::default_namespaced_with(client, &ar)
            }
        };
        let (ar, caps) = self
            .resolve_api_resource(&self.discovery, &resource)
            .with_context(|| format!("resource {:?} not found in cluster", resource))?;
        let api = dynamic_api_closure(ar, caps, self.client.clone(), &namespace, false);
        api.delete(&delete_name, &DeleteParams::default()).await?;
        info!(
            "Delete resource: {:?} successful, namespace: {:?}",
            delete_name, namespace
        );
        Ok(())
    }

    ///利用yaml文件进行删除
    pub async fn delete_from_yaml(&self, yaml: &str) -> Result<(), anyhow::Error> {
        for doc in multidoc_deserialize(yaml)? {
            let obj: DynamicObject = serde_yaml::from_value(doc)?;
            let resource = obj.clone().types.unwrap().kind;
            let namespace = obj.clone().namespace();
            let name = obj.meta().name.clone();
            self.delete(resource, namespace, name.unwrap()).await?;
        }
        Ok(())
    }

    ///创建api资源
    fn resolve_api_resource(
        &self,
        discovery: &Discovery,
        name: &str,
    ) -> Option<(ApiResource, ApiCapabilities)> {
        // iterate through groups to find matching kind/plural names at recommended versions
        // and then take the minimal match by group.name (equivalent to sorting groups by group.name).
        // this is equivalent to kubectl's api group preference
        discovery
            .groups()
            .flat_map(|group| {
                group
                    .recommended_resources()
                    .into_iter()
                    .map(move |res| (group, res))
            })
            .filter(|(_, (res, _))| {
                // match on both resource name and kind name
                // ideally we should allow shortname matches as well
                name.eq_ignore_ascii_case(&res.kind) || name.eq_ignore_ascii_case(&res.plural)
            })
            .min_by_key(|(group, _res)| group.name())
            .map(|(_, res)| res)
    }
}

///测试提供默认值的方式
pub fn kata() {
    let pp: Option<&str> = None;
    // let d = pp.ok_or_else(||->&str {"kkt"});
    // let ppc = pp.unwrap_or_else(|| "default");
    let ppc = pp.unwrap_or("default");
    println!("{:?}", ppc);
}

#[test]
fn test_kata() {
    kata();
}

fn dynamic_api(
    ar: ApiResource,
    caps: ApiCapabilities,
    client: Client,
    ns: &Option<String>,
    all: bool,
) -> Api<DynamicObject> {
    if caps.scope == Scope::Cluster || all {
        Api::all_with(client, &ar)
    } else if let Some(namespace) = ns {
        Api::namespaced_with(client, namespace, &ar)
    } else {
        Api::default_namespaced_with(client, &ar)
    }
}

///将yaml多段序列化为单段数组
pub fn multidoc_deserialize(data: &str) -> Result<Vec<serde_yaml::Value>> {
    use serde::Deserialize;
    let mut docs = vec![];
    for de in serde_yaml::Deserializer::from_str(data) {
        docs.push(serde_yaml::Value::deserialize(de)?);
    }
    Ok(docs)
}

#[test]
fn test_multidoc_deserialize() {
    tracing_subscriber::fmt::init();
    let spc = r#"apiVersion: v1
kind: PersistentVolume
metadata:
  name: rust-pv
  labels:
    type: local
spec:
  storageClassName: rust-pv-config
  capacity:
    storage: 20Gi
  accessModes:
    - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  hostPath:
    path: "/mnt/rust/rust-config"
---
apiVersion: v1
kind: PersistentVolume
metadata:
  name: rust-pv111
  labels:
    type: local
spec:
  storageClassName: rust-pv-config
  capacity:
    storage: 20Gi
  accessModes:
    - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  hostPath:
    path: "/mnt/rust/rust-config""#;
    let svec = multidoc_deserialize(spc).unwrap();
    //解析yaml文件，获取想要的类型并进行解析
    for oc in svec {
        let p1 = oc.get("kind").ok_or_else(|| "default");

        if let Ok(value) = p1 {
            println!("{:?}", value);
        }
        if let Err(err) = p1 {
            println!("{:?}", err);
        }
    }
}
