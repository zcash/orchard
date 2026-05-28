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

.PHONY: appendix-gates
appendix-gates: ## Regenerate docs/appendix-gates.md from the pinned circuit description
	(cd tools/gates-to-latex && cargo build --release)
	tools/gates-to-latex/target/release/gates-to-latex \
		data/orchard-0.13.1-circuit_description.txt \
		> docs/appendix-gates.md
	npx prettier --write docs/appendix-gates.md

.PHONY: clean-doc
clean-doc: ## Remove generated doc artefacts (keeps node_modules)
	npx docusaurus clear
	(cd tools/gates-to-latex && cargo clean)
	# Stray deploy output if anything was ever copied to the repo root.
	# These files are produced by CI only; the repo should never carry
	# them. See .github/workflows/onboarding-docs.yml.
	rm -rf ../404.html ../index.html ../sitemap.xml ../search-index.json \
		../.nojekyll ../assets ../img \
		../action-circuit ../appendix-gates ../audits \
		../build-test-contribute ../bundle-and-builder ../cheat-sheet \
		../crate-map ../dependencies ../discovery ../glossary \
		../halo2-primer ../keys-and-addresses ../merkle-tree \
		../note-encryption ../notes-nullifiers-commitments \
		../pasta-curves ../poseidon ../pr-checklist ../protocol-lineage \
		../redpallas ../references ../research-frontier ../search \
		../shielded-transfers ../sinsemilla ../study-plan ../test-vectors \
		../value-commitments ../worked-example

.PHONY: clean
clean: clean-doc ## Remove all build artefacts plus node_modules
	rm -rf build node_modules
