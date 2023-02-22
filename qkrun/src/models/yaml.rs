//启动一个vscode-web-server
use crate::{k8sapply::apply_delete, cli::random_string};
use serde::Serialize;
use tinytemplate::TinyTemplate;
use tracing::info;
///配置模板文件参数
#[derive(Serialize, Default)]
pub struct VscodeServerPod<T:Into<String>> {
    ///pv卷名称
    pub pv_name: T,
    pub host_path: T,
    pub pvc_name: T,
    pub statefulset_name: T,
    pub vscode_password: T,
}

//不带web界面的vscode
#[derive(Serialize, Default)]
pub struct VscodeWithoutWeb {
    // pub generate_str: String,
    pub cpu_limit: String,
    pub memory_limit: String,
}
pub static VSCODE: &str = r#"kind: StatefulSet
apiVersion: apps/v1
metadata:
  name: code-server
  namespace: default
  labels:
    app: code-server
spec:
  replicas: 1
  selector:
    matchLabels:
      app: code-server
  template:
    metadata:
      creationTimestamp: null
      labels:
        app: code-server
    spec:
      containers:
        - name: container
          image: registry.cn-hangzhou.aliyuncs.com/clouddevs/ubuntu20.04:ssh
          ports:
            - name: http-0
              containerPort: 8022
              protocol: TCP
          resources:
            limits:
              cpu: "0.5"
              memory: 100Mi
            requests:
              cpu: "0.1"
              memory: 50Mi
          volumeMounts:
            - name: localgene
              mountPath: /home
          livenessProbe:
            tcpSocket:
              port: 8022
            timeoutSeconds: 1
            periodSeconds: 10
            successThreshold: 1
            failureThreshold: 3
          terminationMessagePath: /dev/termination-log
          terminationMessagePolicy: File
          imagePullPolicy: IfNotPresent
          securityContext:
            privileged: true
            runAsNonRoot: false
            allowPrivilegeEscalation: true
      restartPolicy: Always
      terminationGracePeriodSeconds: 30
      dnsPolicy: ClusterFirst
      serviceAccountName: default
      serviceAccount: default
      schedulerName: default-scheduler
  volumeClaimTemplates:
    - kind: PersistentVolumeClaim
      apiVersion: v1
      metadata:
        name: localgene
        namespace: default
        creationTimestamp: null
      spec:
        accessModes:
          - ReadWriteOnce
        resources:
          requests:
            storage: 10Gi
        storageClassName: local
        volumeMode: Filesystem
  serviceName: code-server-pcwx
  podManagementPolicy: OrderedReady
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      partition: 0
  revisionHistoryLimit: 10"#;
pub fn ubuntu_server_withoutweb(vscodett: VscodeWithoutWeb) -> String {
    let mut tt = TinyTemplate::new();
    tt.add_template("vscode", VSCODE).unwrap();
    tt.render("vscode", &vscodett).unwrap()
}

#[test]
fn ubuntu_server_withoutweb_test() {
    let ubuntu_server = VscodeWithoutWeb {
        // generate_str: random_string(),
        cpu_limit: "1.5".to_string(),
        memory_limit: "1000Mi".to_string(),
    };
    let d = ubuntu_server_withoutweb(ubuntu_server);
    println!("{d}");
}

pub static VSCODE_SERVER_POD: &str = r#"---
kind: PersistentVolume
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
      app: {statefulset_name}
  template:
    metadata:
      labels:
        app: {statefulset_name}
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
          protocol: TCP
          name: code-server
        - containerPort: 8022
          protocol: TCP
          name: ssh-port
        volumeMounts:
        - name: rust-config
          mountPath: /config
      dnsPolicy: ClusterFirst
      volumes:
      - name: rust-config
        persistentVolumeClaim:
          claimName: {pvc_name}
---
apiVersion: v1
kind: Service
metadata:
  name: {statefulset_name}-svc
spec:
  type: NodePort
  selector:
    app: {statefulset_name}
  ports:
    - protocol: TCP
      port: 8022
      targetPort: 18022
      name: ssh-port
    - protocol: TCP
      port: 8443
      name: code-server
      targetPort: 18443"#;

///为此模板进行实例化
pub fn new_vscode_server_pod<T:Into<String> + Serialize>(
    template: &str,
    pv_name: T,
    pvc_name: T,
    host_path: T,
    statefulset_name: T,
    vscode_password: T,
) -> String {
    let mut tt = TinyTemplate::new();
    tt.add_template("vscode_server_pod", template).unwrap();
    let kaka = VscodeServerPod {
        pv_name,
        host_path,
        pvc_name,
        statefulset_name,
        vscode_password,
    };
    tt.render("vscode_server_pod", &kaka).unwrap()
}

