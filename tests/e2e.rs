mod src;

#[tokio::test]
async fn test_get_expenses() {
    src::config::load_env();
    src::handlers::expenses::test_get_expenses().await;
}

// TODO: IMPLEMENT A NEW ERROR TYOE FOR TESTS ALONE, THE BOXING METHOD LOOKS HORRIBLE!!!

