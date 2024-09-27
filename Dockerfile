# Okay, what's going on here?

# Okay, what do we have here?

# 1. Download this layer. It consists of some distribution(no GUI) and rust toolchain

# this image has also cargo-chef installed#

# цей рівень закешувати просто
FROM lukemathwalker/cargo-chef:latest-rust-1.81.0 AS chef

# the root directoy is called app? It seems to be a single directory inside `/` directory
WORKDIR /app


# цю операцію, мабуть, також 
# сама операційна система кешує таке
# While building a container you will run the command below
RUN apt update && apt install lld clang -y

# Все, що відбулося до цього моменту - оновилися утиліти дистрибутива, а також поставився lld компонувальник


# Попередній image тепер буде використовуватися як фундамент для наступного


FROM chef AS planner

# це також якось можна закешувати, думаю
# З поточної директорії, де є Dockerfile, всі залежності(крім тих, що в .dockerignore) скопіюються в app?
COPY . . 
# Compute a lock-like file for our project
# приготували список всіх залежностей проекту, які є в поточній директорії

# знову-таки, сама операція має мати можливість кешувати свій результат
RUN cargo chef prepare --recipe-path recipe.json

# Ще раз використавли chef image як фундамент вже тут 
FROM chef AS builder

# Скопіювали з поперднього імеджа цей файл
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our app

# Скомпілювали залежності 
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.

# Скопіювали файли з поточної директорії вже в той image/container
COPY . .
ENV SQLX_OFFLINE=true
# Build our project

# лише тепер ми перекомпільовуємо наш проект, якщо в ньому були якісь зміни



RUN cargo build --release --bin zero2prod


FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# з білдера дістали скомпільований файл(в самому контейнері нам треба лише його)
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./zero2prod"] 


# Чи можна тут обійтися без цих різних image'ів, а зробити все одним?


# ДОбре, як саме тут відбудеться кешування?
# Він ніби казав, що після кожної операції COPY і RUN щось може закешуватися в нього там.