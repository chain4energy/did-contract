check-contract:
	cosmwasm-check ./target/wasm32-unknown-unknown/release/did_contract.wasm

build:
	cargo wasm

clean:
	cargo clean

optimize:
	@echo "!!!!!!! NOTE: for production use only intel porcessor, so no Mac M1 - see https://github.com/CosmWasm/optimizer"
# CosmWasm Rust Optimizer
	docker run  --rm -v .:/code \
	--mount type=volume,source="empty-contract_cache",target=/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/optimizer:0.16.0


# --user $$(id -u):$$(id -g)
# docker volume rm empty-contract_cache
# docker volume rm registry_cache
