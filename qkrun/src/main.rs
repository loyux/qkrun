use kube::{Client, Discovery};
use qkrun::{
    k8sapply::MyApiForApplyDelete,
    models::yaml::{ubuntu_server_withoutweb, VscodeWithoutWeb, new_statefulset_codeserver, new_vscode_server_pod}, cli::{random_string, build_cli_run},
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    //new client->
    let client = Client::try_default().await?;
    let discovery = Discovery::new(client.clone()).run().await?;
    let myc = MyApiForApplyDelete { client, discovery };
    build_cli_run(myc).await;
  // myc.delete_from_yaml(rr).await.unwrap();
    Ok(())
}
