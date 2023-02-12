# sqlx-sqlhelper
基于`sqlx`和`过程宏`实现的`sqlhelper`生成。

目前只支持`sqlx`的`mysql`生成。
## 依赖
需要首先在您的项目中添加`sqlx`的依赖。
``` toml
sqlx = {version = "0.6", features = ["runtime-tokio-rustls", "mysql", "chrono", "decimal"]}
```
## 实现的宏
### SqlHelper
`SqlHelper`是`derive`过程宏。主要实现了`struct`的`find`、`list`、`delete`、`add`、`update`、`save_or_update`、`new`、`new_common`、`base_page`、`base_count`等常用查询方法。
### common_fields
`common_fields`类属性宏对常用`id`、`create_time`、`update_time`等字段的自动添加。依赖`SqlHelper`宏。
### sql_args
`sql_args`声明宏主要是为了方便生成`sqlx`的`MySqlArguments`对象。
``` rust
let (sql, args) = sql_args!("user_name = ?", "张三");
```
## 使用方法
1、创建一个`db.rs`文件，代码如下。

该文件主要作用是配置`sqlx`和`mysql`数据库的连接。
``` rust
use lazy_static::lazy_static;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env::var;

lazy_static! {
    pub static ref POOL: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&format!(
            "mysql://{}:{}@{}/{}",
            var("db_user").expect("配置文件db_user错误"),
            var("db_pass").expect("配置文件db_pass错误"),
            var("db_host").expect("配置文件db_host错误"),
            var("db_name").expect("配置文件db_host错误"),
        ))
        .unwrap();
}
```

2、在struct的上下文中引入`sqlx`的`db`对象。

