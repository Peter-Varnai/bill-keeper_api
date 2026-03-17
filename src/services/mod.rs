use crate::models::{ApplicationReport, EarTotals, Expense, Summary};
use std::collections::HashMap;

pub mod pdf_converter;

pub fn calculate_summaries(
    expenses: &[Expense],
    application_reports: &[ApplicationReport],
) -> Vec<Summary> {
    let mut summaries = Vec::new();

    for app_report in application_reports {
        let app_id = app_report.id as u8;

        let app_expenses: Vec<&Expense> = expenses
            .iter()
            .filter(|e| e.application == Some(app_id))
            .collect();

        let total: f64 = app_expenses.iter().map(|e| e.amount).sum();

        // Group by expense_type for details
        let mut type_map: HashMap<u16, f64> = HashMap::new();
        for expense in &app_expenses {
            *type_map.entry(expense.expense_type).or_insert(0.0) += expense.amount;
        }

        let details: Vec<(String, String)> = type_map
            .iter()
            .map(|(type_id, amount)| (get_expense_type_name(*type_id), format_amount(*amount)))
            .collect();

        // Check if target is met
        let is_target_met = if app_report.amount > 0.0 {
            Some(total >= app_report.amount)
        } else {
            None
        };

        summaries.push(Summary {
            application: app_id as i32,
            application_name: app_report.name.clone(),
            total: format_amount(total),
            details,
            target_amount: Some(app_report.amount),
            is_target_met,
        });
    }

    summaries
}

pub fn calculate_ear_totals(expenses: &[Expense]) -> EarTotals {
    let mut bank_total = 0.0;
    let mut cash_total = 0.0;

    for expense in expenses {
        if expense.Bargeldabhebung == Some(true) {
            cash_total += expense.amount;
        } else {
            bank_total += expense.amount;
        }
    }

    EarTotals {
        bank_total: format_amount(bank_total),
        cash_total: format_amount(cash_total),
    }
}

pub fn format_amount(amount: f64) -> String {
    format!("{:.2}", amount)
}

pub fn get_expense_type_name(expense_type: u16) -> String {
    match expense_type {
        0 => "None".to_string(),
        1 => "Honorare Kurator:innen".to_string(),
        2 => "Honorare Texte".to_string(),
        3 => "Honorare Grafik/Layout/Fotos".to_string(),
        4 => "Honorare Künstler:innen – Gruppenausstellung".to_string(),
        5 => "Honorar Künstler:in – Einzelausstellung".to_string(),
        6 => "Materialkosten".to_string(),
        7 => "Reisekosten, Aufenthaltskosten".to_string(),
        8 => "Transporte".to_string(),
        9 => "Öffentlichkeitsarbeit, Marketing, PR, Social-Media".to_string(),
        10 => "Abgaben, Versicherungen".to_string(),
        11 => "Miete Veranstaltungsort".to_string(),
        12 => "Technische Ausstattung".to_string(),
        13 => "Druckkosten Publikation".to_string(),
        14 => "Discotec künstlerische Leitung, Geschäftsführung".to_string(),
        15 => "Bewirtung, Eröffnung".to_string(),
        16 => "Homepage/Internet/EDV".to_string(),
        17 => "Sonstige Bürokosten".to_string(),
        18 => "Büromaterial, Sachgüter".to_string(),
        19 => "Bankkonto/Website-Domäne".to_string(),
        50 => "Getränkespende".to_string(),
        51 => "Förderung MA 7".to_string(),
        52 => "Förderung Bezirk".to_string(),
        53 => "Förderung Bund".to_string(),
        54 => "Habenzinsen".to_string(),
        55 => "Bargeldabhebung".to_string(),
        56 => "Other Income".to_string(),
        _ => format!("Unknown ({})", expense_type),
    }
}
