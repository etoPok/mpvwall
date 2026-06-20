# mpv-wallpaper

mpv-wallpaper le permite reproducir videos como fondo de pantalla usando mpv. Es una herramienta minima para Hyprland/Wayland.

<https://github.com/user-attachments/assets/57822542-0f9a-4e3c-80a4-d007ef744a7e>

## Dependencias del sistema

### Runtime

```bash
# Arch Linux / Manjaro
sudo pacman -S mpv

# Ubuntu 24.04 / Debian Bookworm
sudo apt install libmpv-dev libmpv2

# Fedora
sudo dnf install mpv-libs
```

### Build

```bash
# Arch Linux
sudo pacman -S mpv pkg-config

# Ubuntu / Debian
sudo apt install libmpv-dev pkg-config build-essential

# Fedora
sudo dnf install mpv-devel pkg-config gcc
```

Verificar que pkg-config encuentra libmpv:

```bash
pkg-config --modversion mpv
# Debe imprimir algo como: 0.37.0
```

## Compilación

```bash
git clone <repo>
cd mpv-wallpaper
cargo build --release
```

## Uso

```bash
# Básico
./target/release/mpv-wallpaper /ruta/al/video.mp4

# O con cargo
cargo run --release -- /ruta/al/video.mp4

# Con logging más verbose
RUST_LOG=mpv_wallpaper=debug ./target/release/mpv-wallpaper video.mp4
```

### CLI Flags

| Flag | Valores | Default | Notas |
|------|---------|---------|-------|
| `-h, --help` | | | Muestra ayuda |
| `<video_path>` | ruta de archivo | requerido | Validado que existe |

## Integración con Hyprland

Añadir a `~/.config/hypr/hyprland.conf`:

```conf
# Iniciar wallpaper al arrancar Hyprland
exec-once = /ruta/a/mpv-wallpaper /ruta/al/video.mp4
```

## Formatos de video recomendados

Para bajo consumo de CPU/GPU como wallpaper:

```bash
# Convertir a H.264 optimizado para loop
ffmpeg -i original.mp4 \
  -c:v libx264 -preset slow -crf 18 \
  -an \
  -movflags +faststart \
  -vf "scale=1920:1080:flags=lanczos" \
  wallpaper.mp4

# AV1 (mejor calidad/tamaño, requiere GPU moderna para hwdec)
ffmpeg -i original.mp4 \
  -c:v libaom-av1 -crf 30 -b:v 0 \
  -an \
  wallpaper.mp4
```

## Limitaciones conocidas

- **Un solo monitor**: no hay lógica multi-output.

- **Resize no implementado**: los cambios de resolución del monitor no
  redimensionan `wl_egl_window`.

## Troubleshooting

### El video no aparece / pantalla negra

```bash
# Verificar logs con debug
RUST_LOG=mpv_wallpaper=debug cargo run --release -- video.mp4 2>&1 | head -30

# Verificar que mpv funciona standalone
mpv --vo=gpu --gpu-context=wayland --no-audio video.mp4
```

### Error "zwlr_layer_shell_v1 not available"

El compositor no soporta layer-shell. Verifica que Hyprland está corriendo
y que `WAYLAND_DISPLAY` apunta al socket correcto:

```bash
echo $WAYLAND_DISPLAY
ls /run/user/$(id -u)/
```

### Crash al arrancar

```bash
RUST_LOG=debug cargo run -- video.mp4 2>&1 | head -50
```
