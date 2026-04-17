mod src;

#[tokio::test]
async fn test_expenses_crud_flow() {
    src::config::load_env();
    src::handlers::expenses::test_expenses_crud_flow()
        .await
        .expect("test failed");
}