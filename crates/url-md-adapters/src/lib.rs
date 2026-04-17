//! url-md 站点适配器矩阵.
//!
//! Phase 1 MVP: 交付 `generic`(兜底) + `weixin`(首个特化).
//! 更多 adapter (zhihu/substack/github/twitter) 后续阶段添加.

pub mod generic;
pub mod weixin;

use url_md_core::Registry;

pub use generic::GenericAdapter;
pub use weixin::WeixinAdapter;

/// 注册全部已实现的 adapter. 调用方(通常是 CLI `main`)在启动时调一次.
///
/// 注册顺序决定 Registry 的匹配优先级: **特化 adapter 在前,generic 在最后**.
pub fn register_all(registry: &mut Registry) {
    registry.register(WeixinAdapter::new());
    registry.register(GenericAdapter::new()); // 兜底,必须最后
}
