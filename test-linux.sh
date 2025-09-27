#!/bin/bash
# Скрипт для тестирования в WSL (Windows Subsystem for Linux)

echo "🐧 Подготовка Linux окружения для тестирования..."

# Обновляем пакеты
sudo apt update

# Устанавливаем Rust (если не установлен)
if ! command -v cargo &> /dev/null; then
    echo "📦 Устанавливаем Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Устанавливаем необходимые зависимости
echo "📦 Устанавливаем системные зависимости..."
sudo apt install -y \
    pkg-config \
    libasound2-dev \
    libxcb1-dev \
    libxrandr-dev \
    libxss-dev \
    libxcursor-dev \
    libxcomposite-dev \
    libxdamage-dev \
    libxfixes-dev \
    libxinerama-dev \
    libxi-dev \
    libxrender-dev \
    libxkbfile-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libdbus-1-dev \
    libgtk-3-dev

# Собираем проект
echo "🔨 Собираем проект..."
cargo build --release

# Проверяем сборку
if [ -f "target/release/PrakRust" ]; then
    echo "✅ Сборка успешна!"
    echo "📍 Исполняемый файл: $(pwd)/target/release/PrakRust"
    
    # Показываем информацию о файле
    ls -la target/release/PrakRust
    file target/release/PrakRust
    
    echo ""
    echo "🚀 Для запуска выполните:"
    echo "   ./target/release/PrakRust"
    echo ""
    echo "⚠️  Убедитесь, что у вас настроен X11 форвардинг если запускаете через SSH"
else
    echo "❌ Ошибка сборки!"
    exit 1
fi