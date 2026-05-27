.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; \
		{printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: install
install: ## Install dependencies
	npm install

.PHONY: dev
dev: ## Run dev server
	npx docusaurus start

.PHONY: build
build: ## Build the production site
	npx docusaurus build

.PHONY: preview
preview: ## Serve the built site locally
	npx docusaurus serve

.PHONY: format
format: ## Format markdown and TS files
	npx prettier --write "docs/**/*.{md,mdx}" "*.ts" "src/**/*.{ts,tsx,css}"

.PHONY: check-format
check-format: ## Check formatting
	npx prettier --check "docs/**/*.{md,mdx}" "*.ts" "src/**/*.{ts,tsx,css}"

.PHONY: clean
clean: ## Remove build artefacts and the Docusaurus cache
	npx docusaurus clear
	rm -rf build node_modules
