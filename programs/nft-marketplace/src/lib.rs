use anchor_lang::prelude::*;
use mpl_token_metadata::state::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("FxHLWgvCiF8PxSJ94E1kckuA6RkFYa3N1MhBTJaQEjn4");

const MIN_PRICE: u64 = 5;
const MAX_PRICE: u64 = 1000000000000;
const MAX_FEE: u64 = 40000;

#[program]
pub mod nft_marketplace {

    use super::*;

    impl<'info> InitializeMarketplace<'info> {
        pub fn initialize_marketplace(ctx: Context<InitializeMarketplace>, marketplace_fee_percentage: u64) -> Result<()> {
            if marketplace_fee_percentage > MAX_FEE {
                return Err(error!(ErrorCode::HighMarketplaceFee));
            }
            let global_state = &mut ctx.accounts.global_state;
            global_state.marketplace_fee_percentage = marketplace_fee_percentage;
            global_state.initializer = ctx.accounts.user.key();
            Ok(())
        }
    }

    impl<'info> ListNft<'info> {
        pub fn list_nft(ctx: Context<ListNft>, price: u64) -> Result<()> {
            if price < MIN_PRICE || price > MAX_PRICE {
                return Err(error!(ErrorCode::PriceOutOfRange));
            }
            let accounts_listing = &mut ctx.accounts.listing;
            accounts_listing.is_spl_listing = false;

            accounts_listing.price = price;
            accounts_listing.global_state_address = ctx.accounts.global_state.key();
            accounts_listing.initializer = ctx.accounts.user.key();
            accounts_listing.nft_holder_address = ctx.accounts.nft_holder_account.key();
            accounts_listing.creation_time = Clock::get()?.unix_timestamp;
            accounts_listing.updated_at = accounts_listing.creation_time;
            accounts_listing.nft_mint_address = ctx.accounts.nft_mint_address.key();

            let cpi_accounts = Transfer {
                from: ctx.accounts.nft_associated_account.to_account_info(),
                to: ctx.accounts.nft_holder_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };
            let program = ctx.accounts.token_program.to_account_info();
            let ctx = CpiContext::new(program, cpi_accounts);
            token::transfer(ctx, 1)?;

            Ok(())
        }
    }

    impl<'info> ListNftInSpl<'info> {
        pub fn list_nft_in_spl(ctx: Context<ListNftInSpl>, price: u64, trade_spl_token_mint_address: Pubkey, trade_spl_token_seller_account_address: Pubkey) -> Result<()> {
            let listing = &mut ctx.accounts.listing;
            listing.price = price;

            listing.global_state_address = ctx.accounts.global_state.key();
            listing.initializer = ctx.accounts.user.key();
            listing.nft_mint_address = ctx.accounts.nft_mint_address.key();
            listing.nft_holder_address = ctx.accounts.nft_holder_account.key();
            listing.updated_at = listing.creation_time;
            listing.is_spl_listing = true;
            listing.trade_spl_token_mint_address = trade_spl_token_mint_address;
            listing.creation_time = Clock::get()?.unix_timestamp;
            listing.trade_spl_token_seller_account_address = trade_spl_token_seller_account_address;

            let cpi_accounts = Transfer {
                from: ctx.accounts.nft_associated_account.to_account_info(),
                to: ctx.accounts.nft_holder_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, 1)?;

            Ok(())
        }
    }
    
    
    impl<'info> UpdatePrice<'info> {
        pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
            let listing = &mut ctx.accounts.listing;
            listing.price = new_price;
            listing.updated_at = Clock::get()?.unix_timestamp;
            Ok(())
        }
    }
    
    
    impl<'info> CancelListing<'info> {
        pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
            let listing = &mut ctx.accounts.listing;
            if !listing.active {
                return Err(error!(ErrorCode::InactiveListing));
            }
            let cpi_accounts = Transfer {
                from: ctx.accounts.nft_holder_account.to_account_info(),
                to: ctx.accounts.user_nft_account.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.clone();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, 1)?;
            Ok(())
        }
    }
    
    impl<'info> BuyNft<'info> {
        pub fn buy_nft(ctx: Context<BuyNft>, amount: u64) -> Result<()> {
            let listing = &ctx.accounts.listing;
            if amount != listing.price {
                return Err(error!(ErrorCode::IncorrectAmount));
            }
    
            let transfer_nft_accounts = Transfer {
                from: ctx.accounts.nft_holder_account.to_account_info(),
                to: ctx.accounts.buyer_nft_account.to_account_info(),
                authority: ctx.accounts.pda_account.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let transfer_nft_ctx = CpiContext::new_with_signer(
                cpi_program,
                transfer_nft_accounts,
                &[&[&b"nft_marketplace"[..], &[ctx.accounts.pda_account.bump]]],
            );
            token::transfer(transfer_nft_ctx, 1)?;
    
            let marketplace_fee = listing.price * listing.marketplace_fee_percentage / 10000;
            let seller_receive_amount = listing.price - marketplace_fee;
            **ctx.accounts.marketplace_account.to_account_info().lamports.borrow_mut() += marketplace_fee;
            **ctx.accounts.initializer_account.to_account_info().lamports.borrow_mut() += seller_receive_amount;
    
            **ctx.accounts.buyer.to_account_info().lamports.borrow_mut() -= listing.price;
    
            Ok(())
        }
    }

    impl<'info> BuyNft<'info> {
        pub fn buy_nft_with_spl(ctx: Context<BuyNftWithSpl>, amount: u64) -> Result<()> {
            let listing = &ctx.accounts.listing;

            if amount != listing.price {
                return Err(error!(ErrorCode::IncorrectAmount));
            }
            
            if !listing.active {
                return Err(error!(ErrorCode::InactiveListing));
            }
            if ctx.accounts.trade_spl_token_mint_address.key() != listing.trade_spl_token_mint_address {
                return Err(error!(ErrorCode::IncorrectTokenType));
            }


            if !listing.is_spl_listing {
                return Err(error!(ErrorCode::IncorrectPaymentMethod));
            }

            
            let marketplace_fee = amount * listing.marketplace_fee_percentage as u64 / 10000;
            let amount_after_fee = amount - marketplace_fee;
            
            {
                let transfer_to_seller_cpi_accounts = Transfer {
                    from: ctx.accounts.buyer_spl_token_account.to_account_info(),
                    to: ctx.accounts.seller_spl_token_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                };
                let transfer_to_seller_cpi_program = ctx.accounts.token_program.to_account_info();
                let transfer_to_seller_cpi_ctx = CpiContext::new(transfer_to_seller_cpi_program, transfer_to_seller_cpi_accounts);
                token::transfer(transfer_to_seller_cpi_ctx, amount_after_fee)?;
            }
            
            {
                let transfer_fee_cpi_accounts = Transfer {
                    from: ctx.accounts.buyer_spl_token_account.to_account_info(),
                    to: ctx.accounts.marketplace_fee_account.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                };
                let transfer_fee_cpi_program = ctx.accounts.token_program.to_account_info();
                let transfer_fee_cpi_ctx = CpiContext::new(transfer_fee_cpi_program, transfer_fee_cpi_accounts);
                token::transfer(transfer_fee_cpi_ctx, marketplace_fee)?;
            }
            
            {
                let transfer_nft_cpi_accounts = Transfer {
                    from: ctx.accounts.nft_holder_account.to_account_info(),
                    to: ctx.accounts.buyer_nft_account.to_account_info(),
                    authority: ctx.accounts.pda.to_account_info(),
                };
                let transfer_nft_cpi_program = ctx.accounts.token_program.to_account_info();
                let transfer_nft_cpi_ctx = CpiContext::new_with_signer(
                    transfer_nft_cpi_program,
                    transfer_nft_cpi_accounts,
                    &[&[b"nft_marketplace", &[ctx.accounts.listing.bump]]],
                );
                token::transfer(transfer_nft_cpi_ctx, 1)?;
            }
            
            Ok(())
        }
    }
}

