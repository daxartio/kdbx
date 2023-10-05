DEFAULT_GOAL := all

.PHONY: all
all: fmt check test

.PHONY: check
check:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

.PHONY: test
test:
	cargo test

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: cli-md
cli-md:
	@rm -f cli.md
	@echo "### commands\n\n\`\`\`" >> cli.md && cargo run -q -- -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### pwd\n\n\`\`\`" >> cli.md && cargo run -q -- pwd -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### totp\n\n\`\`\`" >> cli.md && cargo run -q -- totp -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### show\n\n\`\`\`" >> cli.md && cargo run -q -- show -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### add\n\n\`\`\`" >> cli.md && cargo run -q -- add -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### init\n\n\`\`\`" >> cli.md && cargo run -q -- init -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### list\n\n\`\`\`" >> cli.md && cargo run -q -- list -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@cat cli.md
	@rm cli.md
