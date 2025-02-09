use std::{ffi::{OsStr, OsString}, fs::File, io::{Read, Write}, path::PathBuf};

use tempfile::Builder;

use super::message::{ChatMessage, RawChatMessage};

pub fn file_to_byte_vec(file_path: PathBuf) -> Result<(Vec<u8>, OsString, OsString), std::io::Error> {
    
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

pub fn byte_vec_to_file(file_name: OsString, file_extension: OsString, data: Vec<u8>) -> Result<PathBuf, std::io::Error> {

    // creates a temp file inside the temp_dir (uses the std implementation of temp_dir)
    let named_tempfile = Builder::new().prefix(&file_name).suffix(&file_extension).rand_bytes(3).tempfile()?;

    // writes the data to the temp file
    named_tempfile.as_file().write_all(&data)?;

    Ok(named_tempfile.path().to_path_buf())
}

impl From<RawChatMessage> for ChatMessage {
    fn from(value: RawChatMessage) -> Self {
        match value {
            RawChatMessage::TextMessage { from, to, text } => ChatMessage::TextMessage { from, to, text },
            RawChatMessage::FileMessage { from, to, file, file_name, extension } => ChatMessage::FileMessage { from, to, file_path: byte_vec_to_file(file_name, extension, file).unwrap() },
        }
    }
}

pub fn raw_vec_to_chat_vec(raw_vec: Vec<RawChatMessage>) -> Vec<ChatMessage> {
    raw_vec.into_iter().map(|raw| ChatMessage::from(raw)).collect()
}