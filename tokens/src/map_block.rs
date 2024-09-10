use crate::pb::{
    sf::solana::spl::token::v1::{
        event::Type, Approve, Burn, CloseAccount, Event, Events, FreezeAccount, InitializeAccount,
        InitializeImmutableOwner, InitializeMint, InitializeMultisig, MintTo, Revoke, SetAuthority,
        SyncNative, ThawAccount, Transfer,
    },
    sol::transactions::v1::Transactions,
};
use substreams::{pb::substreams::Clock, skip_empty_output};
use substreams_solana::pb::sf::solana::r#type::v1::ConfirmedTransaction;

#[substreams::handlers::map]
fn map_block(
    params: String,
    clock: Clock,
    trxs: Transactions,
) -> Result<Events, substreams::errors::Error> {
    skip_empty_output();
    let colon_index = params
        .find(":")
        .expect("Invalid params, must be of the form token_contract:<address>");
    let token_contract = &params[colon_index + 1..];

    let block_height = clock.number;
    let block_timestamp = clock
        .timestamp
        .as_ref()
        .map(|t| t.seconds)
        .unwrap_or_default();

    let mut data: Vec<Event> = Vec::new();
    for confirmed_txn in trxs.transactions {
        if confirmed_txn.meta().is_none() {
            continue;
        }

        let tx_id = confirmed_txn.id();
        for (i, instruction) in confirmed_txn.walk_instructions().enumerate() {
            if instruction.program_id() != spl_token::ID {
                continue;
            }

            let token_instruction =
                spl_token::instruction::TokenInstruction::unpack(instruction.data())?;

            let event = match Type::try_from((token_instruction, &instruction)) {
                Ok(event_type) => Event {
                    txn_id: tx_id.clone(),
                    block_height,
                    block_timestamp,
                    block_hash: clock.id.clone(),
                    instruction_index: i as u32,
                    r#type: Some(event_type),
                },
                Err(_) => continue,
            };

            if event
                .r#type
                .as_ref()
                .unwrap()
                .is_for_token_contract(&confirmed_txn, &token_contract)
            {
                data.push(event);
            }
        }
    }

    Ok(Events { data })
}

impl Type {
    fn is_for_token_contract(&self, trx: &ConfirmedTransaction, contract: &str) -> bool {
        match self {
            Type::Transfer(Transfer { accounts, .. }) => {
                match &accounts.as_ref().unwrap().token_mint {
                    Some(token_mint) => token_mint == contract,
                    None => trx
                        .meta()
                        .unwrap()
                        .meta
                        .as_ref()
                        .unwrap()
                        .pre_token_balances
                        .iter()
                        .any(|token_balance| {
                            token_balance.mint == contract || token_balance.owner == contract
                        }),
                }
            }
            Type::InitializeMint(InitializeMint { accounts, .. }) => {
                accounts.as_ref().unwrap().mint == contract
            }
            Type::InitializeImmutableOwner(InitializeImmutableOwner { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::InitializeAccount(InitializeAccount { accounts, .. }) => {
                accounts.as_ref().unwrap().mint == contract
            }
            Type::InitializeMultisig(InitializeMultisig { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::Approve(Approve { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::MintTo(MintTo { accounts, .. }) => accounts.as_ref().unwrap().mint == contract,
            Type::Revoke(Revoke { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::SetAuthority(SetAuthority { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::Burn(Burn { accounts, .. }) => accounts.as_ref().unwrap().mint == contract,
            Type::CloseAccount(CloseAccount { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::FreezeAccount(FreezeAccount { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::ThawAccount(ThawAccount { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
            Type::SyncNative(SyncNative { accounts: _, .. }) => {
                // FIXME: How to filter that out?
                false
            }
        }
    }
}
