use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(crate::handlers::bills::get_bills)
            .service(crate::handlers::bills::get_bill)
            .service(crate::handlers::bills::update_bill)
            .service(crate::handlers::bills::upload_bills)
            .service(crate::handlers::bills::delete_bill)
            .service(crate::handlers::expenses::get_expenses)
            .service(crate::handlers::expenses::create_expense)
            .service(crate::handlers::expenses::bulk_import_expenses)
            .service(crate::handlers::expenses::update_expense_bill)
            .service(crate::handlers::expenses::update_expense_type)
            .service(crate::handlers::expenses::update_expense_application)
            .service(crate::handlers::expenses::update_expense_cash)
            .service(crate::handlers::expenses::delete_expense)
            .service(crate::handlers::summaries::get_summaries)
            .service(crate::handlers::ear::get_ear)
            .service(crate::handlers::reports::get_report)
            .service(crate::handlers::images::get_image)
            .service(crate::handlers::data_groups::get_data_groups)
            .service(crate::handlers::data_groups::create_data_group)
            .service(crate::handlers::data_groups::delete_data_group)
            .service(crate::handlers::application_reports::get_application_reports)
            .service(crate::handlers::application_reports::create_application_report)
            .service(crate::handlers::application_reports::update_application_report)
            .service(crate::handlers::application_reports::delete_application_report)
            .service(crate::handlers::utild::get_utild)
            .service(crate::handlers::utild::update_utild),
    );
}
