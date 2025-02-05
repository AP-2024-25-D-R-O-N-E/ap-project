use crate::fragmentation::message::Message;
use crate::fragmentation::message::MessageData;

#[test]
fn test() {
    let message = Message::new(0, 1, MessageData::ResponseClients(vec![0, 4, 2, 6]));

    println!("original {:?}", message);

    let a = bincode::serialize(&message).unwrap();

    println!("{:?}", a);

    let reconstructed: Message = bincode::deserialize(&a).unwrap();

    println!("{:?}", reconstructed);
}