#[derive(Accounts)]
#[instruction(marketplace_fee_percentage: u64)]
pub struct InitializeMarketplace<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<GlobalState>(),
        seeds = [b"global_state", user.key().as_ref()],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<Listing>(),
        seeds = [b"listing", nft_mint_address.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        "token_program.key == &token::ID"
    )]
    pub token_program: AccountInfo<'info>,
    pub nft_mint_address: AccountInfo<'info>,
    pub nft_holder_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ListNftInSpl<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<Listing>(),
        seeds = [b"listing", nft_mint_address.key().as_ref(), user.key().as_ref(), b"spl"],
        bump
    )]
    pub listing: Account<'info, Listing>,
    pub nft_mint_address: AccountInfo<'info>,
    #[account(
        "token_program.key == &token::ID"
    )]
    pub token_program: AccountInfo<'info>,
    pub nft_holder_account: Account<'info, TokenAccount>,
    pub trade_spl_token_mint_address: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut, has_one = initializer)]
    pub listing: Account<'info, Listing>,
    pub initializer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut, has_one = initializer, close = initializer)]
    pub listing: Account<'info, Listing>,
    #[account(mut)]
    pub nft_holder_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_nft_account: Account<'info, TokenAccount>,
    pub initializer: Signer<'info>,
    #[account("token_program.key == &token::ID")]
    pub token_program: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct BuyNft<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,
    #[account(mut)]
    pub nft_holder_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub initializer_account: Account<'info, TokenAccount>, 
    #[account(mut)]
    pub marketplace_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account("token_program.key == &token::ID")]
    pub token_program: AccountInfo<'info>,
}

    
#[derive(Accounts)]
pub struct BuyNftWithSpl<'info> {
    #[account(mut)]
    pub listing: Account<'info, Listing>,
    pub trade_spl_token_mint_address: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_spl_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_spl_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub marketplace_fee_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub nft_holder_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_nft_account: Account<'info, TokenAccount>,
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub pda: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}


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

#[error_code]
pub enum ErrorCode {
    #[msg("The fee is too high.")]
    HighMarketplaceFee,
    #[msg("The price is out of range.")]
    PriceOutOfRange,
    #[msg("Insuffcient rent.")]
    InsufficientRent,
    #[msg("listing is not active.")]
    InactiveListing,
    #[msg("You do not have permission.")]
    InvalidAccess,
    #[msg("The payment method is incorrect for this listing.")]
    IncorrectPaymentMethod,
    #[msg("Insufficient funds.")]
    InsufficientFunds,
    #[msg("The token type is invalid.")]
    IncorrectTokenType,
    #[msg("Failed to transfer tokens.")]
    CouldNotTransfer,
    #[msg("Initializer does not match.")]
    WrongInitializer,
}