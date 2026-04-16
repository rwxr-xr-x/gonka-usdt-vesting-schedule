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

## Dependencies

- CosmWasm 3.0.x
- cw-storage-plus 3.0.x
- cw2 3.0.x

## License

This project is licensed under the [MIT License](LICENSE).
