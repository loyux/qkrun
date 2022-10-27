use std::process::{Command, Stdio};
use tinytemplate::TinyTemplate;
use uuid::Uuid;
//zerotier
use serde::Serialize;

///配置模板文件参数
#[derive(Serialize, Default)]
pub struct CfgYaml {
    pv_name: String,
    host_path: String,
    pvc_name: String,
    statefulset_name: String,
    vscode_password: String,
}
impl CfgYaml {
    pub fn new(
        pv_name: String,
        host_path: String,
        statefulset_name: String,
        vscode_password: String,
    ) -> Self {
        let mut pvc_name = pv_name.clone();
        pvc_name.push_str("-pvc");
        CfgYaml {
            pv_name,
            host_path,
            pvc_name,
            statefulset_name,
            vscode_password,
        }
    }
    pub fn generate_sts_yaml(&self) -> String {
        static STATEFULSET_YAML: &'static str = r#"kind: PersistentVolume
apiVersion: v1
metadata:
  name: {pv_name}
  labels:
    type: local
spec:
  storageClassName: {pv_name}
  capacity:
    storage: 20Gi
  accessModes:
    - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  hostPath:
    path: "{host_path}"
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {pvc_name}
spec:
  storageClassName: {pv_name}
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {statefulset_name}
spec:
  serviceName: {statefulset_name}
  replicas: 1
  selector:
    matchLabels:
      app: rust
  template:
    metadata:
      labels:
        app: rust
    spec:
      containers:
      - name: {statefulset_name}
        image: registry.cn-hangzhou.aliyuncs.com/clouddevs/code-server-vscode:latest
        imagePullPolicy: IfNotPresent
        env:
        - name: PASSWORD
          value: {vscode_password}
        - name: SUDO_PASSWORD
          value: {vscode_password}
        - name: DEFAULT_WORKSPACE
          value: /config/workspace
        - name: PUID
          value: "0"
        - name: PGID
          value: "0"
        - name: HOME
          value: /config
        #挂载点注意，连接vscode-client，默认配置文件在 /root/.vscode-server
        ports:
        - containerPort: 8443
          name: code-server
        - containerPort: 8022
          name: ssh-port
        volumeMounts:
        - name: rust-config
          mountPath: /config
      dnsPolicy: ClusterFirst
      volumes:
      volumes:
      - name: rust-config
        persistentVolumeClaim:
          claimName: {pvc_name}
---
apiVersion: v1
kind: Service
metadata:
  name: service4rust
spec:
  type: NodePort
  selector:
    app: rust
  ports:
    - port: 8022
      targetPort: 8022
      name: ssh-port
    - port: 8443
      name: code-server
      targetPort: 8443
"#;
        let mut tt = TinyTemplate::new();
        tt.add_template("hello", STATEFULSET_YAML).unwrap();
        let rendered = tt.render("hello", &self).unwrap();
        let mut file_name = Uuid::new_v4().to_string();
        file_name.push_str(".yaml");
        std::fs::write(file_name.clone(), rendered).unwrap();
        file_name
    }
    pub fn start_sts(self) {
        let pp = generate_sts_yaml(self);
        Command::new("kubectl")
            .args(&["apply", "-f", &pp])
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .output()
            .unwrap();
    }
}

pub fn generate_sts_yaml(cfg_yaml: CfgYaml) -> String {
    static STATEFULSET_YAML: &'static str = r#"kind: PersistentVolume
apiVersion: v1
metadata:
  name: {pv_name}
  labels:
    type: local
spec:
  storageClassName: {pv_name}
  capacity:
    storage: 20Gi
  accessModes:
    - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  hostPath:
    path: "{host_path}"
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {pvc_name}
spec:
  storageClassName: {pv_name}
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {statefulset_name}
spec:
  serviceName: {statefulset_name}
  replicas: 1
  selector:
    matchLabels:
      app: rust
  template:
    metadata:
      labels:
        app: rust
    spec:
      containers:
      - name: {statefulset_name}
        image: registry.cn-hangzhou.aliyuncs.com/clouddevs/code-server-vscode:latest
        imagePullPolicy: IfNotPresent
        env:
        - name: PASSWORD
          value: {vscode_password}
        - name: SUDO_PASSWORD
          value: {vscode_password}
        - name: DEFAULT_WORKSPACE
          value: /config/workspace
        - name: PUID
          value: "0"
        - name: PGID
          value: "0"
        - name: HOME
          value: /config
        #挂载点注意，连接vscode-client，默认配置文件在 /root/.vscode-server
        ports:
        - containerPort: 8443
          name: code-server
        - containerPort: 8022
          name: ssh-port
        volumeMounts:
        - name: rust-config
          mountPath: /config
      dnsPolicy: ClusterFirst
      volumes:
      volumes:
      - name: rust-config
        persistentVolumeClaim:
          claimName: {pvc_name}
---
apiVersion: v1
kind: Service
metadata:
  name: service4rust
spec:
  type: NodePort
  selector:
    app: rust
  ports:
    - port: 8022
      targetPort: 8022
      name: ssh-port
    - port: 8443
      name: code-server
      targetPort: 8443
"#;
    let mut tt = TinyTemplate::new();
    tt.add_template("hello", STATEFULSET_YAML).unwrap();
    let rendered = tt.render("hello", &cfg_yaml).unwrap();
    let mut file_name = Uuid::new_v4().to_string();
    file_name.push_str(".yaml");
    std::fs::write(file_name.clone(), rendered).unwrap();
    file_name
}

// ///利用生成的yaml，启动k8s
pub fn start_sts() {
    let pp = generate_sts_yaml(CfgYaml::new(
        "haha".into(),
        "/home".into(),
        "kk".into(),
        "li".into(),
    ));
    Command::new("kubectl")
        .args(&["apply", "-f", &pp])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .unwrap();
    //将file删除
    // std::fs::remove_file(&pp).unwrap();
}
pub fn rm_sts(file_name: &str) {
    Command::new("kubectl")
        .args(&["delete", "-f", file_name])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .unwrap();
    //将file删除
    std::fs::remove_file(file_name).unwrap();
}

// pub fn start_and_rm_sts() {
//   start_sts(generate_sts_yaml(CfgYaml::new()).as_str());
//   rm_sts(generate_sts_yaml(CfgYaml::new()).as_str());
// }
