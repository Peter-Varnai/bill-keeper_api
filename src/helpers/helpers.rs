use chrono::NaiveDate;

pub fn parse_date_or_panic(date_opt: Option<String>) -> Option<NaiveDate> {
    date_opt.map(|d| {
        NaiveDate::parse_from_str(&d, "%Y-%m-%d")
            .expect("Failed to parse date: expected format YYYY-MM-DD")
    })
}
