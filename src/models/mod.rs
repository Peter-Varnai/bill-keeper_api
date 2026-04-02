use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bill {
    pub id: i32,
    pub data_group: i32,
    pub filename: String,
    pub amount: Option<f64>,
    pub date: Option<String>,
    pub is_cash: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expense {
    pub id: i32,
    pub data_group: i32,
    // pub date: Option<String>,
    pub date: chrono::NaiveDate,
    pub partner: String,
    pub amount: f64,
    pub expense_type: i32,
    pub bill: Option<i32>,
    pub application: Option<i32>,
    pub is_cash: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillToHtml {
    pub expense_id: i32,
    pub partner: String,
    pub amount: String,
    pub date: Option<String>,
    pub expense_type: i32,
    pub filename: String,
    pub is_cash: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct BillNumberUpdate {
    pub expense_id: i32,
    pub new_number: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Report {
    pub name: String,
    pub bills: Vec<BillToHtml>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Summary {
    pub application: i32,
    pub application_name: String,
    pub total: String,
    pub details: Vec<(String, String)>,
    pub target_amount: Option<f64>,
    pub is_target_met: Option<bool>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EarTotals {
    pub bank_total: String,
    pub cash_total: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApplicationReport {
    pub id: i32,
    pub data_group: i32,
    pub name: String,
    pub amount: f64,
    pub created_at: String,
    pub submission_deadline: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataGroup {
    pub id: i32,
    pub name: String,
    pub group_type: String,
    pub created_at: String,
    pub bills_storage_path: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateDataGroupRequest {
    pub name: String,
    pub group_type: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateExpenseRequest {
    pub partner: String,
    pub amount: String,
    pub date: Option<String>,
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
    pub amount: String,
    pub date: String,
    pub row_number: usize,
}

#[derive(serde::Serialize)]
pub struct CsvImportResult {
    pub inserted: usize,
    pub duplicates_found: usize,
    pub duplicates_skipped: usize,
    pub errors: Vec<CsvImportError>,
    pub total_processed: usize,
}

#[derive(serde::Serialize)]
pub struct CsvImportError {
    pub row: usize,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct CreateApplicationReportRequest {
    pub name: String,
    pub amount: f64,
    pub submission_deadline: Option<String>,
    pub data_group: i32,
}

#[derive(Deserialize)]
pub struct UpdateApplicationReportRequest {
    pub name: Option<String>,
    pub amount: Option<f64>,
    pub submission_deadline: Option<String>,
}