#[test]
fn test_new_vscode_server_pod() {}

///kaniko构建模板
//context_git_url 为   String(- '--context-sub-path=).push_str("git_url")
//sub_path
#[derive(Serialize, Default)]
struct KanikoBuild {
    kaniko_build_name: String,
    context_git_url: String,
    sub_path: String, //r#"- '--context-sub-path=dockerfiles/pp/'"#   sub_path的格式
    image_name: String,
    cm_name: String,
    ns: String,
}

pub static KANIKO_BUILD: &str = r#"apiVersion: v1
kind: Pod
metadata:
  name: {kaniko_build_name}
  namespace: {ns}
spec:
  containers:
    - name: {kaniko_build_name}
      image: "registry.cn-hangzhou.aliyuncs.com/clouddevs/kanico:latest"
      imagePullPolicy: IfNotPresent
      #   stdin: false
      #stdinOnce: true
      args:
        - '--dockerfile=Dockerfile'
        - '--context={context_git_url}'
        - '--context-sub-path={sub_path}'
        - '--destination={image_name}'
      volumeMounts:
        - name: docker-config
          mountPath: /kaniko/.docker/
  restartPolicy: Never
  volumes:
    - name: docker-config
      configMap:
        name: {cm_name}
"#;

pub fn new_kaniko_build(
    template: &str,
    kaniko_build_name: String,
    context_git_url: String,
    sub_path: String,
    image_name: String,
    cm_name: String,
    ns: String,
) -> String {
    let kaniko_build = KanikoBuild {
        kaniko_build_name,
        context_git_url,
        sub_path,
        image_name,
        cm_name,
        ns,
    };
    let mut tt = TinyTemplate::new();
    tt.add_template("kaniko_build", template).unwrap();
    tt.render("kaniko_build", &kaniko_build).unwrap()
}

#[test]
fn test_new_kaniko_build() -> Result<(), anyhow::Error> {
    // let pp = new_kaniko_build(KANIKO_BUILD, "mykani".into(), "git://github.com/loyurs/qkrun.git#refs/heads/master".into(), "build_images/dockerfiles/code_server_with_ssh/".into(),"ccr.ccs.tencentyun.com/loyu/litong1:latest".into(),"docker-config".into(),"default".into());
    // println!("{}", pp);
    let pp = new_kaniko_build(
        KANIKO_BUILD,
        "mykani".into(),
        "git://github.com/loyurs/posmtp.git#".into(),
        "".into(),
        "ccr.ccs.tencentyun.com/loyu/litong1:latest".into(),
        "docker-config".into(),
        "default".into(),
    );

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(apply_delete("create", &pp))?;
    Ok(())
}

#[derive(Serialize)]
pub struct DockerRegistry {
    based64: String,
    api_url: String,
    cm_name: String,
    ns: String,
}
///docker 私有仓库凭证
pub static DOCKER_REGISTRY: &str = r#"apiVersion: v1
data:
  config.json: |
    \{
        "auths": \{
            "{api_url}": \{
                "auth": "{based64}"
            }
        }
    }
kind: ConfigMap
metadata:
  name: {cm_name}
  namespace: {ns}"#;

// 如果模板包含左大括号 （），则必须使用前导字符对大括号进行转义。例如：{\

//   h2 \{
//       font-size: {fontsize};
//   }

fn based64go<'a>(user: &'a str, passwd: &'a str) -> String {
    let c = DOCKER_REGISTRY;
    let based64 = base64::encode(format!("{}:{}", user, passwd));
    based64
}
///生成添加了用户名密码的configmap yaml配置文件
pub fn new_docker_registry(
    user: &str,
    password: &str,
    api_url: String,
    template: &str,
    configmap_name: String,
    namespace: String,
) -> String {
    info!(
        "namespace:{}, configmap_name: {} should be the same with mounted pod",
        namespace, configmap_name
    );
    let based64 = based64go(user, password);
    let docker_registry = DockerRegistry {
        based64,
        api_url,
        cm_name: configmap_name,
        ns: namespace,
    };
    let mut tt = TinyTemplate::new();
    tt.add_template("docker_registry", template).unwrap();
    tt.render("docker_registry", &docker_registry).unwrap()
}

#[test]
fn test_new_docker_registry() {
    let pc = new_docker_registry(
        "100016367772",
        "***",
        "ccr.ccs.tencentyun.com".into(),
        DOCKER_REGISTRY,
        "docker-config".into(),
        "default".into(),
    );
    // println!("{}",pc);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(apply_delete("create", &pc))
        .unwrap();
}

