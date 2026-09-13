use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn image(bytes: usize) -> serde_json::Value {
    let prefix = "data:image/png;base64,";
    json!({"type": "input_image", "image_url": format!("{prefix}{}", "A".repeat(bytes - prefix.len()))})
}

fn item(kind: &str, images: Vec<serde_json::Value>) -> ResponseItem {
    serde_json::from_value(if kind == "message" {
        json!({"type": kind, "role": "user", "content": images})
    } else {
        json!({"type": kind, "call_id": kind, "output": images})
    })
    .unwrap()
}

#[test]
fn byte_budget_preserves_latest_user_and_tool_pairing() {
    let image = image(5 * 1024 * 1024);
    let text = json!({"type": "input_text", "text": "original tool text"});
    let mut input = vec![
        item("message", vec![image.clone()]),
        item("function_call_output", vec![text.clone(), image.clone()]),
        item("custom_tool_call_output", vec![image.clone()]),
    ];
    let mut expected = input.clone();
    expected[1] = item(
        "function_call_output",
        vec![
            text,
            json!({
                "type": "input_text", "text": ImageBudgetOmission.body()
            }),
        ],
    );
    assert_eq!(
        apply_inline_image_byte_budget(&mut input),
        ImageBudgetStats {
            original_bytes: 15 * 1024 * 1024,
            retained_bytes: 10 * 1024 * 1024,
            retained_images: 2,
            omitted_images: 1,
        }
    );
    assert_eq!(input, expected);
}

#[test]
fn count_budget_protects_only_four_latest_user_images() {
    let mut input = vec![item("message", vec![image(32); 6])];
    input.extend((0..6).map(|_| item("function_call_output", vec![image(32)])));
    let original = input.clone();
    let stats = apply_inline_image_byte_budget(&mut input);
    let omitted = json!({"type": "input_text", "text": ImageBudgetOmission.body()});
    let mut expected = original.clone();
    expected[0] = item(
        "message",
        [vec![omitted.clone(); 2], vec![image(32); 4]].concat(),
    );
    expected[1] = item("function_call_output", vec![omitted.clone()]);
    expected[2] = item("function_call_output", vec![omitted]);
    assert_eq!(input, expected);
    assert_eq!(stats.retained_images, 8);
    let mut retry = original;
    assert_eq!(apply_inline_image_byte_budget(&mut retry), stats);
    assert_eq!(retry, input);
    apply_inline_image_byte_budget(&mut retry);
    assert_eq!(retry, input);
}

#[test]
fn single_oversized_user_image_cannot_bypass_budget() {
    let mut input = vec![item(
        "message",
        vec![image(MAX_REQUEST_INLINE_IMAGE_BYTES + 1)],
    )];
    let stats = apply_inline_image_byte_budget(&mut input);
    assert_eq!(stats.retained_bytes, 0);
    assert_eq!(
        input,
        vec![item(
            "message",
            vec![json!({
                "type": "input_text", "text": ImageBudgetOmission.body()
            })]
        )]
    );
}

#[test]
fn remote_urls_non_base64_and_unrelated_content_are_unchanged() {
    let mut input = vec![item(
        "message",
        vec![
            image(MAX_REQUEST_INLINE_IMAGE_BYTES),
            json!({"type": "input_image", "image_url": "https://example.com/image.png"}),
            json!({"type": "input_image", "image_url": "data:image/png,raw"}),
            json!({"type": "input_audio", "audio_url": "data:audio/wav;base64,AAAA"}),
            json!({"type": "input_text", "text": "keep this text exactly"}),
        ],
    )];
    let expected = input.clone();
    let stats = apply_inline_image_byte_budget(&mut input);
    assert_eq!(stats.omitted_images, 0);
    assert_eq!(input, expected);
}

#[test]
fn latest_text_only_user_message_releases_previous_user_priority() {
    let mut input = vec![item("message", vec![image(32)])];
    input.extend((0..8).map(|_| item("custom_tool_call_output", vec![image(32)])));
    input.push(item(
        "message",
        vec![json!({"type": "input_text", "text": "continue"})],
    ));
    let mut expected = input.clone();
    expected[0] = item(
        "message",
        vec![json!({"type": "input_text", "text": ImageBudgetOmission.body()})],
    );
    apply_inline_image_byte_budget(&mut input);
    assert_eq!(input, expected);
}
