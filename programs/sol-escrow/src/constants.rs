pub const ANCHOR_DISCRIMINATOR: usize = 8;

// 订单超时时间最小值，单位：秒（必须至少比当前时间晚 60 秒）
pub const MIN_EXPIRATION_TIME: i64 = 60;