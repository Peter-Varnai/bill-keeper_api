mod src;

#[tokio::test]
async fn test_expenses_get() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_get()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_expenses_create() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_create()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_expenses_update_bill() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_update_bill()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_expenses_update_type() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_update_type()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_expenses_update_application() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_update_application()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_expenses_update_cash() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_update_cash()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_expenses_delete() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_delete()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_expenses_bulk_import() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_bulk_import()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_data_groups_get() {
    src::config::load_env();
    src::handlers::data_groups::test_data_groups_get()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_data_groups_create() {
    src::config::load_env();
    src::handlers::data_groups::test_data_groups_create()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_get() {
    src::config::load_env();
    src::handlers::bills::test_bills_get()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_get_by_id() {
    src::config::load_env();
    src::handlers::bills::test_bills_get_by_id()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_update() {
    src::config::load_env();
    src::handlers::bills::test_bills_update()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_delete() {
    src::config::load_env();
    src::handlers::bills::test_bills_delete()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_upload_jpg() {
    src::config::load_env();
    src::handlers::bills::test_bills_upload_jpg()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_upload_png() {
    src::config::load_env();
    src::handlers::bills::test_bills_upload_png()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_upload_pdf() {
    src::config::load_env();
    src::handlers::bills::test_bills_upload_pdf()
        .await
        .expect("failed");
}

#[tokio::test]
async fn test_bills_upload_unsupported() {
    src::config::load_env();
    src::handlers::bills::test_bills_upload_unsupported()
        .await
        .expect("failed");
}
