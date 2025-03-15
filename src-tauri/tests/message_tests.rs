use bytes::Bytes;
use chrono::Utc;
use serde_json;
use uniclipboard::Payload;

#[test]
fn test_payload_new_text() {
    let content = Bytes::from("测试内容");
    let device_id = "test_device".to_string();
    let timestamp = Utc::now();

    let payload = Payload::new_text(
        content.clone(),
        device_id.clone(),
        timestamp,
    );

    assert_eq!(*payload.get_content(), content);
    assert_eq!(payload.get_device_id(), &device_id);
    assert_eq!(payload.get_timestamp(), timestamp);
}

#[test]
fn test_payload_new_image() {
    let content = Bytes::from(vec![0u8; 100]); // 模拟图片数据
    let device_id = "test_device".to_string();
    let timestamp = Utc::now();
    let width = 100;
    let height = 100;
    let format = "png".to_string();
    let size = 100;

    let payload = Payload::new_image(
        content.clone(),
        device_id.clone(),
        timestamp,
        width,
        height,
        format.clone(),
        size,
    );

    assert_eq!(*payload.get_content(), content);
    assert_eq!(payload.get_device_id(), &device_id);
    assert_eq!(payload.get_timestamp(), timestamp);
    if let Payload::Image(image_payload) = payload {
        assert_eq!(image_payload.width, width);
        assert_eq!(image_payload.height, height);
        assert_eq!(image_payload.format, format);
        assert_eq!(image_payload.size, size);
    } else {
        panic!("Expected ImagePayload");
    }
}

#[test]
fn test_payload_hash() {
    let payload1 = Payload::new_text(
        Bytes::from("内容1"),
        "device1".to_string(),
        Utc::now(),
    );
    let payload2 = Payload::new_text(
        Bytes::from("内容1"),
        "device2".to_string(),
        Utc::now(),
    );
    let payload3 = Payload::new_text(
        Bytes::from("内容2"),
        "device1".to_string(),
        Utc::now(),
    );

    assert_eq!(payload1.get_key(), payload2.get_key());
    assert_ne!(payload1.get_key(), payload3.get_key());
}

#[test]
fn test_payload_eq() {
    let payload1 = Payload::new_text(
        Bytes::from("内容"),
        "device1".to_string(),
        Utc::now(),
    );
    let payload2 = Payload::new_text(
        Bytes::from("内容"),
        "device2".to_string(),
        Utc::now(),
    );
    let payload3 = Payload::new_text(
        Bytes::from("不同内容"),
        "device1".to_string(),
        Utc::now(),
    );

    assert!(payload1.eq(&payload2));
    assert!(!payload1.eq(&payload3));
}

#[test]
fn test_text_payload_to_json() {
    let content = Bytes::from("JSON测试");
    let device_id = "json_device".to_string();
    let timestamp = Utc::now();

    let payload = Payload::new_text(
        content.clone(),
        device_id.clone(),
        timestamp,
    );

    let json_string = payload.to_json();
    let deserialized: Payload = serde_json::from_str(&json_string).unwrap();

    if let Payload::Text(text_payload) = deserialized {
        assert_eq!(text_payload.content, content);
        assert_eq!(text_payload.device_id, device_id);
        assert_eq!(text_payload.timestamp, timestamp);
    } else {
        panic!("Expected TextPayload");
    }
}

#[test]
fn test_image_payload_to_json() {
    let content = Bytes::from(vec![0u8; 100]); // 模拟图片数据
    let device_id = "json_device".to_string();
    let timestamp = Utc::now();
    let width = 100;
    let height = 100;
    let format = "png".to_string();
    let size = 100;

    let payload = Payload::new_image(
        content.clone(),
        device_id.clone(),
        timestamp,
        width,
        height,
        format.clone(),
        size,
    );

    let json_string = payload.to_json();
    let deserialized: Payload = serde_json::from_str(&json_string).unwrap();

    if let Payload::Image(image_payload) = deserialized {
        assert_eq!(image_payload.content, content);
        assert_eq!(image_payload.device_id, device_id);
        assert_eq!(image_payload.timestamp, timestamp);
        assert_eq!(image_payload.width, width);
        assert_eq!(image_payload.height, height);
        assert_eq!(image_payload.format, format);
        assert_eq!(image_payload.size, size);
    } else {
        panic!("Expected ImagePayload");
    }
}

#[test]
fn test_payload_serialization() {
    let text_payload = Payload::new_text(
        Bytes::from("序列化测试"),
        "serialize_device".to_string(),
        Utc::now(),
    );

    let serialized = serde_json::to_string(&text_payload).unwrap();
    let deserialized: Payload = serde_json::from_str(&serialized).unwrap();

    assert_eq!(text_payload.get_content(), deserialized.get_content());
    assert_eq!(text_payload.get_device_id(), deserialized.get_device_id());
    assert_eq!(text_payload.get_timestamp(), deserialized.get_timestamp());

    let image_payload = Payload::new_image(
        Bytes::from(vec![0u8; 100]),
        "serialize_device".to_string(),
        Utc::now(),
        100,
        100,
        "png".to_string(),
        100,
    );

    let serialized = serde_json::to_string(&image_payload).unwrap();
    let deserialized: Payload = serde_json::from_str(&serialized).unwrap();

    assert_eq!(image_payload.get_content(), deserialized.get_content());
    assert_eq!(image_payload.get_device_id(), deserialized.get_device_id());
    assert_eq!(image_payload.get_timestamp(), deserialized.get_timestamp());
    
    if let Payload::Image(img) = image_payload {
        if let Payload::Image(deserialized_img) = deserialized {
            assert_eq!(img.width, deserialized_img.width);
            assert_eq!(img.height, deserialized_img.height);
            assert_eq!(img.format, deserialized_img.format);
            assert_eq!(img.size, deserialized_img.size);
        } else {
            panic!("Deserialized payload is not an image");
        }
    } else {
        panic!("Original payload is not an image");
    }
}
