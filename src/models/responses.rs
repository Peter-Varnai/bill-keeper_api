use serde::Serialize;

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

#[derive(Serialize, Debug, Clone)]
pub struct Report {
    pub name: String,
    pub bills: Vec<super::entities::BillToHtml>,
}

#[derive(Serialize)]
pub struct CsvImportResult {
    pub inserted: usize,
    pub duplicates_found: usize,
    pub duplicates_skipped: usize,
    pub errors: Vec<CsvImportError>,
    pub total_processed: usize,
}

#[derive(Serialize)]
pub struct CsvImportError {
    pub row: usize,
    pub reason: String,
}
