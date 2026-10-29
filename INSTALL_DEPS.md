# Установка системных зависимостей

Vibe Timer использует **Tauri 2**, которому на Linux нужен WebKitGTK 4.1
(в Tauri 1 требовался 4.0, которого больше нет в актуальных Fedora).

## Fedora / RHEL

```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  alsa-lib-devel \
  gtk3-devel \
  openssl-devel \
  curl \
  wget \
  file
```

## Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev
```

> Для Ubuntu 22.04 и новее доступен `libwebkit2gtk-4.1-dev`.
> В более старых релизах может понадобиться `libwebkit2gtk-4.0-dev`,
> но Tauri 2 рассчитан на 4.1.

## Windows

Для Windows 10/11 дополнительных системных зависимостей нет.
Убедитесь, что установлены:

- WebView2 Runtime (обычно уже есть в Windows 10/11);
- Visual Studio 2022 Build Tools с компонентом «Desktop development with C++»;
- Windows 10/11 SDK.

## Rust и Node.js

```bash
# Rust (если ещё не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Node.js 18+ и npm
node --version
npm --version
```

## Проверка и запуск

```bash
npm install
npm run tauri dev
```

Production-сборка:

```bash
npm run tauri build
```

Если установщик (AppImage/DEB/RPM) не нужен, а нужно только проверить
компиляцию приложения:

```bash
npm run tauri build -- --no-bundle
```
