use anchor_lang::{ prelude::*, system_program };
use anchor_spl::token::{ self, Token };
use anchor_spl::token_2022::{ transfer_checked, TransferChecked };
use anchor_spl::token_interface::{ Mint, TokenAccount };

pub fn transfer_token<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: &u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    signer_seeds: Option<&[&[&[u8]]]>
) -> Result<()> {
    let transfer_accounts_options = TransferChecked {
        mint: mint.to_account_info(),
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let mut cpi_context = CpiContext::new(
        token_program.to_account_info(),
        transfer_accounts_options
    );
    if let Some(seeds) = signer_seeds {
        let seeds_slice = seeds;
        cpi_context = cpi_context.with_signer(seeds_slice);
        transfer_checked(cpi_context, *amount, mint.decimals)
    } else {
        transfer_checked(cpi_context, *amount, mint.decimals)
    }
}

pub fn transfer_sol<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    amount: u64
) -> Result<()> {
    // transfer sol to token account
    let cpi_context = CpiContext::new(system_program.to_account_info(), system_program::Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
    });
    system_program::transfer(cpi_context, amount)?;

    // Sync the native token to reflect the new SOL balance as wSOL
    let cpi_accounts = token::SyncNative {
        account: to.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::sync_native(cpi_ctx)?;
    Ok(())
}
