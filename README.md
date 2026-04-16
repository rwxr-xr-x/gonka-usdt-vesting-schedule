# gonka-usdt-vesting-schedule

CosmWasm smart contract for time-locked USDT vesting on Gonka chain.

## Overview

This contract holds IBC USDT tokens (`ibc/115F68FBA220A028C6F6ED08EA0C1A9C8C52798B14FB66E6C89D5D8C06A524D4`) and releases them to a designated beneficiary address according to a predefined schedule.

A governor address (e.g. chain governance module) holds freeze and emergency withdraw controls.

All vesting parameters (token denom, tranche amounts, schedule) are hardcoded as constants in `src/state.rs` and must be configured before compilation.

### Vesting Schedule

| Tranche | Amount | Base Units | Unlock Time |
|---------|--------|------------|-------------|
| 0 | 51,000 USDT | 51,000,000,000 | Immediate (at instantiation) |
| 1 | 15,000 USDT | 15,000,000,000 | +3 months (90 days) |
| 2 | 15,000 USDT | 15,000,000,000 | +6 months (180 days) |
| 3 | 15,000 USDT | 15,000,000,000 | +9 months (270 days) |
| **Total** | **96,000 USDT** | **96,000,000,000** | |

## How It Works

1. Contract is instantiated with a **governor** and a **beneficiary** address
2. Four tranches are created automatically with hardcoded amounts and maturity offsets
3. Contract must be funded separately after instantiation
4. After each tranche's maturity time passes, **anyone** can call `ReleaseTranche` to send tokens to the beneficiary
5. The governor can freeze/unfreeze the contract, change the beneficiary, or withdraw all remaining funds

## Messages

### Execute

| Message | Access | Description |
|---------|--------|-------------|
| `ReleaseTranche { tranche_id }` | Anyone | Send tokens for a matured tranche to the beneficiary |
| `Freeze {}` | Governor | Freeze all releases |
| `Unfreeze {}` | Governor | Unfreeze releases |
| `SetBeneficiary { address }` | Governor | Change beneficiary address |
| `WithdrawAll { to }` | Governor | Withdraw all remaining tokens to a specified address |

### Query

| Message | Description |
|---------|-------------|
| `Config {}` | Contract configuration (governor, beneficiary, frozen state, created_at) |
| `Tranche { id }` | Single tranche details |
| `AllTranches {}` | All four tranches |
| `Balance {}` | Contract's current token balance |

## Build

### Prerequisites

- Rust with `wasm32-unknown-unknown` target
- Docker (for optimized production build)

### Check Compilation

```bash
make check
```

### Run Tests

```bash
make test
```

### Dev Build

```bash
make build-dev
```

### Production Build

```bash
make build
```

Output: `artifacts/gonka_usdt_vesting_schedule.wasm`

# Deploy

### Store WASM code on-chain:

```bash
./inferenced tx wasm store ./artifacts/gonka_usdt_vesting_schedule.wasm \
  --from gonka-account-name \
  --gas auto \
  --gas-adjustment 1.5 \
  --fees 200000ngonka \
  --node http://node1.gonka.ai:8000/chain-rpc/ \
  --chain-id gonka-mainnet \
  --keyring-backend file \
  --home ./.inference
```

Get TXID from command output, e.g. A072D3F440CD0847E5C6A19C32C95660DBC985BC1BD6E4D631B7AE1A5B2863B6. Will be used in next step.

### Get contract code_id

```bash
./inferenced query tx A072D3F440CD0847E5C6A19C32C95660DBC985BC1BD6E4D631B7AE1A5B2863B6 \
  --node http://node1.gonka.ai:8000/chain-rpc/ \
  --chain-id gonka-mainnet --output json | jq -r '.events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value'
```

Get code_id from command output, e.g. 99. Will be used in next step.

## Instantiate contract with exact governor and beneficiary addresses

