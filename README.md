# 🎵 Rust Audio Player

Кросс-платформенный аудиоплеер с графическим интерфейсом, написанный на Rust.

## ✨ Возможности

- 🎵 Воспроизведение MP3, WAV, FLAC, OGG файлов
- 🎛️ Интерактивная перемотка с помощью слайдера
- 📱 Управление плейлистами 
- 🔊 Регулировка громкости
- ⏯️ Полный набор элементов управления (play/pause/stop/next/previous)
- ⏩ Быстрая перемотка вперед/назад с настраиваемым интервалом
- 🌊 Красивая анимация во время воспроизведения
- 💾 Автоматическое сохранение плейлистов

## 🚀 Быстрый старт

### Windows
```bash
# Сборка релизной версии
cargo build --release

# Запуск
.\target\release\PrakRust.exe
```

### Linux
```bash
# Установка зависимостей (Ubuntu/Debian)
sudo apt install pkg-config libasound2-dev libxcb1-dev libxrandr-dev libxss-dev libxcursor-dev libxcomposite-dev libxdamage-dev libxfixes-dev libxinerama-dev libxi-dev libxrender-dev libxkbfile-dev libxkbcommon-dev libwayland-dev libdbus-1-dev libgtk-3-dev

# Сборка и запуск
cargo build --release
./target/release/PrakRust
```

### macOS
```bash
# Сборка и запуск
cargo build --release
./target/release/PrakRust
```

## 🐧 Тестирование на Linux

### Способ 1: WSL (рекомендуемый)
```bash
# В WSL выполните:
chmod +x test-linux.sh
./test-linux.sh
```

### Способ 2: Docker
```bash
# Сборка Docker образа
docker build -t rust-player .

# Запуск (для тестирования без GUI)
docker run --rm rust-player
```

### Способ 3: GitHub Actions
Автоматическая сборка и тестирование происходит при каждом push в репозиторий.

### Способ 4: Кросс-компиляция
```bash
# Добавить target для Linux
rustup target add x86_64-unknown-linux-gnu

# Установить необходимые инструменты (требует дополнительной настройки)
cargo build --release --target x86_64-unknown-linux-gnu
```

## 📁 Структура проекта

```
PrakRust/
├── src/
│   └── main.rs          # Основной код приложения
├── music/               # Папка с музыкой
│   ├── Default/         # Плейлист по умолчанию
│   └── Test/           # Тестовый плейлист с треками
├── target/
│   ├── debug/          # Debug сборка
│   └── release/        # Release сборка
├── Cargo.toml          # Зависимости проекта
├── Dockerfile          # Для тестирования в Docker
└── test-linux.sh       # Скрипт для тестирования в Linux
```

## 🔧 Зависимости

- **eframe/egui** - GUI фреймворк
- **kira** - Аудиодвижок
- **symphonia** - Чтение метаданных аудио
- **rfd** - Диалоги выбора файлов

## 🎮 Управление

- **▶️ Play/Pause** - Воспроизведение/пауза
- **⏹️ Stop** - Остановка
- **⏮️/⏭️** - Предыдущий/следующий трек
- **⏪/⏩** - Перемотка назад/вперед (настраиваемый интервал)
- **🎚️ Слайдер** - Перемотка к определенному времени
- **🔊 Громкость** - Регулировка громкости
- **📁 Плейлисты** - Создание и управление плейлистами

## 🐛 Решение проблем

### Linux: Нет звука
```bash
# Проверьте ALSA
aplay -l
# Или попробуйте PulseAudio
pulseaudio --check
```

### Linux: Не запускается GUI
```bash
# Проверьте X11 forwarding (для SSH)
echo $DISPLAY
# Или установите дополнительные пакеты
sudo apt install libgtk-3-dev
```

### WSL: Проблемы с GUI
```bash
# Установите X11 сервер для Windows (например, VcXsrv)
# Экспортируйте DISPLAY
export DISPLAY=:0
```

## 📝 Лицензия

MIT License

## 🤝 Вклад в проект

Pull requests приветствуются! Для больших изменений сначала откройте issue.

## 📞 Поддержка

При возникновении проблем создайте issue в репозитории.