``` rust
//此处use需要根据db.rs位置进行引用。
use super::db;

use billing_api_macros::{common_fields, SqlHelper};
use chrono::NaiveDateTime;
use poem_openapi::Object;

/// 用户表
#[common_fields]
#[derive(sqlx::FromRow, Debug, Object, SqlHelper)]
pub struct User {
    /// 登录账号
    pub account: String,
    /// 登录密码
    pub pwd: String,
    /// 登录token
    pub login_token: String,
    /// 登录token过期时间
    pub login_token_expire_date: NaiveDateTime,
    /// 最后登录时间
    pub last_login_time: NaiveDateTime,
    /// 最后登录ip
    pub last_login_ip: String,
}
```
SqlHelper宏展开之后的代码。
``` rust
// Recursive expansion of SqlHelper! macro
// ========================================

impl User {
    pub async fn find(id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as:: <_,Self>("SELECT id, account, pwd, login_token, login_token_expire_date, last_login_time, last_login_ip, create_time, update_time FROM user WHERE id = ?").bind(id).fetch_one(& *db::POOL).await
    }
    pub async fn list() -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as:: <_,Self>("SELECT id, account, pwd, login_token, login_token_expire_date, last_login_time, last_login_ip, create_time, update_time FROM user").fetch_all(& *db::POOL).await
    }
    pub async fn delete(&self) -> Result<bool, sqlx::Error> {
        sqlx::query("DELETE FROM user WHERE id = ?")
            .bind(self.id)
            .execute(&*db::POOL)
            .await
            .map(|f| f.rows_affected() > 0)
    }
    pub async fn insert(&self) -> Result<Self, sqlx::Error> {
        let sql = "INSERT INTO user (account, pwd, login_token, login_token_expire_date, last_login_time, last_login_ip, create_time, update_time) VALUES(?, ?, ?, ?, ?, ?, ?, ?)";
        let last_id = sqlx::query(sql)
            .bind(&self.account)
            .bind(&self.pwd)
            .bind(&self.login_token)
            .bind(self.login_token_expire_date)
            .bind(self.last_login_time)
            .bind(&self.last_login_ip)
            .bind(self.create_time)
            .bind(self.update_time)
            .execute(&*db::POOL)
            .await?
            .last_insert_id();
        Self::find(last_id as i32).await
    }
    #[doc = r" 如果定义的`create_time`，`update_time`字段是`Default::default()`默认值，则更新为当前时间"]
    #[doc = r""]
    #[doc = r" `Default::default()`一般为`1970-01-01T00:00:00`等"]
    pub async fn insert_auto_time(&mut self) -> Result<Self, sqlx::Error> {
        if self.create_time == Default::default() {
            self.create_time = chrono::Local::now().naive_local();
        }
        if self.update_time == Default::default() {
            self.update_time = chrono::Local::now().naive_local();
        }
        self.insert().await
    }
    pub async fn update(&self) -> Result<bool, sqlx::Error> {
        let sql = "UPDATE user SET account = ?, pwd = ?, login_token = ?, login_token_expire_date = ?, last_login_time = ?, last_login_ip = ?, create_time = ?, update_time = ? WHERE id = ?";
        sqlx::query(sql)
            .bind(&self.account)
            .bind(&self.pwd)
            .bind(&self.login_token)
            .bind(self.login_token_expire_date)
            .bind(self.last_login_time)
            .bind(&self.last_login_ip)
            .bind(self.create_time)
            .bind(self.update_time)
            .bind(self.id)
            .execute(&*db::POOL)
            .await
            .map(|f| f.rows_affected() > 0)
    }
    #[doc = r" 如果定义的update_time字段是`Default::default()`默认值，则更新为当前时间"]
    #[doc = r""]
    #[doc = r" `Default::default()`一般为`1970-01-01T00:00:00`等"]
    pub async fn update_auto_time(&mut self) -> Result<bool, sqlx::Error> {
        if self.update_time == Default::default() {
            self.update_time = chrono::Local::now().naive_local();
        }
        self.update().await
    }
    #[doc = r" 调用`save_or_update`方法时有一定风险"]
    #[doc = r""]
    #[doc = r" `save_or_update`只是简单判断id是否大于0，大于0则更新，小于等于0则插入。"]
    #[doc = r""]
    #[doc = r" 此时如果手动将`id`赋值为大于0时，会出现更新其他数据的情况，请注意这一块。"]
    pub async fn save_or_update(&self) -> Result<bool, sqlx::Error> {
        match self.id > 0 {
            true => self.update().await,
            false => self.insert().await.map(|_| true),
        }
    }
    #[doc = r" 如果定义的update_time字段是`Default::default()`默认值，则更新为当前时间"]
    #[doc = r""]
    #[doc = r" Default::default()一般为`1970-01-01T00:00:00`等"]
    #[doc = r""]
    #[doc = r" 调用`save_or_update`方法时有一定风险"]
    #[doc = r""]
    #[doc = r" `save_or_update`只是简单判断id是否大于0，大于0则更新，小于等于0则插入。"]
    #[doc = r""]
    #[doc = r" 此时如果手动将`id`赋值为大于0时，会出现更新其他数据的情况，请注意这一块。"]
    pub async fn save_or_update_auto_time(&mut self) -> Result<bool, sqlx::Error> {
        match self.id > 0 {
            true => self.update_auto_time().await,
            false => self.insert_auto_time().await.map(|_| true),
        }
    }
    pub fn new(
        account: String,
        pwd: String,
        login_token: String,
        login_token_expire_date: NaiveDateTime,
        last_login_time: NaiveDateTime,
        last_login_ip: String,
        create_time: chrono::NaiveDateTime,
        update_time: chrono::NaiveDateTime,
    ) -> Self {
        Self {
            id: 0,
            account,
            pwd,
            login_token,
            login_token_expire_date,
            last_login_time,
            last_login_ip,
            create_time,
            update_time,
        }
    }
    pub fn new_common(
        account: String,
        pwd: String,
        login_token: String,
        login_token_expire_date: NaiveDateTime,
        last_login_time: NaiveDateTime,
        last_login_ip: String,
    ) -> Self {
        Self::new(
            account,
            pwd,
            login_token,
            login_token_expire_date,
            last_login_time,
            last_login_ip,
            chrono::Local::now().naive_local(),
            chrono::Local::now().naive_local(),
        )
    }
    pub async fn base_page(
        page_index: i32,
        page_size: i32,
        where_sql: &str,
        args: sqlx::mysql::MySqlArguments,
    ) -> Result<(Vec<Self>, i32, i32, i32), sqlx::Error> {
        let mut index = page_index - 1;
        if index < 0 {
            index = 0;
        }
        let rows = page_size;
        let (count,) = Self::base_count(where_sql, args.clone()).await?;
        let arr = match count > 0 {
            true => {
                let sql = format!("SELECT id, account, pwd, login_token, login_token_expire_date, last_login_time, last_login_ip, create_time, update_time FROM user WHERE {} LIMIT {}, {}",where_sql,index*rows,rows);
                sqlx::query_as_with::<_, Self, sqlx::mysql::MySqlArguments>(&sql, args)
                    .fetch_all(&*db::POOL)
                    .await?
            }
            false => Vec::new(),
        };
        let total_page = (count as f32 / page_size as f32).ceil();
        Ok((arr, count, index + 1, total_page as i32))
    }
    pub async fn base_count(
        where_sql: &str,
        args: sqlx::mysql::MySqlArguments,
    ) -> Result<(i32,), sqlx::Error> {
        let count_sql = format!("SELECT count(1) FROM user WHERE {}", where_sql);
        sqlx::query_as_with::<_, (i32,), sqlx::mysql::MySqlArguments>(&count_sql, args)
            .fetch_one(&*db::POOL)
            .await
    }
}
```
## 示例
参考`examples`中的demo

## 注意