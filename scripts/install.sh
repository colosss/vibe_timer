#!/bin/sh
# Установка Vibe Timer из готовых пакетов.
set -eu

DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

APPIMAGE=""
RPM=""
DEB=""

for f in "$DIR"/vibe-timer-*-x86_64.AppImage "$DIR"/vibe-timer-x86_64.AppImage; do
    if [ -f "$f" ]; then APPIMAGE="$f"; break; fi
done
for f in "$DIR"/vibe-timer-*-1.x86_64.rpm "$DIR"/vibe-timer-x86_64.rpm; do
    if [ -f "$f" ]; then RPM="$f"; break; fi
done
for f in "$DIR"/vibe-timer_*_amd64.deb "$DIR"/vibe-timer_amd64.deb; do
    if [ -f "$f" ]; then DEB="$f"; break; fi
done

usage() {
    cat <<EOF
Установка Vibe Timer

  ./install.sh              установить системный пакет (RPM или DEB)
  ./install.sh --appimage   установить AppImage для текущего пользователя
                            (без прав root, появится в меню приложений)
  ./install.sh --help       показать эту справку
EOF
}

install_appimage_user() {
    if [ -z "$APPIMAGE" ]; then
        echo "Рядом со скриптом не найден AppImage." >&2
        echo "Собери пакеты командой: npm run package" >&2
        exit 1
    fi

    BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
    DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}"
    APP_DIR="$DATA_DIR/applications"
    ICON_DIR="$DATA_DIR/icons/hicolor/128x128/apps"

    install -d "$BIN_DIR" "$APP_DIR" "$ICON_DIR"
    install -m 755 "$APPIMAGE" "$BIN_DIR/vibe-timer.AppImage"
    install -m 644 "$DIR/vibe-timer.png" "$ICON_DIR/vibe-timer.png"

    cat > "$APP_DIR/vibe-timer.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Vibe Timer
Comment=Таймер с автоматическим управлением питанием
Exec=$BIN_DIR/vibe-timer.AppImage
Icon=vibe-timer
Terminal=false
Categories=Utility;
EOF

    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
    fi

    echo "Готово. Приложение установлено для пользователя:"
    echo "  $BIN_DIR/vibe-timer.AppImage"
    echo "Ярлык добавлен в меню приложений (Vibe Timer)."
}

case "${1:-}" in
    --help|-h)
        usage
        ;;
    --appimage|--user)
        install_appimage_user
        ;;
    "")
        if [ -n "$RPM" ] && command -v dnf >/dev/null 2>&1; then
            PACKAGE="$RPM"
            MANAGER="dnf"
        elif [ -n "$DEB" ] && command -v apt-get >/dev/null 2>&1; then
            PACKAGE="$DEB"
            MANAGER="apt-get"
        else
            echo "Подходящий пакетный менеджер не найден, устанавливаю AppImage..." >&2
            install_appimage_user
            exit 0
        fi

        if [ "$(id -u)" -eq 0 ]; then
            SUDO=""
        elif command -v sudo >/dev/null 2>&1; then
            SUDO="sudo"
        else
            echo "Для установки пакета нужны права root." >&2
            echo "Можно установить без прав: ./install.sh --appimage" >&2
            exit 1
        fi

        exec $SUDO "$MANAGER" install -y "$PACKAGE"
        ;;
    *)
        echo "Неизвестный аргумент: $1" >&2
        usage >&2
        exit 2
        ;;
esac
