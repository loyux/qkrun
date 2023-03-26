use kube::{Client, Discovery};
use qkrun::{
    cli::{build_cli_run, random_string},
    k8sapply::MyApiForApplyDelete,
    models::yaml::{
        new_statefulset_codeserver, new_vscode_server_pod, ubuntu_server_withoutweb,
        VscodeWithoutWeb,
    },
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    //new client->
    // let client = Client::try_default().await?;
    // let discovery = Discovery::new(client.clone()).run().await?;
    // let myc = MyApiForApplyDelete { client, discovery };
    build_cli_run().await;
    // myc.delete_from_yaml(rr).await.unwrap();
    Ok(())
}
