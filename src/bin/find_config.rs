use directories::ProjectDirs;

fn main() {
    if let Some(proj_dirs) = ProjectDirs::from("com", "vibecoding", "vibe-iptv") {
        println!("Config dir: {:?}", proj_dirs.config_dir());
        println!("Config file: {:?}", proj_dirs.config_dir().join("config.json"));
    } else {
        println!("Could not determine project paths");
    }
}