```bash
./inferenced tx wasm instantiate 99 '{"governor":"gonka10d07y265gmmuvt4z0w9aw880jnsr700j2h5m33","beneficiary":"gonka12ss9dh7fj3xxmk23s8aje4hrpqq669u20v3ja6"}' \
  --label "gonka_usdt_vesting_schedule" \
  --admin gonka10d07y265gmmuvt4z0w9aw880jnsr700j2h5m33 \
  --from gonka-account-name \
  --gas auto \
  --gas-adjustment 1.5 \
  --fees 200000ngonka \
  --node http://node1.gonka.ai:8000/chain-rpc/ \
  --chain-id gonka-mainnet \
  --keyring-backend file \
  --home ./.inference
```

Governor address **gonka10d07y265gmmuvt4z0w9aw880jnsr700j2h5m33** it is a Gonka Governance Module.

Beneficiary address is the final recipient of the tranches.

Get TXID from command output, e.g. 80543DE28CB6A569FBC99490AB2270655BBC82500B497057CB8E35718620CAF6. Will be used in next step.

## Get contract instance address

```bash
./inferenced query tx 80543DE28CB6A569FBC99490AB2270655BBC82500B497057CB8E35718620CAF6 \
  --node http://node1.gonka.ai:8000/chain-rpc/ \
  --chain-id gonka-mainnet --output json | jq -r '.events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value'
```

Get contract exact instance address, e.g. gonka1yf2f23sqx8fradjn7laqp0twamlhy4sj6vzwmg946ux4awfqaaes9avx7a. Will be used in next steps.

## Verify contract config

```bash
curl -s "https://node3.gonka.ai//chain-api/cosmwasm/wasm/v1/contract/gonka1yf2f23sqx8fradjn7laqp0twamlhy4sj6vzwmg946ux4awfqaaes9avx7a/smart/$(echo -n '{"config":{}}' | base64)" | jq '.data'
```

Output:
```json
{
  "governor": "gonka10d07y265gmmuvt4z0w9aw880jnsr700j2h5m33",
  "beneficiary": "gonka12ss9dh7fj3xxmk23s8aje4hrpqq669u20v3ja6",
  "frozen": false,
  "created_at": 1776362125
}
```

## Verify tranche amounts, timestamps and finalization

```bash
./inferenced query wasm contract-state smart gonka1yf2f23sqx8fradjn7laqp0twamlhy4sj6vzwmg946ux4awfqaaes9avx7a '{"all_tranches":{}}' \
  --node http://node1.gonka.ai:8000/chain-rpc/ \
  --chain-id gonka-mainnet --output json | jq
```

Output:
```json
{
  "data": {
    "tranches": [
      {
        "index": 0,
        "token_amount": "51000000000",
        "matures_at": 1776362125,
        "released": false
      },
      {
        "index": 1,
        "token_amount": "15000000000",
        "matures_at": 1784138125,
        "released": false
      },
      {
        "index": 2,
        "token_amount": "15000000000",
        "matures_at": 1791914125,
        "released": false
      },
      {
        "index": 3,
        "token_amount": "15000000000",
        "matures_at": 1799690125,
        "released": false
      }
    ]
  }
}
```

## Verify contract balance

```bash
./inferenced query bank balances gonka1yf2f23sqx8fradjn7laqp0twamlhy4sj6vzwmg946ux4awfqaaes9avx7a --node http://node2.gonka.ai:8000/chain-rpc/
```

or
```bash
./inferenced query wasm contract-state smart gonka1yf2f23sqx8fradjn7laqp0twamlhy4sj6vzwmg946ux4awfqaaes9avx7a '{"balance":{}}' \
  --node http://node1.gonka.ai:8000/chain-rpc/ \
  --chain-id gonka-mainnet --output json | jq
```

```json
{
  "data": {
    "balance": {
      "denom": "ibc/115F68FBA220A028C6F6ED08EA0C1A9C8C52798B14FB66E6C89D5D8C06A524D4",
      "amount": "0"
    }
  }
}
```

## Dependencies

- CosmWasm 3.0.x
- cw-storage-plus 3.0.x
- cw2 3.0.x

## License

This project is licensed under the [MIT License](LICENSE).
