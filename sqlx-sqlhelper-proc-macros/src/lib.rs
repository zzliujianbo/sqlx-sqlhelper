use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod common_fields;
mod sql_helper;

pub(crate) const DEFAULT_ID_NAME: &str = "id";
pub(crate) const DEFAULT_CREATE_TIME_NAME: &str = "create_time";
pub(crate) const DEFAULT_UPDATE_TIME_NAME: &str = "update_time";

/// 自动生成mysql数据库增删改查方法
///
/// 基于sqlx生成`get_by_id`、`list`、`delete`、`add`、`update`、`save_or_update`、`new`、`new_common`、`base_page`、`base_count`等方法。
///
///
/// 需要在struct上下文中引入sqlx的db对象。
///
/// ```
/// use super::db;
/// ```
///
/// # Examples
///
/// ```
/// #[derive(SqlHelper)]
/// pub struct Person {
///     #[id]
///     pub id: i32,
///     pub name: String,
///     pub age: i32,
///     pub weight: Option<i32>,
///     #[create_time]
///     pub create_time: NaiveDateTime,
///     #[update_time]
///     pub update_time: NaiveDateTime,
/// }
/// ```
#[proc_macro_derive(SqlHelper, attributes(id, field_name, create_time, update_time))]
pub fn derive_sql_helper(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemStruct);
    sql_helper::impl_sql_helper(&ast)
}

/// 自动实现公用`id`、`create_time`、`update_time`的字段。
/// 
/// 需要配合`SqlHelper`派生宏使用
/// 
/// # Examples
///
/// ```
/// #[common_fields]
/// #[derive(SqlHelper)]
/// pub struct Person {
///     pub name: String,
///     pub age: i32,
///     pub weight: Option<i32>,
/// }
/// ```
#[proc_macro_attribute]
pub fn common_fields(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as ItemStruct);
    common_fields::impl_common_fields(&mut ast)
}
