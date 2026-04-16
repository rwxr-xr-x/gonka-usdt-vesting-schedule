use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Storage, Uint128, Uint256,
};
use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{
    AllTranchesResponse, BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    TrancheResponse, ConfigResponse,
};
use crate::state::{
    Config, Tranche, TRANCHE_AMOUNTS, TRANCHE_COUNT, TRANCHE_OFFSETS, TRANCHES,
    TOKEN_DENOM, CONFIG,
};

const CONTRACT_NAME: &str = "gonka-usdt-vesting-schedule";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let governor = deps.api.addr_validate(&msg.governor)?;
    let beneficiary = deps.api.addr_validate(&msg.beneficiary)?;

    let config = Config {
        governor: governor.clone(),
        beneficiary: beneficiary.clone(),
        frozen: false,
        created_at: env.block.time,
    };
    CONFIG.save(deps.storage, &config)?;

    let base_time = env.block.time;
    for i in 0..TRANCHE_COUNT {
        let entry = Tranche {
            index: i,
            token_amount: Uint128::from(TRANCHE_AMOUNTS[i as usize]),
            matures_at: base_time.plus_seconds(TRANCHE_OFFSETS[i as usize]),
            released: false,
        };
        TRANCHES.save(deps.storage, i, &entry)?;
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("governor", governor)
        .add_attribute("beneficiary", beneficiary)
        .add_attribute("tranches", TRANCHE_COUNT.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReleaseTranche { tranche_id } => release_tranche(deps, env, tranche_id),
        ExecuteMsg::Freeze {} => freeze(deps, info),
        ExecuteMsg::Unfreeze {} => unfreeze(deps, info),
        ExecuteMsg::SetBeneficiary { address } => set_beneficiary(deps, info, address),
        ExecuteMsg::WithdrawAll { to } => withdraw_all(deps, env, info, to),
    }
}

fn ensure_governor(storage: &dyn Storage, sender: &Addr) -> Result<Config, ContractError> {
    let config = CONFIG.load(storage)?;
    if *sender != config.governor {
        return Err(ContractError::NotGovernor {});
    }
    Ok(config)
}

fn release_tranche(deps: DepsMut, env: Env, tranche_id: u8) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if config.frozen {
        return Err(ContractError::ContractFrozen {});
    }

    let mut tranche = TRANCHES
        .load(deps.storage, tranche_id)
        .map_err(|_| ContractError::TrancheNotFound { id: tranche_id })?;

    if tranche.released {
        return Err(ContractError::TrancheAlreadyReleased { id: tranche_id });
    }

    if env.block.time < tranche.matures_at {
        return Err(ContractError::TrancheNotYetMature {
            id: tranche_id,
            matures_at: tranche.matures_at.seconds(),
            now: env.block.time.seconds(),
        });
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), TOKEN_DENOM)?;
    let required = Uint256::from(tranche.token_amount);

    if balance.amount < required {
        return Err(ContractError::InsufficientFunds {
            held: balance.amount.to_string(),
            required: tranche.token_amount.to_string(),
        });
    }

    tranche.released = true;
    TRANCHES.save(deps.storage, tranche_id, &tranche)?;

    let send = BankMsg::Send {
        to_address: config.beneficiary.to_string(),
        amount: vec![Coin {
            denom: TOKEN_DENOM.to_string(),
            amount: tranche.token_amount.into(),
        }],
    };

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "release_tranche")
        .add_attribute("tranche_id", tranche_id.to_string())
        .add_attribute("token_amount", tranche.token_amount.to_string())
        .add_attribute("beneficiary", config.beneficiary.to_string()))
}

fn freeze(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = ensure_governor(deps.storage, &info.sender)?;
    config.frozen = true;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "freeze"))
}

fn unfreeze(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = ensure_governor(deps.storage, &info.sender)?;
    config.frozen = false;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "unfreeze"))
}

fn set_beneficiary(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut config = ensure_governor(deps.storage, &info.sender)?;
    let new_beneficiary = deps.api.addr_validate(&address)?;
    config.beneficiary = new_beneficiary.clone();
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "set_beneficiary")
        .add_attribute("beneficiary", new_beneficiary.to_string()))
}

