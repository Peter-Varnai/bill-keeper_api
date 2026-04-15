use core::f64;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CreateDataGroupRequest {
    pub name: String,
    pub group_type: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateExpenseRequest {
    pub partner: String,
    pub amount: String,
    pub date: Option<NaiveDate>,
    pub expense_type: Option<i32>,
    pub bill: Option<i32>,
    pub application: Option<i32>,
    pub is_cash: Option<bool>,
    pub data_group: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct CsvImportRequest {
    pub date_format: String,
    pub rows: Vec<CsvRow>,
    pub data_group: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct CsvRow {
    pub partner: String,
    pub amount: Decimal,
    pub date: Option<NaiveDate>,
    pub row_number: usize,
}

#[derive(Deserialize, Debug)]
pub struct CreateApplicationReportRequest {
    pub name: String,
    pub amount: rust_decimal::Decimal,
    pub submission_deadline: Option<NaiveDate>,
    pub data_group: i32,
}

#[derive(Deserialize, Debug)]
pub struct UpdateApplicationReportRequest {
    pub name: Option<String>,
    pub amount: Option<f64>,
    pub submission_deadline: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BillNumberUpdate {
    pub expense_id: i32,
    pub new_number: i32,
}
