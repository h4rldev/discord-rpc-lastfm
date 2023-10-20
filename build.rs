fn build() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("icon.ico")
        .set("InternalName", "Last.fm Discord RPC")
        .set("OriginalFilename", "discord_rpc_lastfm.exe")
        .set("ProductName", "Last.fm Discord RPC")
        .set(
            "FileDescription",
            "Discord Rich Presence for Last.fm scrobbles",
        )
        .set("LegalCopyright", "H4rl, 2023")
        /*.set_manifest(manifest)*/
        .compile()
        .expect("Failed to compile resource file");
}

fn main() {
    if build_target::target_triple()
        .expect("Can't find Target")
        .contains("windows")
    {
        build();
    } else {
        println!("cargo:warning=Not building for Windows, Might not work properly, idk")
    }
}
