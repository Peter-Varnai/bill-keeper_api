use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bill {
    pub id: i32,
    pub data_group: i32,
    pub filename: String,
    pub amount: Option<f64>,
    pub date: Option<chrono::NaiveDate>,
    pub is_cash: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expense {
    pub id: i32,
    pub data_group: i32,
    pub date: Option<chrono::NaiveDate>,
    pub partner: String,
    pub amount: Decimal,
    pub expense_type: i32,
    pub bill: Option<i32>,
    pub application: Option<i32>,
    pub is_cash: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationReport {
    pub id: i32,
    pub data_group: i32,
    pub name: String,
    pub amount: f64,
    pub created_at: String,
    pub submission_deadline: Option<chrono::NaiveDate>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataGroup {
    pub id: i32,
    pub name: String,
    pub group_type: String,
    pub created_at: String,
    pub bills_storage_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillToHtml {
    pub expense_id: i32,
    pub partner: String,
    pub amount: Decimal,
    pub date: Option<chrono::NaiveDate>,
    pub expense_type: i32,
    pub filename: String,
    pub is_cash: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BelegaufstellungItem {
    pub expense_id: i32,
    pub partner: String,
    pub amount: Decimal,
    pub date: Option<chrono::NaiveDate>,
    pub expense_type_name: String,
    pub is_cash: Option<bool>,
    pub bill_date: Option<chrono::NaiveDate>,
    pub bill_filename: Option<String>,
}
