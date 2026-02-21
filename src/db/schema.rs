use anyhow::Result;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub async fn initialize(db: &Surreal<Db>) -> Result<()> {
    db.query(r#"
        DEFINE TABLE IF NOT EXISTS todo SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS uuid ON TABLE todo TYPE string;
        DEFINE FIELD IF NOT EXISTS content ON TABLE todo TYPE string;
        DEFINE FIELD IF NOT EXISTS status ON TABLE todo TYPE string
            DEFAULT 'pending';
        DEFINE FIELD IF NOT EXISTS priority ON TABLE todo TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS created_at ON TABLE todo TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS updated_at ON TABLE todo TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS completed_at ON TABLE todo TYPE option<datetime>;
        DEFINE FIELD IF NOT EXISTS project ON TABLE todo TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS project_path ON TABLE todo TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS file_path ON TABLE todo TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS tags ON TABLE todo TYPE array<string> DEFAULT [];
        DEFINE FIELD IF NOT EXISTS metadata ON TABLE todo TYPE option<object>;

        DEFINE INDEX IF NOT EXISTS idx_status ON TABLE todo COLUMNS status;
        DEFINE INDEX IF NOT EXISTS idx_project ON TABLE todo COLUMNS project;
    "#).await?;

    Ok(())
}
