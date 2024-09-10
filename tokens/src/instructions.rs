use anyhow::anyhow;
use spl_token::instruction::{AuthorityType as SplAuthorityType, TokenInstruction};
use substreams_solana::{block_view::InstructionView, Address};

use crate::pb::sf::solana::spl::token::v1::{
    approve::{ApproveAccounts, ApproveInstruction},
    burn::{BurnAccounts, BurnInstruction},
    close_account::{CloseAccountAccounts, CloseAccountInstruction},
    event::Type,
    freeze_account::{FreezeAccountAccounts, FreezeAccountInstruction},
    initialize_account::{
        InitializeAccountAccounts, InitializeAccountInstruction, InitializeAccountVersion,
    },
    initialize_immutable_owner::{
        InitializeImmutableOwnerAccounts, InitializeImmutableOwnerInstruction,
    },
    initialize_mint::{InitializeMintAccounts, InitializeMintInstruction, InitializeMintVersion},
    initialize_multisig::{
        InitializeMultisigAccounts, InitializeMultisigInstruction, InitializeMultisigVersion,
    },
    mint_to::{MintToAccounts, MintToInstruction},
    revoke::{RevokeAccounts, RevokeInstruction},
    set_authority::{AuthorityType, SetAuthorityAccounts, SetAuthorityInstruction},
    signer::Kind as SignerKind,
    sync_native::{SyncNativeAccounts, SyncNativeInstruction},
    thaw_account::{ThawAccountAccounts, ThawAccountInstruction},
    transfer::{TransferAccounts, TransferInstruction},
    Approve, Burn, CloseAccount, FreezeAccount, InitializeAccount, InitializeImmutableOwner,
    InitializeMint, InitializeMultisig, MintTo, MultiSignature, Revoke, SetAuthority, Signer,
    SingleSignature, SyncNative, ThawAccount, Transfer,
};

impl TryFrom<(TokenInstruction<'_>, &InstructionView<'_>)> for Type {
    type Error = substreams::errors::Error;

