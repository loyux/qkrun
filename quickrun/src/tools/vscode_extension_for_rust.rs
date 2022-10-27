use tracing::info;

//TO make A quickly environment for rust development
//快速安装开发rust所需插件,code命令vscode-client才可以使用
pub fn rust_extensions_install() {
    let extensions = r#"bungcip.better-toml
    errorlens
    CoenraadS.bracket-pair-colorizer-2
    dustypomerleau.rust-syntax
    evgeniypeshkov.syntax-highlighter
    gitpod.gitpod-remote-ssh
    JScearcy.rust-doc-viewer
    lanza.lldb-vscode
    lorenzopirro.rust-flash-snippets
    nyxiative.rust-and-friends
    PolyMeilex.rust-targets
    rust-lang.rust-analyzer
    serayuzgur.crates
    vadimcn.vscode-lldb
    ZhangYue.rust-mod-generator"#;
    let pd: Vec<&str> = extensions.split("\n").into_iter().collect();
    for ii in pd {
        info!("Installing {}", ii);
        println!("{}", ii);
        std::process::Command::new("code")
            .arg("--install-extension")
            .arg(ii)
            .output()
            .unwrap();
    }
}

///Java extensions install function
pub fn java_extensions_install() {}
