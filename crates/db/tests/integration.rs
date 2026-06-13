//! Integration test: workspace → block chain → export

#[test]
fn full_reasoning_chain_integration() {
    use contextlayer_db::{GraphStore, SaveBlockInput};
    use contextlayer_export::compile_workspace_summary_markdown;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("graph.db");
    let store = GraphStore::open(&path).unwrap();

    let ws = store
        .create_workspace("Integration", "Test goal", "blank")
        .unwrap();

    store
        .save_block(SaveBlockInput {
            workspace_id: ws.id.clone(),
            hypothesis_text: Some("API may allow unauthenticated access".into()),
            action_text: Some("Sent unauthenticated GET to /admin".into()),
            evidence_text: Some("HTTP 200 with admin dashboard HTML".into()),
            conclusion_text: Some("Unauthenticated admin access confirmed".into()),
            conclusion_outcome: Some("confirmed".into()),
            conclusion_tag: Some("act".into()),
            confidence_level: Some("high".into()),
            belief_state: Some("confirmed".into()),
            ..Default::default()
        })
        .unwrap();

    let md = compile_workspace_summary_markdown(&store, &ws.id).unwrap();
    assert!(md.contains("Reasoning blocks"));
    assert!(md.contains("API may allow"));
    assert!(md.contains("Unauthenticated admin access"));

    let blocked = store.save_block(SaveBlockInput {
        workspace_id: ws.id.clone(),
        conclusion_text: Some("Orphan conclusion".into()),
        conclusion_outcome: Some("uncertain".into()),
        ..Default::default()
    });
    assert!(blocked.is_err());
}
