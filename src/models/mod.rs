use serde::{Deserialize, Serialize};

pub trait HasDate {
    fn get_date(&self) -> Option<&str>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bill {
    pub id: u8,
    pub filename: String,
    pub amount: Option<f32>,
    pub date: Option<String>,
    pub Bargeldabhebung: Option<bool>,
}

impl HasDate for Bill {
    fn get_date(&self) -> Option<&str> {
        self.date.as_deref()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Expense {
    pub id: u16,
    pub date: Option<String>,
    pub partner: String,
    pub amount: f64,
    pub bill: u16,
    pub expense_type: u16,
    pub application: Option<u8>,
    pub Bargeldabhebung: Option<bool>,
}

impl HasDate for Expense {
    fn get_date(&self) -> Option<&str> {
        self.date.as_deref()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillToHtml {
    pub expense_id: u16,
    pub partner: String,
    pub amount: String,
    pub date: Option<String>,
    pub expense_type: i8,
    pub filename: String,
    pub Bargeldabhebung: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct BillQuery {
    pub no: u8,
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
    pub data_group_id: i32,
    pub name: String,
    pub amount: f64,
    pub date_created: String,
    pub submission_deadline: Option<String>,
}
