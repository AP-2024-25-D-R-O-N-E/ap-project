use crate::fragmentation::message::Message;
use crate::fragmentation::message::MessageData;

#[test]
fn test() {
    let message = Message::new(
        0,
        1,
        MessageData::TextMessage {
            from: 0,
            to: 5,
            text: "gay".to_string(),
        },
    );

    println!("original {:?}", message);

    let a = message.into_u8();

    println!("{:?}", a);

    let reconstructed: Message = Message::from_u8(a);

    println!("{:?}", reconstructed);
}
