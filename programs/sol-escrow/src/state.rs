use anchor_lang::prelude::*;


#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
     /// 交易刚创建，等待买家存款
    Created,
    /// 买家已存款，等待卖家响应
    Funded,
    /// 卖家确认发货/同意交易
    InTransit,
    /// 交易完成
    Success,
    /// 交易已取消，资金退还买家
    Cancelled,
    /// 交易进入争议状态
    Disputed,
    /// 交易超时未完成
    Expired,
}

impl TransactionStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TransactionStatus::Created),
            1 => Some(TransactionStatus::Funded),
            2 => Some(TransactionStatus::InTransit),
            3 => Some(TransactionStatus::Success),
            4 => Some(TransactionStatus::Cancelled),
            5 => Some(TransactionStatus::Disputed),
            6 => Some(TransactionStatus::Expired),
            _ => None
        }
    }
}


#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub buyer: Pubkey,  // 买家
    pub seller: Option<Pubkey>,  // 卖家
    pub token_mint: Pubkey,  // 交易的spl代币
    pub buyer_nft_account: Option<Pubkey>, // 买家nft账户
    pub nft_mint: Option<Pubkey>, // 购买的NFT的mint地址
    pub amount: u64,  // 交易金额
    pub escrow_vault: Pubkey,  // 资金托管账户
    pub is_nft: bool, // 是否是nft交易
    pub expiration: i64,  // 交易超时时间
    pub status: u8,  // 交易状态
    pub arbitrator: Option<Pubkey>  // 仲裁者
}

impl Escrow {
    pub fn get_transaction_status(&self) -> Option<TransactionStatus> {
        TransactionStatus::from_u8(self.status)
    }
}
