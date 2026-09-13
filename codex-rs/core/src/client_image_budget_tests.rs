use super::*;
use crate::context::ContextualUserFragment;
use crate::context::ImageBudgetOmission;
use crate::responses_metadata::CodexResponsesRequestKind;
use crate::responses_metadata::CompactionTurnMetadata;
use codex_analytics::CompactionImplementation;
use codex_analytics::CompactionPhase;
use codex_analytics::CompactionReason;
use codex_analytics::CompactionTrigger;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn request_image_budget_covers_http_compaction_and_websocket_fallback() -> anyhow::Result<()>
{
    for compaction in [false, true] {
        for websockets in [false, true] {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(ResponseTemplate::new(/*status*/ 426))
                .expect(usize::from(websockets) as u64)
                .mount(&server)
                .await;
            Mock::given(method("POST"))
                .and(path("/v1/responses"))
                .respond_with(ResponseTemplate::new(/*status*/ 200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(concat!(
                        "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp-1\"}}\n\n",
                        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp-1\"}}\n\n",
                    )))
                .expect(/*requests*/ 1).mount(&server).await;
            let mut provider =
                ModelProviderInfo::create_openai_provider(Some(format!("{}/v1", server.uri())));
            provider.requires_openai_auth = false;
            provider.supports_websockets = websockets;
            let mut client = test_model_client(SessionSource::Cli);
            Arc::get_mut(&mut client.state).unwrap().provider =
                create_model_provider(provider, /*auth_manager*/ None);
            let image = json!({"type": "input_image", "image_url": format!("data:image/png;base64,{}", "A".repeat(5 * 1024 * 1024))});
            let mut original = vec![
                serde_json::from_value(
                    json!({"type": "function_call_output", "call_id": "old", "output": [image.clone()]}),
                )?,
                serde_json::from_value(
                    json!({"type": "custom_tool_call_output", "call_id": "recent", "output": [image.clone()]}),
                )?,
                serde_json::from_value(
                    json!({"type": "message", "role": "user", "content": [image.clone()]}),
                )?,
            ];
            if compaction {
                original.push(ResponseItem::CompactionTrigger {});
            }
            let prompt = Prompt {
                input: original.clone(),
                ..Default::default()
            };
            let mut metadata = test_responses_metadata_for_client(
                &client,
                /*turn_id*/ None,
                format!("{}:0", client.state.thread_id),
                /*parent_thread_id*/ None,
                TestCodexResponsesRequestKind::Turn,
            );
            if compaction {
                metadata.request_kind = Some(CodexResponsesRequestKind::Compaction(
                    CompactionTurnMetadata::new(
                        CompactionTrigger::Manual,
                        CompactionReason::UserRequested,
                        CompactionImplementation::ResponsesCompactionV2,
                        CompactionPhase::StandaloneTurn,
                    ),
                ));
            }
            let mut session = client.new_session();
            let mut stream = session
                .stream(
                    &prompt,
                    &test_model_info(),
                    &test_session_telemetry(),
                    /*effort*/ None,
                    codex_protocol::config_types::ReasoningSummary::None,
                    /*service_tier*/ None,
                    &metadata,
                    &InferenceTraceContext::disabled(),
                )
                .await?;
            let mut completed = false;
            while let Some(event) = stream.next().await {
                if let ResponseEvent::Completed { .. } = event? {
                    completed = true;
                }
            }
            assert!(completed);
            let requests = server.received_requests().await.unwrap();
            let request = requests.iter().find(|r| r.method == "POST").unwrap();
            let body: serde_json::Value = serde_json::from_slice(&request.body)?;
            let mut expected = serde_json::to_value(&original)?;
            expected[0]["output"][0] =
                json!({"type": "input_text", "text": ImageBudgetOmission.body()});
            assert_eq!(body["input"][0]["output"][0]["type"], "input_text");
            assert_eq!(body["input"], expected);
            assert_eq!(prompt.input, original);
        }
    }
    Ok(())
}

#[test]
fn request_image_budget_invalidates_websocket_delta_when_old_images_change() -> anyhow::Result<()> {
    let client = test_model_client(SessionSource::Cli);
    let metadata = test_responses_metadata_for_client(
        &client,
        /*turn_id*/ None,
        format!("{}:0", client.state.thread_id),
        /*parent_thread_id*/ None,
        TestCodexResponsesRequestKind::Turn,
    );
    let image: ResponseItem = serde_json::from_value(json!({
        "type": "function_call_output", "call_id": "image",
        "output": [{"type": "input_image", "image_url": "data:image/png;base64,AAAA"}],
    }))?;
    let mut prompt = Prompt {
        input: vec![image.clone(); 8],
        ..Default::default()
    };
    let build = |prompt: &Prompt| {
        client.build_responses_request(
            prompt,
            &test_model_info(),
            /*effort*/ None,
            codex_protocol::config_types::ReasoningSummary::None,
            /*service_tier*/ None,
            &metadata,
        )
    };
    let previous = build(&prompt)?;
    let mut session = client.new_session();
    session.websocket_session.last_request = Some(previous);
    prompt.input.push(image);
    let request = build(&prompt)?;
    assert_eq!(
        session.get_incremental_items(
            &request, /*last_response*/ None, /*allow_empty_delta*/ false
        ),
        None
    );
    assert_eq!(build(&prompt)?, request);
    Ok(())
}