fn withdraw_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to: String,
) -> Result<Response, ContractError> {
    ensure_governor(deps.storage, &info.sender)?;
    let recipient = deps.api.addr_validate(&to)?;

    let balance = deps
        .querier
        .query_balance(env.contract.address.to_string(), TOKEN_DENOM)?;

    if balance.amount.is_zero() {
        return Ok(Response::new()
            .add_attribute("action", "withdraw_all")
            .add_attribute("status", "nothing_to_withdraw"));
    }

    let amount_str = balance.amount.to_string();
    let send = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![balance],
    };

    Ok(Response::new()
        .add_message(send)
        .add_attribute("action", "withdraw_all")
        .add_attribute("amount", amount_str)
        .add_attribute("recipient", recipient.to_string()))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_vesting_info(deps)?),
        QueryMsg::Tranche { id } => to_json_binary(&query_tranche(deps, id)?),
        QueryMsg::AllTranches {} => to_json_binary(&query_all_tranches(deps)?),
        QueryMsg::Balance {} => to_json_binary(&query_balance(deps, env)?),
    }
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Binary) -> Result<Response, ContractError> {
    let prev = get_contract_version(deps.storage)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("from_version", prev.version)
        .add_attribute("to_version", CONTRACT_VERSION))
}

fn query_vesting_info(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        governor: config.governor.to_string(),
        beneficiary: config.beneficiary.to_string(),
        frozen: config.frozen,
        created_at: config.created_at.seconds(),
    })
}

fn query_tranche(deps: Deps, id: u8) -> StdResult<TrancheResponse> {
    let s = TRANCHES
        .load(deps.storage, id)
        .map_err(|_| cosmwasm_std::StdError::msg(format!("tranche {id} does not exist")))?;
    Ok(TrancheResponse {
        index: s.index,
        token_amount: s.token_amount,
        matures_at: s.matures_at.seconds(),
        released: s.released,
    })
}

fn query_all_tranches(deps: Deps) -> StdResult<AllTranchesResponse> {
    let mut tranches = Vec::with_capacity(TRANCHE_COUNT as usize);
    for i in 0..TRANCHE_COUNT {
        let s = TRANCHES.load(deps.storage, i)?;
        tranches.push(TrancheResponse {
            index: s.index,
            token_amount: s.token_amount,
            matures_at: s.matures_at.seconds(),
            released: s.released,
        });
    }
    Ok(AllTranchesResponse { tranches })
}