    fn try_from(value: (TokenInstruction<'_>, &InstructionView<'_>)) -> Result<Self, Self::Error> {
        let (value, instruction_view) = value;
        let accounts = instruction_view.accounts();

        Ok(match value {
            TokenInstruction::Transfer { amount } => Type::Transfer(Transfer {
                instruction: Some(TransferInstruction {
                    amount,
                    decimals: None,
                }),
                accounts: Some(TransferAccounts {
                    source: accounts.get(0).unwrap().to_string(),
                    destination: accounts.get(1).unwrap().to_string(),
                    signer: new_signer_at(&accounts, 2),
                    token_mint: None,
                }),
            }),
            TokenInstruction::InitializeMint {
                mint_authority,
                freeze_authority,
                decimals,
            } => Type::InitializeMint(InitializeMint {
                version: InitializeMintVersion::V1 as i32,
                instruction: Some(InitializeMintInstruction {
                    mint_authority: mint_authority.to_string(),
                    freeze_authority: freeze_authority
                        .map(|a| Some(a.to_string()))
                        .unwrap_or_default(),
                    decimals: decimals as u32,
                }),
                accounts: Some(InitializeMintAccounts {
                    mint: accounts.get(0).unwrap().to_string(),
                }),
            }),
            TokenInstruction::InitializeAccount => Type::InitializeAccount(InitializeAccount {
                version: InitializeAccountVersion::V1 as i32,
                instruction: Some(InitializeAccountInstruction {}),
                accounts: Some(InitializeAccountAccounts {
                    account: accounts.get(0).unwrap().to_string(),
                    mint: accounts.get(1).unwrap().to_string(),
                    owner: accounts.get(2).unwrap().to_string(),
                }),
            }),
            TokenInstruction::InitializeMultisig { m } => {
                Type::InitializeMultisig(InitializeMultisig {
                    version: InitializeMultisigVersion::V1 as i32,
                    instruction: Some(InitializeMultisigInstruction {
                        signature_count_threshold: m as u32,
                    }),
                    accounts: Some(InitializeMultisigAccounts {
                        account: accounts.get(0).unwrap().to_string(),
                        signers: accounts[1..].iter().map(|a| a.to_string()).collect(),
                    }),
                })
            }
            TokenInstruction::Approve { amount } => Type::Approve(Approve {
                instruction: Some(ApproveInstruction {
                    amount,
                    decimals: None,
                }),
                accounts: Some(ApproveAccounts {
                    source: accounts.get(0).unwrap().to_string(),
                    delegate: accounts.get(1).unwrap().to_string(),
                    signer: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                    token_mint: None,
                }),
            }),
            TokenInstruction::Revoke => Type::Revoke(Revoke {
                instruction: Some(RevokeInstruction {}),
                accounts: Some(RevokeAccounts {
                    source: accounts.get(0).unwrap().to_string(),
                    signer: new_signer(match accounts.len() {
                        2 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(1).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(1).unwrap().to_string(),
                            signers: accounts[2..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::SetAuthority {
                authority_type,
                new_authority,
            } => Type::SetAuthority(SetAuthority {
                instruction: Some(SetAuthorityInstruction {
                    authority_type: match authority_type {
                        SplAuthorityType::MintTokens => AuthorityType::AuthorityMintTokens,
                        SplAuthorityType::FreezeAccount => AuthorityType::AuthorityFreezeAccount,
                        SplAuthorityType::AccountOwner => AuthorityType::AuthorityAccountOwner,
                        SplAuthorityType::CloseAccount => AuthorityType::AuthorityCloseAccount,
                    } as i32,
                    new_authority: new_authority
                        .map(|a| Some(a.to_string()))
                        .unwrap_or_default(),
                }),
                accounts: Some(SetAuthorityAccounts {
                    account: accounts.get(0).unwrap().to_string(),
                    current_authority: accounts.get(1).unwrap().to_string(),
                }),
            }),
            TokenInstruction::MintTo { amount } => Type::MintTo(MintTo {
                instruction: Some(MintToInstruction {
                    amount,
                    decimals: None,
                }),
                accounts: Some(MintToAccounts {
                    mint: accounts.get(0).unwrap().to_string(),
                    destination: accounts.get(1).unwrap().to_string(),
                    mint_authority: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::Burn { amount } => Type::Burn(Burn {
                instruction: Some(BurnInstruction {
                    amount,
                    decimals: None,
                }),
                accounts: Some(BurnAccounts {
                    source: accounts.get(0).unwrap().to_string(),
                    mint: accounts.get(1).unwrap().to_string(),
                    signer: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::CloseAccount => Type::CloseAccount(CloseAccount {
                instruction: Some(CloseAccountInstruction {}),
                accounts: Some(CloseAccountAccounts {
                    account: accounts.get(0).unwrap().to_string(),
                    destination: accounts.get(1).unwrap().to_string(),
                    signer: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::FreezeAccount => Type::FreezeAccount(FreezeAccount {
                instruction: Some(FreezeAccountInstruction {}),
                accounts: Some(FreezeAccountAccounts {
                    account: accounts.get(0).unwrap().to_string(),
                    mint: accounts.get(1).unwrap().to_string(),
                    mint_freeze_authority: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::ThawAccount => Type::ThawAccount(ThawAccount {
                instruction: Some(ThawAccountInstruction {}),
                accounts: Some(ThawAccountAccounts {
                    account: accounts.get(0).unwrap().to_string(),
                    mint: accounts.get(1).unwrap().to_string(),
                    mint_freeze_authority: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::TransferChecked { amount, decimals } => Type::Transfer(Transfer {
                instruction: Some(TransferInstruction {
                    amount,
                    decimals: Some(decimals as u32),
                }),
                accounts: Some(TransferAccounts {
                    source: accounts.get(0).unwrap().to_string(),
                    token_mint: Some(accounts.get(1).unwrap().to_string()),
                    destination: accounts.get(2).unwrap().to_string(),
                    signer: new_signer(match accounts.len() {
                        4 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(3).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(3).unwrap().to_string(),
                            signers: accounts[4..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::ApproveChecked { amount, decimals } => Type::Approve(Approve {
                instruction: Some(ApproveInstruction {
                    amount,
                    decimals: Some(decimals as u32),
                }),
                accounts: Some(ApproveAccounts {
                    source: accounts.get(0).unwrap().to_string(),
                    token_mint: Some(accounts.get(1).unwrap().to_string()),
                    delegate: accounts.get(2).unwrap().to_string(),
                    signer: new_signer(match accounts.len() {
                        4 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(3).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(3).unwrap().to_string(),
                            signers: accounts[4..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::MintToChecked { amount, decimals } => Type::MintTo(MintTo {
                instruction: Some(MintToInstruction {
                    amount,
                    decimals: Some(decimals as u32),
                }),
                accounts: Some(MintToAccounts {
                    mint: accounts.get(0).unwrap().to_string(),
                    destination: accounts.get(1).unwrap().to_string(),
                    mint_authority: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::BurnChecked { amount, decimals } => Type::Burn(Burn {
                instruction: Some(BurnInstruction {
                    amount,
                    decimals: Some(decimals as u32),
                }),
                accounts: Some(BurnAccounts {
                    source: accounts.get(0).unwrap().to_string(),
                    mint: accounts.get(1).unwrap().to_string(),
                    signer: new_signer(match accounts.len() {
                        3 => SignerKind::Single(SingleSignature {
                            signer: accounts.get(2).unwrap().to_string(),
                        }),
                        _ => SignerKind::Multisig(MultiSignature {
                            multisig_account: accounts.get(2).unwrap().to_string(),
                            signers: accounts[3..].iter().map(Address::to_string).collect(),
                        }),
                    }),
                }),
            }),
            TokenInstruction::InitializeAccount2 { owner } => {
                Type::InitializeAccount(InitializeAccount {
                    version: InitializeAccountVersion::V2 as i32,
                    instruction: Some(InitializeAccountInstruction {}),
                    accounts: Some(InitializeAccountAccounts {
                        account: accounts.get(0).unwrap().to_string(),
                        mint: accounts.get(1).unwrap().to_string(),
                        owner: owner.to_string(),
                    }),
                })
            }
            TokenInstruction::SyncNative => Type::SyncNative(SyncNative {
                instruction: Some(SyncNativeInstruction {}),
                accounts: Some(SyncNativeAccounts {
                    native_token_account: accounts.get(0).unwrap().to_string(),
                }),
            }),
            TokenInstruction::InitializeAccount3 { owner } => {
                Type::InitializeAccount(InitializeAccount {
                    version: InitializeAccountVersion::V3 as i32,
                    instruction: Some(InitializeAccountInstruction {}),
                    accounts: Some(InitializeAccountAccounts {
                        account: accounts.get(0).unwrap().to_string(),
                        mint: accounts.get(1).unwrap().to_string(),
                        owner: owner.to_string(),
                    }),
                })
            }
            TokenInstruction::InitializeMultisig2 { m } => {
                Type::InitializeMultisig(InitializeMultisig {
                    version: InitializeMultisigVersion::V2 as i32,
                    instruction: Some(InitializeMultisigInstruction {
                        signature_count_threshold: m as u32,
                    }),
                    accounts: Some(InitializeMultisigAccounts {
                        account: accounts.get(0).unwrap().to_string(),
                        signers: accounts[1..].iter().map(|a| a.to_string()).collect(),
                    }),
                })
            }
            TokenInstruction::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => Type::InitializeMint(InitializeMint {
                version: InitializeMintVersion::V1 as i32,
                instruction: Some(InitializeMintInstruction {
                    mint_authority: mint_authority.to_string(),
                    freeze_authority: freeze_authority
                        .map(|a| Some(a.to_string()))
                        .unwrap_or_default(),
                    decimals: decimals as u32,
                }),
                accounts: Some(InitializeMintAccounts {
                    mint: accounts.get(0).unwrap().to_string(),
                }),
            }),
            TokenInstruction::InitializeImmutableOwner => {
                Type::InitializeImmutableOwner(InitializeImmutableOwner {
                    instruction: Some(InitializeImmutableOwnerInstruction {}),
                    accounts: Some(InitializeImmutableOwnerAccounts {
                        account: accounts.get(0).unwrap().to_string(),
                    }),
                })
            }
            TokenInstruction::GetAccountDataSize => {
                return Err(anyhow!("GetAccountDataSize is not supported").into())
            }
            TokenInstruction::AmountToUiAmount { .. } => {
                return Err(anyhow!("GetAccountDataSize is not supported").into())
            }
            TokenInstruction::UiAmountToAmount { .. } => {
                return Err(anyhow!("GetAccountDataSize is not supported").into())
            }
        })
    }
}

fn single_signer(signer: &Address) -> SignerKind {
    SignerKind::Single(SingleSignature {
        signer: signer.to_string(),
    })
}

fn multi_signers<'a>(multisig: &Address, signers: &'a [Address]) -> SignerKind {
    SignerKind::Multisig(MultiSignature {
        multisig_account: multisig.to_string(),
        signers: signers.iter().map(|a| a.to_string()).collect(),
    })
}

fn new_signer_at(accounts: &Vec<Address>, at: usize) -> Option<Signer> {
    Some(Signer {
        kind: Some(if accounts.len() == at + 1 {
            single_signer(accounts.get(at).expect("single signer at index {}"))
        } else {
            multi_signers(
                accounts.get(at).expect("multi signer at index {}"),
                &accounts[at + 1..],
            )
        }),
    })
}

fn new_signer(kind: SignerKind) -> Option<Signer> {
    Some(Signer { kind: Some(kind) })
}

#[cfg(test)]
mod tests {
    use substreams_solana::Address;

    #[test]
    fn test_new_signer_at() {
        let addresses: Vec<_> = vec!["B".to_string(), "C".to_string(), "D".to_string()]
            .iter()
            .map(|data| bs58::decode(data).into_vec().unwrap())
            .collect();

        let accounts = addresses.iter().map(|a| Address(a)).collect();
        let signer = super::new_signer_at(&accounts, 2);
        assert_eq!(
            signer,
            Some(super::Signer {
                kind: Some(super::SignerKind::Single(super::SingleSignature {
                    signer: "D".to_string()
                }))
            })
        );

        let signer = super::new_signer_at(&accounts, 1);
        assert_eq!(
            signer,
            Some(super::Signer {
                kind: Some(super::SignerKind::Multisig(super::MultiSignature {
                    multisig_account: "C".to_string(),
                    signers: vec!["D".to_string()]
                }))
            })
        );
    }
}
