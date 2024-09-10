use crate::pb::sf::solana::spl::token::v1::{event::Type, signer, Events, Signer, Transfer};
use substreams::skip_empty_output;
use substreams_database_change::{
    pb::database::DatabaseChanges,
    tables::{Row, Tables},
};

#[substreams::handlers::map]
fn db_out(events: Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    skip_empty_output();

    let mut tables = Tables::new();

    for event in events.data {
        let event_type = event.r#type.as_ref().unwrap();

        let row = tables
            .create_row(
                event_type.to_table(),
                [
                    ("evt_tx", event.txn_id.clone()),
                    ("evt_instruction_index", event.instruction_index.to_string()),
                ],
            )
            .set("evt_block_timestamp", event.block_timestamp)
            .set("evt_block_height", event.block_height)
            .set("evt_block_hash", &event.block_hash);

        match event_type {
            Type::Transfer(Transfer {
                instruction,
                accounts,
            }) => {
                let instruction = instruction.as_ref().unwrap();
                let accounts = accounts.as_ref().unwrap();

                row.set("amount", instruction.amount)
                    .set("source", &accounts.source)
                    .set("destination", &accounts.destination)
                    .set_signer("signers", accounts.signer.as_ref().unwrap());
            }
            _ => continue,
        }
    }

    Ok(tables.to_database_changes())
}

trait SetSigners {
    fn set_signer(&mut self, name: &str, signer: &Signer) -> &mut Row;
}

impl SetSigners for Row {
    fn set_signer(&mut self, name: &str, signer: &Signer) -> &mut Row {
        match signer.kind.as_ref().unwrap() {
            signer::Kind::Single(single) => {
                self.set(name, &single.signer);
            }
            signer::Kind::Multisig(multi) => {
                self.set(name, multi.signers.join(","));
            }
        }

        self
    }
}

impl Type {
    fn to_table(&self) -> &'static str {
        match self {
            Type::Transfer(_) => "transfer",
            Type::InitializeMint(_) => "initialize_mint",
            Type::InitializeImmutableOwner(_) => "initialize_immutable_owner",
            Type::InitializeAccount(_) => "initialize_account",
            Type::InitializeMultisig(_) => "initialize_multisig",
            Type::Approve(_) => "approve",
            Type::MintTo(_) => "mint_to",
            Type::Revoke(_) => "revoke",
            Type::SetAuthority(_) => "set_authority",
            Type::Burn(_) => "burn",
            Type::CloseAccount(_) => "close_account",
            Type::FreezeAccount(_) => "freeze_account",
            Type::ThawAccount(_) => "thaw_account",
            Type::SyncNative(_) => "sync_native",
        }
    }
}