fn query_balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
    let balance = deps
        .querier
        .query_balance(&env.contract.address, TOKEN_DENOM)?;
    Ok(BalanceResponse { balance })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi};
    use cosmwasm_std::{from_json, Addr, MessageInfo, Uint128};

    fn test_setup_msg(api: &MockApi) -> InstantiateMsg {
        InstantiateMsg {
            governor: api.addr_make("governor").to_string(),
            beneficiary: api.addr_make("beneficiary").to_string(),
        }
    }

    macro_rules! fund_contract {
        ($deps:expr, $env:expr, $amount:expr) => {
            $deps.querier.bank.update_balance(
                $env.contract.address.to_string(),
                vec![Coin {
                    denom: TOKEN_DENOM.to_string(),
                    amount: Uint256::from($amount as u128),
                }],
            );
        };
    }

    #[test]
    fn test_instantiate_creates_tranches() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let env = mock_env();
        let beneficiary = api.addr_make("beneficiary").to_string();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };

        let res = instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();
        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "beneficiary" && a.value == beneficiary));
        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "tranches" && a.value == "4"));

        let all: AllTranchesResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::AllTranches {}).unwrap()).unwrap();

        assert_eq!(all.tranches.len(), 4);
        assert_eq!(all.tranches[0].token_amount, Uint128::from(51_000_000_000u128));
        assert_eq!(all.tranches[1].token_amount, Uint128::from(15_000_000_000u128));
        assert_eq!(all.tranches[2].token_amount, Uint128::from(15_000_000_000u128));
        assert_eq!(all.tranches[3].token_amount, Uint128::from(15_000_000_000u128));
        assert!(!all.tranches[0].released);
    }

    #[test]
    fn test_release_immediate() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        fund_contract!(deps, &env,96_000_000_000);

        let info = MessageInfo {
            sender: Addr::unchecked("anyone"),
            funds: vec![],
        };
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::ReleaseTranche { tranche_id: 0 },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "token_amount" && a.value == "51000000000"));

        let s: TrancheResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Tranche { id: 0 }).unwrap()).unwrap();
        assert!(s.released);
    }

    #[test]
    fn test_release_after_maturity() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        fund_contract!(deps, &env,96_000_000_000);

        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(91 * 24 * 60 * 60);

        let info = MessageInfo {
            sender: Addr::unchecked("anyone"),
            funds: vec![],
        };
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseTranche { tranche_id: 1 },
        )
        .unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "token_amount" && a.value == "15000000000"));
    }

    #[test]
    fn test_release_before_maturity_fails() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        fund_contract!(deps, &env,96_000_000_000);

        let info = MessageInfo {
            sender: Addr::unchecked("anyone"),
            funds: vec![],
        };
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseTranche { tranche_id: 1 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::TrancheNotYetMature { id: 1, .. }));
    }

    #[test]
    fn test_double_release_fails() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        fund_contract!(deps, &env,96_000_000_000);

        let info = MessageInfo {
            sender: Addr::unchecked("anyone"),
            funds: vec![],
        };

        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::ReleaseTranche { tranche_id: 0 },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseTranche { tranche_id: 0 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::TrancheAlreadyReleased { id: 0 }));
    }

    #[test]
    fn test_release_while_frozen_fails() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let governor_addr = api.addr_make("governor");
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        fund_contract!(deps, &env,96_000_000_000);

        let info = MessageInfo {
            sender: governor_addr,
            funds: vec![],
        };
        execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Freeze {}).unwrap();

        let info = MessageInfo {
            sender: Addr::unchecked("anyone"),
            funds: vec![],
        };
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseTranche { tranche_id: 0 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::ContractFrozen {}));
    }

    #[test]
    fn test_freeze_unfreeze_cycle() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let governor_addr = api.addr_make("governor");
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        let info = MessageInfo {
            sender: governor_addr.clone(),
            funds: vec![],
        };
        execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::Freeze {}).unwrap();

        let cfg: ConfigResponse =
            from_json(query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap())
                .unwrap();
        assert!(cfg.frozen);

        execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Unfreeze {}).unwrap();

        let cfg: ConfigResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
        assert!(!cfg.frozen);
    }

    #[test]
    fn test_set_beneficiary() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let governor_addr = api.addr_make("governor");
        let new_beneficiary = api.addr_make("new_beneficiary").to_string();
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        let info = MessageInfo {
            sender: governor_addr,
            funds: vec![],
        };
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::SetBeneficiary {
                address: new_beneficiary.clone(),
            },
        )
        .unwrap();

        let cfg: ConfigResponse =
            from_json(query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(cfg.beneficiary, new_beneficiary);
    }

    #[test]
    fn test_non_governor_rejected() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let attacker = api.addr_make("attacker");
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        let info = MessageInfo {
            sender: attacker.clone(),
            funds: vec![],
        };

        let err = execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::Freeze {}).unwrap_err();
        assert!(matches!(err, ContractError::NotGovernor {}));

        let err = execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::Unfreeze {}).unwrap_err();
        assert!(matches!(err, ContractError::NotGovernor {}));

        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::SetBeneficiary {
                address: attacker.to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotGovernor {}));

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::WithdrawAll {
                to: attacker.to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotGovernor {}));
    }

    #[test]
    fn test_withdraw_all() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let governor_addr = api.addr_make("governor");
        let someone = api.addr_make("someone");
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        fund_contract!(deps, &env,96_000_000_000);

        let info = MessageInfo {
            sender: governor_addr,
            funds: vec![],
        };
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::WithdrawAll {
                to: someone.to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.messages.len(), 1);
        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "action" && a.value == "withdraw_all"));
    }

    #[test]
    fn test_withdraw_empty_is_noop() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let governor_addr = api.addr_make("governor");
        let someone = api.addr_make("someone");
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        let info = MessageInfo {
            sender: governor_addr,
            funds: vec![],
        };
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::WithdrawAll {
                to: someone.to_string(),
            },
        )
        .unwrap();

        assert!(res.messages.is_empty());
        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "status" && a.value == "nothing_to_withdraw"));
    }

    #[test]
    fn test_release_insufficient_funds() {
        let mut deps = mock_dependencies();
        let api = MockApi::default();
        let env = mock_env();
        let info = MessageInfo {
            sender: Addr::unchecked("creator"),
            funds: vec![],
        };
        instantiate(deps.as_mut(), env.clone(), info, test_setup_msg(&api)).unwrap();

        // Fund with less than tranche 0 requires (51_000_000_000)
        fund_contract!(deps, &env,1_000_000);

        let info = MessageInfo {
            sender: Addr::unchecked("anyone"),
            funds: vec![],
        };
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseTranche { tranche_id: 0 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::InsufficientFunds { .. }));
    }
}
