FROM archlinux:base-devel

# Atualiza o sistema
RUN pacman -Syu --noconfirm

# Instala Rust, cargo, CMake antigo e dependências
RUN pacman -S --noconfirm rust git alsa-lib libpulse wget gcc make clang pkgconf pulseaudio alsa-utils

# Compila e instala CMake 3.20
WORKDIR /opt
RUN wget https://cmake.org/files/v3.20/cmake-3.20.5.tar.gz && \
    tar -xvzf cmake-3.20.5.tar.gz && \
    cd cmake-3.20.5 && \
    ./bootstrap && make -j$(nproc) && make install

# Cria diretório do projeto
WORKDIR /app

# Copia o código
COPY . .

# Configura variáveis de ambiente para áudio no WSLg
ENV PULSE_SERVER=unix:/mnt/wslg/PulseServer

# Compila o projeto
RUN cargo build --release

# Comando padrão ao rodar o container
CMD ["./target/release/web-radio"]
