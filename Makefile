POOL_PARTY_RAYDIUM_KEY := $(shell anchor keys list | grep "pool_party_raydium:" | awk '{print $$2}')


build:
	anchor build

build-devnet:
	 anchor build -- --features devnet

clean-build:
	anchor clean && anchor build

deploy:
	anchor deploy

deploy-local:
	anchor deploy --provider.cluster localnet

program-show:
	solana program show $(POOL_PARTY_RAYDIUM_KEY)

set-config-local:
	solana config set --url localhost 

set-config-devnet:
	solana config set --url devnet

logs:
	solana logs

airdrop:
	 solana airdrop 1000 3Hce9T6umcyH3pGg916o73qvMjjWv8YMExTyrSLqvtW2

test:
	anchor test

test-skip-local-validator:
	anchor test --skip-build --skip-local-validator

test-skip-local-validator-build:
	anchor test --skip-local-validator

sync-keys:
	anchor keys sync

my-wallet:
	solana address
 
start-test-validator:
	solana-test-validator -r

dump-devnet:
	solana program dump -u d devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH clmm_devnet.so
	solana program dump -u d metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata_program_devnet.so 
	solana account -u d --output json-compact --output-file amm_config_devnet.json CQYbhr6amxUER4p5SC44C63R4qw4NFc9Z4Db9vF4tZwG
	# solana account -u http://127.0.0.1:8899 --output json-compact --output-file mint_0_devnet.json 6YMTJbG8p7y3sYDuPJWHqghKFYkNa3aR5iCZqtmv1dU5
	# solana account -u http://127.0.0.1:8899 --output json-compact --output-file mint_1_devnet.json Dc9PLgqViL3k8erk5CTW42zZCUnM6sijN4KGWJnadUrL

start-test-validator-from-dump-devnet:
	# make dump-devnet
	solana-test-validator \
	--account CQYbhr6amxUER4p5SC44C63R4qw4NFc9Z4Db9vF4tZwG amm_config_devnet.json \
	--bpf-program devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH clmm_devnet.so \
	--bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata_program_devnet.so \
	--reset

dump-mainnet:
	solana program dump -u m CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK clmm_mainnet.so
	solana program dump -u m metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata_program_mainnet.so 
	solana account -u m --output json-compact --output-file amm_config_mainnet.json 4BLNHtVe942GSs4teSZqGX24xwKNkqU7bGgNn3iUiUpw
	solana account -u m --output json-compact --output-file amm_config_0_mainnet.json 9iFER3bpjf1PTTCQCfTRu17EJgvsxo9pVyA9QWwEuX4x

start-test-validator-from-dump-mainnet:
	# make dump-mainnet
	solana-test-validator \
	--account 9iFER3bpjf1PTTCQCfTRu17EJgvsxo9pVyA9QWwEuX4x amm_config_0_mainnet.json \
	--bpf-program CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK clmm_mainnet.so \
	--bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata_program_mainnet.so \
	--reset

# solana account -u d --output json-compact --output-file jup.json devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH

# solana-test-validator --account devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH jup.json --reset

# solana-test-validator --bpf-program CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK spl_governance.so --reset
