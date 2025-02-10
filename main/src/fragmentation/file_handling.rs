use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};

use tempfile::{Builder, TempDir};

use super::message::{ChatMessage, RawChatMessage};

pub fn file_to_byte_vec(
    file_path: PathBuf,
) -> Result<(Vec<u8>, OsString, OsString), std::io::Error> {
    // takes a file and converts it to a vector of bytes
    // furthermore it saves the file name in an OsString to use at reconstruction

    let mut file_name: OsString = OsString::new();
    let mut file_extension: OsString = OsString::new();

    if let Some(file_name_res) = file_path.file_stem() {
        file_name = file_name_res.to_os_string();
    }

    if let Some(file_extension_res) = file_path.extension() {
        file_extension = file_extension_res.to_os_string();
    }

    let mut file = File::open(file_path)?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    Ok((buffer, file_name, file_extension))
}

pub fn byte_vec_to_file(
    file_name: OsString,
    file_extension: OsString,
    data: Vec<u8>,
    temp_dir: Arc<TempDir>,
) -> Result<PathBuf, std::io::Error> {
    // creates a temp file inside the temp_dir (uses the std implementation of temp_dir)

    let mut extension = OsString::new();
    extension.push(OsStr::new("."));
    extension.push(file_extension);

    let named_tempfile = Builder::new()
        .prefix(&file_name)
        .suffix(&extension)
        .rand_bytes(3)
        .keep(true)
        .tempfile_in(temp_dir.path())?;

    // writes the data to the temp file
    named_tempfile.as_file().write_all(&data)?;

    Ok(named_tempfile.path().to_path_buf())
}

fn raw_to_chat(raw: RawChatMessage, temp_dir: Arc<TempDir>) -> ChatMessage {
    match raw {
        RawChatMessage::TextMessage { from, to, text } => {
            ChatMessage::TextMessage { from, to, text }
        }
        RawChatMessage::FileMessage {
            from,
            to,
            file,
            file_name,
            extension,
        } => ChatMessage::FileMessage {
            from,
            to,
            file_path: byte_vec_to_file(file_name, extension, file, temp_dir).unwrap(),
        },
    }
}

impl From<ChatMessage> for RawChatMessage {
    fn from(value: ChatMessage) -> Self {
        match value {
            ChatMessage::TextMessage { from, to, text } => {
                RawChatMessage::TextMessage { from, to, text }
            }
            ChatMessage::FileMessage {
                from,
                to,
                file_path,
            } => {
                let (file, file_name, extension) = file_to_byte_vec(file_path).unwrap();
                RawChatMessage::FileMessage {
                    from,
                    to,
                    file,
                    file_name,
                    extension,
                }
            }
        }
    }
}

pub fn raw_vec_to_chat_vec(
    raw_vec: Vec<RawChatMessage>,
    temp_dir: Arc<TempDir>,
) -> Vec<ChatMessage> {
    raw_vec
        .into_iter()
        .map(|raw| raw_to_chat(raw, temp_dir.clone()))
        .collect()
}

pub fn chat_vec_to_raw_vec(chat_vec: Vec<ChatMessage>) -> Vec<RawChatMessage> {
    chat_vec
        .into_iter()
        .map(RawChatMessage::from)
        .collect()
}
