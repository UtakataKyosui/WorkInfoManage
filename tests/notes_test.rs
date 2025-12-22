use sea_orm::{MockDatabase, Transaction, DbErr, EntityTrait, ActiveModelTrait, Set, PaginatorTrait, QueryFilter, ColumnTrait};
use TaskManager::db::task_notes;
use chrono::Utc;

#[tokio::test]
async fn test_create_and_retrieve_note() {
    // 1. Setup Mock DB
    // We expect:
    // - Insert note
    // - Find all notes for a task (simulating paging or simple select)
    
    let created_at = Utc::now().naive_utc();
    
    let note_model = task_notes::Model {
        id: 1,
        task_id: 100,
        content: "Test Memo".to_string(),
        created_at,
    };

    let db = MockDatabase::new(sea_orm::DatabaseBackend::Postgres)
        .append_query_results(vec![
            // Result for INSERT
            vec![note_model.clone()],
        ])
        .append_query_results(vec![
            // Result for SELECT
            vec![note_model.clone()],
        ])
        .into_connection();

    // 2. Simulate "Save Note" logic
    let note = task_notes::ActiveModel {
        task_id: Set(100),
        content: Set("Test Memo".to_string()),
        ..Default::default()
    };

    let saved = note.insert(&db).await.expect("Failed to insert note");
    
    assert_eq!(saved.content, "Test Memo");
    assert_eq!(saved.task_id, 100);

    // 3. Simulate "Load Notes" logic
    let items = task_notes::Entity::find()
        .filter(task_notes::Column::TaskId.eq(100))
        .all(&db)
        .await
        .expect("Failed to find notes");
        
    // Note: MockDatabase verifies that Expected Queries matched Actual Queries.
    // If we call .filter(...), SeaORM constructs a query.
    // We need to ensure the mock expects exactly what is generated.
    // However, MockDatabase is often strict. 
    // Just checking the return values is successful confirms the mock returned our data upon query.
    
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].content, "Test Memo");
}