//根据信息创建secret
//https://github.com/GoogleContainerTools/kaniko#kubernetes-secret
#[derive(Serialize)]
struct KanikoSecret {
    json_based64: String,
    secret_name: String,
}

static KANIKO_SECRET: &str = r#"apiVersion: v1
data:
  config.json: {json_based64}
kind: Secret
metadata:
  name: {secret_name}
  namespace: default
type: Opaque"#;

pub fn new_secret(
    template: &str,
    json_based64: String,
    secret_name: String,
) -> Result<String, anyhow::Error> {
    let mut tt = TinyTemplate::new();
    tt.add_template("secret", template)?;
    let kaniko_secret = KanikoSecret {
        json_based64,
        secret_name,
    };
    let secret = tt.render("secret", &kaniko_secret)?;
    Ok(secret)
}

#[test]
fn test_new_secret() {
    // let sc = new_secret(KANIKO_SECRET, "213", "kate".into()).unwrap();
}

#[derive(Serialize)]
struct StatefulSetPod {
    ///sts 的名字
    statefulset_name: String,
    ///selector名
    app_name: String,
    ///启动容器web的密码
    passwd: String,
}
// # for versions before 1.9.0 use apps/v1beta2
static STATEFULSET_POD: &str = r#"apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {statefulset_name}
spec:
  serviceName: {statefulset_name}
  replicas: 1
  selector:
    matchLabels:
      app: {app_name}
  template:
    metadata:
      labels:
        app: {app_name}
    spec:
      containers:
      - name: {app_name}
        image: registry.cn-hangzhou.aliyuncs.com/clouddevs/code-server-vscode:latest
        imagePullPolicy: IfNotPresent
        env:
        - name: PASSWORD
          value: {passwd}
        - name: SUDO_PASSWORD
          value: {passwd}
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
        - name: cfv
          mountPath: /config
      dnsPolicy: ClusterFirst
  volumeClaimTemplates:
  - metadata:
      name: cfv
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 1Gi
---
apiVersion: v1
kind: Service
metadata:
  name: {statefulset_name}svc
spec:
  type: NodePort
  selector:
    app: {app_name}
  ports:
      # 默认情况下，为了方便起见，`targetPort` 被设置为与 `port` 字段相同的值。
    - port: 8022
      targetPort: 8022
      name: ssh-port
      # 可选字段
      # 默认情况下，为了方便起见，Kubernetes 控制平面会从某个范围内分配一个端口号（默认：30000-32767）
      # nodePort: 30000
    - port: 8443
      name: code-server
      targetPort: 8443
      # nodePort: 30443"#;

pub fn new_statefulset_codeserver(
    statefulset_name: String,
    app_name: String,
    passwd: String,
) -> Result<String, anyhow::Error> {
    let sts = StatefulSetPod {
        ///sts 的名字
        statefulset_name,
        app_name,
        passwd,
    };
    let mut tt = TinyTemplate::new();
    tt.add_template("sts", STATEFULSET_POD)?;
    let yaml = tt.render("sts", &sts)?;

    Ok(yaml)
}

static RESOURCEQUOTA: &str = r#"apiVersion: v1
kind: ResourceQuota
metadata:
  name: {rs_quota_name}
  namespace: {namespace}
spec:
  hard:
    requests.cpu: "{req_cpu}"
    requests.memory: {req_mem}M
    limits.cpu: "{limit_cpu}"
    limits.memory: {limit_mem}M"#;

#[derive(serde::Serialize)]
pub struct ResourecQuota<T> {
    rs_quota_name: T,
    namespace: T,
    req_mem: T,
    req_cpu: T,
    limit_mem: T,
    limit_cpu: T,
}

///创建namespace资源限制
///memory ("1000","2000") cpu_core ("0.5","1.5")
pub fn new_namespace_resourcequota(
    rs_quota_name: &str,
    memory: (&str, &str),
    cpu_core: (&str, &str),
    namespace: &str,
) -> Result<String, anyhow::Error> {
    let mut tt = TinyTemplate::new();
    let resource_quota = ResourecQuota {
        rs_quota_name,
        namespace,
        req_cpu: cpu_core.0,
        req_mem: memory.0,
        limit_cpu: cpu_core.1,
        limit_mem: memory.1,
    };
    tt.add_template("res_quota", RESOURCEQUOTA)?;
    let yaml = tt.render("res_quota", &resource_quota)?;
    Ok(yaml)
}

#[test]
fn new_namespace_resourcequota_test() {
    let td = new_namespace_resourcequota("kaka", ("100", "1000"), ("0.5", "1.8"), "hello").unwrap();
    println!("{td}");
}
