use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    /// 超时时间设置错误
    #[msg("The expiration time is too soon. It must be at least 60 seconds in the future.")]
    ExpirationTooSoon,

    /// 交易超时错误
    #[msg("The expiration time is too far in the future.")]
    ExpirationTooFar,

    /// 金额为0错误
    #[msg("amount must be greater than zero.")]
    AmountZero,

    /// 订单状态不对
    #[msg("Cancellation not allowed in current order status.")]
    CancellationNotAllowed,

    /// 不允许资金释放
    #[msg("The current order status does not allow funds to be released.")]
    FundsReleaseNotAllowed,

    /// 卖家确认订单失败，当前订单状态不允许确认
    #[msg("Seller confirmation not allowed in the current order status.")]
    SellerConfirmationNotAllowed,

    /// NFT 选择无效
    #[msg("Invalid NFT selection: Either collection_mint or nft_mint must be set, but not both.")]
    InvalidNftSelection,

    /// 卖家未提供 NFT 账户
    #[msg("Missing NFT account: The seller must provide a valid NFT account.")]
    MissingNftAccount,

    /// 提供的 NFT 账户与订单不匹配
    #[msg("Invalid NFT account: The provided NFT mint does not match the expected mint in the order.")]
    InvalidNftAccount,

    /// 卖家不是 NFT 的合法持有者
    #[msg("Invalid NFT owner: The seller does not own the specified NFT.")]
    InvalidNftOwner,

    /// NFT 账户的数量无效（应为1）
    #[msg("Invalid NFT amount: The NFT account must contain exactly one NFT.")]
    InvalidNftAmount,

    /// 元数据账户缺失
    #[msg("Missing metadata account: The metadata account for the NFT is required but was not provided.")]
    MissingMetadata,

    /// 无效的元数据账户
    #[msg("Invalid metadata: Failed to parse or verify the metadata account.")]
    InvalidMetadata,

    /// 未提供 NFT mint 账户
    #[msg("Missing NFT mint: The NFT mint account is required but was not provided.")]
    MissingNftMint,

    /// 未提供买家 NFT 账户
    #[msg("Missing buyer NFT account: The buyer's NFT account is required but was not provided.")]
    MissingBuyerNftAccount,

    /// 未提供 NFT 集合 mint 账户
    #[msg("Missing collection mint: The collection mint account is required but was not provided.")]
    MissingCollectionMint,
}
