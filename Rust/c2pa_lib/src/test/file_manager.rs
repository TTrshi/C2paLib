use std::fs;
use std::io;
use std::path::Path;

// pub struct FileManager {
//     queue: SignerExecuter,
// }

pub struct FileManager;

impl FileManager {
    pub fn list_files_with_extension<P: AsRef<Path>>(
        &self,
        dir: P,
        extension: &str,
    ) -> io::Result<Vec<String>> {
        let mut result = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // ファイルのみを対象にする
            if path.is_file() {
                // 拡張子を確認
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        if let Some(name) = path.to_str() {
                            result.push(name.to_string());
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}