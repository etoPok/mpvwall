use std::env;

pub struct Args {
    pub gpu_api: String,
    pub video_path: String,
}

pub fn parse() -> Args {
    let args: Vec<String> = env::args().collect();

    let mut gpu_api = String::from("auto");
    let mut video_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--gpu-api" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --gpu-api requiere un valor (opengl, vulkan, auto)");
                    std::process::exit(1);
                }
                gpu_api = args[i].clone();
                if !["opengl", "vulkan", "auto"].contains(&gpu_api.as_str()) {
                    eprintln!("Error: --gpu-api debe ser 'opengl', 'vulkan' o 'auto'");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                eprintln!("Uso: {} [OPCIONES] <ruta-al-video>", args[0]);
                eprintln!();
                eprintln!("Opciones:");
                eprintln!("  --gpu-api <opengl|vulkan|auto>  API de rendering GPU (default: auto)");
                eprintln!("  -h, --help                      Mostrar esta ayuda");
                eprintln!();
                eprintln!(
                    "Ejemplo: {} --gpu-api vulkan /home/user/wallpaper.mp4",
                    args[0]
                );
                std::process::exit(0);
            }
            _ => {
                if video_path.is_none() {
                    video_path = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    let video_path = match video_path {
        Some(p) => p,
        None => {
            eprintln!("Uso: {} [OPCIONES] <ruta-al-video>", args[0]);
            eprintln!("Ejemplo: {} /home/user/wallpaper.mp4", args[0]);
            std::process::exit(1);
        }
    };

    Args { gpu_api, video_path }
}
