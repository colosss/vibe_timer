# Выпуск релиза

Релиз собирается и публикуется автоматически в GitHub Actions
(`.github/workflows/release.yml`).

## 1. Поднять версию

Версия должна совпадать в `package.json` и `src-tauri/tauri.conf.json`:

```bash
npm version 0.1.1 --no-git-tag-version
```

Затем поправь `"version"` в `src-tauri/tauri.conf.json` вручную — Tauri
берёт версию оттуда.

## 2. Закоммитить и запушить

```bash
git add package.json package-lock.json src-tauri/tauri.conf.json
git commit -m "chore: версия 0.1.1"
git push origin main
```

## 3. Поставить тег и запушить его

```bash
git tag v0.1.1
git push origin v0.1.1
```

После этого workflow **Release**:

1. соберёт Linux (AppImage, DEB, RPM) на Ubuntu 22.04;
2. соберёт Windows (NSIS `.exe`, MSI) на `windows-latest`;
3. создаст GitHub Release и приложит пакеты, `SHA256SUMS`, `install.sh`
   и `INSTALL.md`.

Ссылки в README ведут на `releases/latest/download/<имя>`, поэтому их не
нужно менять от версии к версии.

## Ручной запуск

Actions → **Release** → **Run workflow** → указать существующий тег
(например, `v0.1.1`).

## Оформление репозитория на GitHub

Чтобы страница выглядела как у крупных проектов:

- **Description:** `Таймер обратного отсчёта с авто-сном/выключением (Tauri 2)`;
- **Topics:** `tauri`, `rust`, `react`, `timer`, `power-management`,
  `linux`, `windows`, `appimage`;
- включи **Releases** (они создаются автоматически) и при желании
  **Discussions**;
- закрепи ссылку на **[Releases](https://github.com/colosss/vibe_timer/releases/latest)**
  в описании репозитория (поле Website).
