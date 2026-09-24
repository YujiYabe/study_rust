use actix_web::{App, test, web::Data};
use serde_json::{Value, json};
use task_api::{ApiSchema, TaskRepository, configure, create_schema};

fn schema() -> Data<ApiSchema> {
    // 各テスト専用のインメモリDBを使い、実ファイルへ副作用を残さない。
    let repository = TaskRepository::connect(":memory:").unwrap();
    Data::new(create_schema(repository))
}

// GraphQLではすべての操作をPOST /graphqlへ送り、クエリ文字列で操作内容を指定する。
fn graphql_request(query: &str, variables: Value) -> actix_http::Request {
    test::TestRequest::post()
        .uri("/graphql")
        .set_json(json!({"query": query, "variables": variables}))
        .to_request()
}

#[actix_web::test]
async fn task_crud_flow() {
    let app = test::init_service(App::new().app_data(schema()).configure(configure)).await;

    let request = graphql_request(
        r#"mutation($input: CreateTask!) {
            createTask(input: $input) { id title description completed }
        }"#,
        json!({"input": {"title": "learn async-graphql", "description": "week 8"}}),
    );
    let response: Value = test::call_and_read_body_json(&app, request).await;
    assert!(response.get("errors").is_none(), "{response}");
    let created = &response["data"]["createTask"];
    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["completed"], false);

    let request = graphql_request(
        r#"mutation($id: Int!, $input: UpdateTask!) {
            updateTask(id: $id, input: $input) { title completed }
        }"#,
        json!({
            "id": id,
            "input": {"title": "learn GraphQL", "description": null, "completed": true}
        }),
    );
    let response: Value = test::call_and_read_body_json(&app, request).await;
    assert!(response.get("errors").is_none(), "{response}");
    assert_eq!(response["data"]["updateTask"]["title"], "learn GraphQL");

    let request = graphql_request(
        "query { tasks(completed: true) { id title completed } }",
        json!({}),
    );
    let response: Value = test::call_and_read_body_json(&app, request).await;
    assert_eq!(response["data"]["tasks"].as_array().unwrap().len(), 1);

    let request = graphql_request(
        "mutation($id: Int!) { deleteTask(id: $id) }",
        json!({"id": id}),
    );
    let response: Value = test::call_and_read_body_json(&app, request).await;
    assert_eq!(response["data"]["deleteTask"], true);

    let request = graphql_request(
        "query($id: Int!) { task(id: $id) { id } }",
        json!({"id": id}),
    );
    let response: Value = test::call_and_read_body_json(&app, request).await;
    assert_eq!(response["data"]["task"], Value::Null);
}

#[actix_web::test]
async fn returns_a_graphql_error_for_an_empty_title() {
    let app = test::init_service(App::new().app_data(schema()).configure(configure)).await;
    let request = graphql_request(
        r#"mutation { createTask(input: {title: "  "}) { id } }"#,
        json!({}),
    );
    let response: Value = test::call_and_read_body_json(&app, request).await;

    assert_eq!(response["data"], Value::Null);
    assert_eq!(
        response["errors"][0]["extensions"]["code"],
        "VALIDATION_ERROR"
    );
}
