use anchor_lang::prelude::*;
use mpl_token_metadata::state::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("FxHLWgvCiF8PxSJ94E1kckuA6RkFYa3N1MhBTJaQEjn4");

#[program]
pub mod nft_marketplace {

    use super::*;

    pub fn initialize_marketplace(ctx: Context<InitializeMarketplace>) -> Result<()> {
        todo!("initialize marketplace");
    }

    pub fn list_nft(ctx: Context<ListNft>) -> Result<()> {

        // example of token transfer
        // let cpi_ctx = CpiContext::new(
        //     ctx.accounts.token_program.to_account_info(),
        //     token::Transfer {
        //         from: ctx.accounts.nft_associated_account.to_account_info(),
        //         to: ctx.accounts.nft_holder_account.to_account_info(),
        //         authority: ctx.accounts.signer.to_account_info(),
        //     },
        // );
        // token::transfer(cpi_ctx, 1)?;
        todo!("initialize marketplace");
    }

    pub fn list_nft_in_spl(ctx: Context<ListNftInSpl>) -> Result<()> {
        todo!("initialize marketplace");
    }

    pub fn update_price(ctx: Context<UpdatePrice>) -> Result<()> {
        todo!("initialize marketplace");
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        todo!("initialize marketplace");
    }

    pub fn buy_nft(ctx: Context<BuyNft>) -> Result<()> {
        // examples of how to get creators and remaining acounts
        let creator_accounts = ctx.remaining_accounts;

        let metadata: Metadata = Metadata::from_account_info(&ctx.accounts.nft_metadata_account.to_account_info())?;
        let creators_array = metadata.data.creators.unwrap();

        todo!("initialize marketplace");
    }

    pub fn buy_nft_with_spl(ctx: Context<BuyNftInSpl>) -> Result<()> {
        let creator_accounts = ctx.remaining_accounts;
        todo!("initialize marketplace");
    }
}

#[derive(Accounts)]
pub struct InitializeMarketplace {}

#[derive(Accounts)]
pub struct ListNft<'info> {
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ListNftInSpl {}

#[derive(Accounts)]
pub struct UpdatePrice {}

#[derive(Accounts)]
pub struct CancelListing {}


#[derive(Accounts)]
pub struct BuyNft<'info> {

    #[account(mut)]
    /// CHECK: checking in instruction
    pub nft_metadata_account: AccountInfo<'info>,

}

#[derive(Accounts)]
pub struct BuyNftInSpl {}


#[account]
#[derive(Default)]
pub struct GlobalState {
	pub initializer: Pubkey,
	pub total_listed_count_sol: u32,
	pub total_listed_count_spl: u32,

	pub total_volume_all_time_sol: u128,

	pub all_time_sale_count_spl: u64,
	pub all_time_sale_count_sol: u64,
	pub marketplace_fee_percentage: u64
}

#[account]
#[derive(Default)]
pub struct Listing {
    // Marketplace instance global state address
    pub global_state_address: Pubkey,

    // User who listed this nft
    pub initializer: Pubkey,
    // NFT mint address
    pub nft_mint_address: Pubkey,
    // Program PDA account address, who holds NFT now
    pub nft_holder_address: Pubkey,
    // Price of this NFT.
    pub price: u64,

    // listing creation time
    pub creation_time: i64,
    pub updated_at: i64,

    // if trade payment is in spl token currency
    pub is_spl_listing: bool,
    // trade spl token address
    pub trade_spl_token_mint_address: Pubkey,
    pub trade_spl_token_seller_account_address: Pubkey,
}