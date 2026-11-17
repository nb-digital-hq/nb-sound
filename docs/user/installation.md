# Instalación

> **Estado:** Implementado. Linux está verificado en un equipo real; Windows y macOS se publican sin haberse probado
> en un equipo real.

Descarga el paquete de tu sistema desde la publicación más reciente de
[nb-digital-hq/nb-sound](https://github.com/nb-digital-hq/nb-sound/releases) y, si quieres, verifícalo antes de
instalar ([verificar una descarga](../developer/releases.md#verificar-una-descarga)).

| Sistema | Paquete | Cómo se instala |
|---|---|---|
| Debian, Ubuntu y derivadas | `.deb` | `sudo apt install ./NB.Sound_1.0.0_amd64.deb` |
| Fedora, openSUSE y derivadas | `.rpm` | `sudo dnf install ./NB.Sound-1.0.0-1.x86_64.rpm` |
| Arch y cualquier otra | `.tar.gz` | `sudo tar -xzf NB.Sound_1.0.0_amd64.tar.gz -C /` (usa el WebKitGTK del sistema) |
| Cualquier Linux, sin instalar | AppImage | Darle permiso de ejecución y abrirla |
| Windows 10 y 11 | `-setup.exe` o `.msi` | Abrirlo. SmartScreen avisa porque no lleva firma de Microsoft: "Más información", "Ejecutar de todas formas" |
| macOS en Apple Silicon | `.dmg` | Arrastrar NB Sound a Aplicaciones. Como no lleva firma de Apple, la primera vez: clic derecho, "Abrir", y confirmar; o en Ajustes del Sistema, Privacidad y seguridad, "Abrir igualmente" |

No hace falta instalar nada más: el motor de audio, la huella acústica y el runtime de inferencia viajan dentro. Lo
grande y opcional (karaoke, análisis musical, pedidos al DJ, aceleración por GPU) se instala desde la propia
aplicación, en Ajustes, Agregados.

## Actualizar

En Ajustes, Acerca de, "Buscar actualizaciones" (con los servicios en línea encendidos). La actualización se descarga,
se comprueba su firma y se instala; tus datos se conservan. Si instalaste NB Sound con NB Launcher, se actualiza desde
NB Launcher.

## Desinstalar

Con el gestor de paquetes de tu sistema, o borrando la AppImage. Desinstalar **no** borra tu carpeta de música. Los
datos de la aplicación (biblioteca, historial, portadas, instrumentales) quedan en su directorio por si vuelves a
instalarla; si quieres borrarlos, están en `~/.local/share/nb-sound` (Linux), `%APPDATA%\NB Sound` (Windows) o
`~/Library/Application Support/NB Sound` (macOS).
