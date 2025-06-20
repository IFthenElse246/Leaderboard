use std::{fs::File, io::Read};

pub fn read_file(file: &File) -> Result<String, std::io::Error> {
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    if !metadata.is_file() {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Is not a file!",
        ))
    } else if file_size > isize::MAX as u64 {
        Err(std::io::Error::new(
            std::io::ErrorKind::FileTooLarge,
            "File is too large!",
        ))
    } else if file_size == 0 {
        Ok(String::new())
    } else if file_size <= 16 * 1024 {
        let mut buffer = String::new();
        #[allow(suspicious_double_ref_op)]
        file.clone().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        let map = unsafe {
            memmap2::MmapOptions::new()
                .len(file_size as usize)
                .map(file)?
        };

        Ok(String::from_utf8_lossy(&*map).to_string())
    }
}
