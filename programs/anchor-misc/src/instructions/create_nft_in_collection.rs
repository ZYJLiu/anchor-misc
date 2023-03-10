// create new nft in collection
use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    instruction::{
        create_master_edition_v3, create_metadata_accounts_v3, set_and_verify_collection,
        sign_metadata,
    },
    pda::{find_master_edition_account, find_metadata_account},
    state::Creator,
    ID as MetadataID,
};

#[derive(Accounts)]
pub struct CreateNftInCollection<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = auth,
        mint::freeze_authority = auth,
    )]
    pub mint: Box<Account<'info, Mint>>,
    /// CHECK: metadata account
    #[account(
        mut,
        address=find_metadata_account(&mint.key()).0
    )]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: master edition account
    #[account(
        mut,
        address=find_master_edition_account(&mint.key()).0
    )]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub collection_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        address=find_metadata_account(&collection_mint.key()).0
    )]
    /// CHECK:
    pub collection_metadata: UncheckedAccount<'info>,
    #[account(
         mut,
        address=find_master_edition_account(&collection_mint.key()).0
    )]
    /// CHECK:
    pub collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: mint authority
    #[account(
        mut,
        seeds = ["auth".as_bytes().as_ref()],
        bump,
    )]
    pub auth: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: user receiving mint
    pub user: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, TokenMetaData>,
}
#[derive(Clone)]
pub struct TokenMetaData;
impl anchor_lang::Id for TokenMetaData {
    fn id() -> Pubkey {
        MetadataID
    }
}

pub fn create_nft_in_collection_handler(
    ctx: Context<CreateNftInCollection>,
    uri: String,
    name: String,
    symbol: String,
) -> Result<()> {
    let seeds = &["auth".as_bytes(), &[*ctx.bumps.get("auth").unwrap()]];
    let signer = [&seeds[..]];

    // mint token
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                authority: ctx.accounts.auth.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
            &signer,
        ),
        1, // only 1 token minted
    )?;

    let account_info = vec![
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.auth.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
    ];

    let creator = vec![Creator {
        address: ctx.accounts.auth.key(),
        verified: false,
        share: 100,
    }];

    // create metadata account
    invoke_signed(
        &create_metadata_accounts_v3(
            ctx.accounts.token_metadata_program.key(), // token metadata program
            ctx.accounts.metadata.key(),               // metadata account PDA for mint
            ctx.accounts.mint.key(),                   // mint account
            ctx.accounts.auth.key(),                   // mint authority
            ctx.accounts.payer.key(),                  // payer for transaction
            ctx.accounts.auth.key(),                   // update authority
            name,                                      // name
            symbol,                                    // symbol
            uri,                                       // nft uri (offchain metadata)
            Some(creator),                             // (optional) creators
            0,                                         // seller free basis points
            true,                                      // (bool) update authority is signer
            true,                                      // (bool)is mutable
            None,                                      // (optional) collection
            None,                                      // (optional) uses
            None,                                      // (optional) collection details
        ),
        account_info.as_slice(),
        &signer,
    )?;

    let master_edition_infos = vec![
        ctx.accounts.master_edition.to_account_info(),
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.auth.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
    ];

    // create master edition account
    invoke_signed(
        &create_master_edition_v3(
            ctx.accounts.token_metadata_program.key(), // token metadata program
            ctx.accounts.master_edition.key(),         // master edition account PDA
            ctx.accounts.mint.key(),                   // mint account
            ctx.accounts.auth.key(),                   // update authority
            ctx.accounts.auth.key(),                   // mint authority
            ctx.accounts.metadata.key(),               // metadata account
            ctx.accounts.payer.key(),                  // payer
            Some(0),                                   // (optional) max supply
        ),
        master_edition_infos.as_slice(),
        &signer,
    )?;

    let sign_metadata_info = vec![
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.auth.to_account_info(),
    ];

    // sign to verify creator
    invoke_signed(
        &sign_metadata(
            ctx.accounts.token_metadata_program.key(), // token metadata program
            ctx.accounts.metadata.key(),               // metadata account
            ctx.accounts.auth.key(),                   // collection pdate authority
        ),
        sign_metadata_info.as_slice(),
        &signer,
    )?;

    let set_verify_collection_item_info = vec![
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.auth.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.auth.to_account_info(),
        ctx.accounts.collection_mint.to_account_info(),
        ctx.accounts.collection_metadata.to_account_info(),
        ctx.accounts.collection_master_edition.to_account_info(),
    ];

    // set collection and set to verified
    // note to self, could not get "Collection Details" to work on collection nft, otherwise would use set_and_verify_collection_item
    invoke_signed(
        &set_and_verify_collection(
            ctx.accounts.token_metadata_program.key(), // token metadata program
            ctx.accounts.metadata.key(),               // metadata account
            ctx.accounts.auth.key(),                   // collection pda authority
            ctx.accounts.payer.key(),                  // payer
            ctx.accounts.auth.key(),                   // collection pda authority
            ctx.accounts.collection_mint.key(),        // collection mint PDA
            ctx.accounts.collection_metadata.key(),    // collection metadata account PDA
            ctx.accounts.collection_master_edition.key(), // collection master edition account PDA
            None,
        ),
        set_verify_collection_item_info.as_slice(),
        &signer,
    )?;

    Ok(())
}
