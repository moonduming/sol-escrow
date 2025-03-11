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
}
