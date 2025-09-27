# Dockerfile для тестирования на Linux
FROM rust:1.75-slim-bookworm

# Устанавливаем необходимые системные зависимости
RUN apt-get update && apt-get install -y \
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
    libgtk-3-dev \
    xvfb \
    && rm -rf /var/lib/apt/lists/*

# Создаем рабочую директорию
WORKDIR /app

# Копируем файлы проекта
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY music/ ./music/

# Собираем проект
RUN cargo build --release

# Команда для запуска (для тестирования без GUI используем xvfb)
CMD ["xvfb-run", "-a", "./target/release/PrakRust"]