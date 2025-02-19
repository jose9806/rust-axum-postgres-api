dev:
	docker-compose up -d

dev-down:
	docker-compose down

migrate-up:
	sqlx migrate run

migrate-down:
	sqlx migrate revert

start-server:
	cargo watch -q -c -w src/ -x run

install:
	cargo add axum@0.8 --features multipart
	cargo add utoipa --features "chrono uuid"
	cargo add utoipa-swagger-ui --features "axum"
	cargo add tokio -F full
	cargo add tower-http -F "cors"
	cargo add serde_json
	cargo add serde -F derive
	cargo add chrono -F serde
	cargo add dotenv
	cargo add validator --features derive
	cargo add uuid -F "serde v4"
	cargo add sqlx -F "runtime-async-std-native-tls postgres chrono uuid"
	cargo add reqwest --features "json multipart"
	# HotReload
	cargo install cargo-watch
	# SQLX-CLI
	cargo install sqlx-cli


test:
	cargo test
