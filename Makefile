# task-apiのローカル開発で使う既定値。実行時に `make run BIND_ADDRESS=...` で上書きできる。
BIND_ADDRESS ?= 127.0.0.1:3000
DATABASE_URL ?= tasks.db

.DEFAULT_GOAL := help

.PHONY: help setup setup-rust setup-pre-commit setup-git-hooks fetch run fmt fmt-check check clippy test lint verify pre-commit clean

help: ## 利用できるコマンドを表示する
	@awk 'BEGIN {FS = ":.*## "; printf "利用できるコマンド:\n"} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: setup-rust setup-pre-commit setup-git-hooks fetch ## 開発環境をまとめてセットアップする

setup-rust: ## rustfmtとClippyをインストールする
	rustup component add rustfmt clippy

setup-pre-commit: ## pre-commitをインストールする
	@if ! command -v pre-commit >/dev/null 2>&1; then \
		if ! command -v pipx >/dev/null 2>&1; then \
			echo "エラー: pipxが必要です。先にpipxをインストールしてください。"; \
			exit 1; \
		fi; \
		pipx install pre-commit; \
	fi

setup-git-hooks: ## .githooks内のGit hookを有効化する
	chmod +x .githooks/pre-commit
	git config core.hooksPath .githooks

fetch: ## Cargoの依存crateを事前に取得する
	cargo fetch

run: ## task-apiを起動する
	BIND_ADDRESS=$(BIND_ADDRESS) DATABASE_URL=$(DATABASE_URL) cargo run -p task-api

fmt: ## Rustコードを自動整形する
	cargo fmt --all

fmt-check: ## Rustコードが整形済みか検査する
	cargo fmt --all --check

check: ## workspace全体を型検査する
	cargo check --workspace --all-targets

clippy: ## Clippyでworkspace全体を静的解析する
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test: ## workspace全体のテストを実行する
	cargo test --workspace

lint: fmt-check check clippy ## フォーマット・型・Lintをまとめて検査する

verify: lint test ## コミット前の全品質チェックを実行する

pre-commit: ## pre-commitの全hookを手動実行する
	pre-commit run --all-files

clean: ## Cargoのビルド成果物を削除する
	cargo clean
