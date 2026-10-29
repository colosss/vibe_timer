# Установка Vibe Timer

В этой папке лежат готовые пакеты. Выбери один способ.

## Вариант 1 — AppImage (один файл, без установки)

Самодостаточный файл: внутри уже есть WebKit, GTK и appindicator.
Ничего устанавливать не нужно, достаточно сделать файл исполняемым:

```bash
chmod +x vibe-timer-0.1.0-x86_64.AppImage
./vibe-timer-0.1.0-x86_64.AppImage
```

Если в системе нет FUSE 2, запусти так:

```bash
./vibe-timer-0.1.0-x86_64.AppImage --appimage-extract-and-run
```

Чтобы добавить приложение в меню (без root):

```bash
./install.sh --appimage
```

## Вариант 2 — Fedora / RHEL (RPM, рекомендуется)

```bash
sudo dnf install ./vibe-timer-0.1.0-1.x86_64.rpm
```

Или через скрипт:

```bash
./install.sh
```

После установки приложение появится в меню как **Vibe Timer**.
Удаление: `sudo dnf remove vibe-timer`.

## Вариант 3 — Ubuntu / Debian (DEB)

```bash
sudo apt install ./vibe-timer_0.1.0_amd64.deb
```

Удаление: `sudo apt remove vibe-timer`.

## Проверка целостности

```bash
sha256sum -c SHA256SUMS
```

## После первого запуска

- Закрытие окна сворачивает приложение в системный трей (по умолчанию).
  Полный выход — меню трея → **Выход**.
- Для сна/гибернации/выключения/перезагрузки на Linux может понадобиться
  polkit-правило — пример есть в корневом `README.md`, раздел
  «Важные замечания».
- «Перезагрузка + Сон» сработает только после следующего входа в систему.
- Настройки и пресеты хранятся в `~/.local/share/com.vibecode.vibe-timer/`.

## Windows

Готового `.exe` здесь нет: установщик Windows собирается на Windows командой
`npm run tauri build` (см. `README.md`). На Linux кросс-сборка установщика
Windows не поддерживается.
