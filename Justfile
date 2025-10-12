all: fmt check test

check:
	cargo +nightly fmt --all -- --check
	cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo test

fmt:
	cargo +nightly fmt --all

cli-md:
	@rm -f cli.md
	@echo "### commands\n\n\`\`\`" >> cli.md && cargo run -q -- -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### pwd\n\n\`\`\`" >> cli.md && cargo run -q -- pwd -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### totp\n\n\`\`\`" >> cli.md && cargo run -q -- totp -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### show\n\n\`\`\`" >> cli.md && cargo run -q -- show -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### add\n\n\`\`\`" >> cli.md && cargo run -q -- add -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### init\n\n\`\`\`" >> cli.md && cargo run -q -- init -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### list\n\n\`\`\`" >> cli.md && cargo run -q -- list -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@echo "### completion\n\n\`\`\`" >> cli.md && cargo run -q -- completion -h >> cli.md && echo "\`\`\`\n" >> cli.md
	@cat cli.md
	@rm cli.md
