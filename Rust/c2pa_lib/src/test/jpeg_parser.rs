use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use anyhow::{Result};
use c2pa::{HashRange, hash_stream_by_alg};


pub struct JpegParser;

impl JpegParser {
    fn get_hash_range(file: &mut File) -> Vec<HashRange> {
        let mut ranges = Vec::new();
        let mut range_start = 0;

        let mut file_position = 0u64;
        let mut buffer = [0u8; 2];

        // SOI (Start Of Image) の確認
        file.read_exact(&mut buffer).unwrap();
        file_position += 2;

        // フォーマットの確認
        if buffer != [0xFF, 0xD8] {
            // フォーマットがJPEGでない
            return ranges;
        }

        loop {
            // 0xFF を探す
            let mut byte = [0u8; 1];
            if file.read(&mut byte).unwrap() == 0 {
                break; // EOF
            }
            file_position += 1;

            if byte[0] == 0xFF {
                // 次のバイトを読む
                if file.read(&mut byte).unwrap() == 0 {
                    break;
                }
                file_position += 1;

                if byte[0] == 0x00 {
                    continue; // エスケープされた 0xFF (データ中)
                }

                let marker = 0xFF00 | byte[0] as u16;

                // SOS(0xFFDA) や EOI(0xFFD9) は後続データの扱いが特殊
                if marker == 0xFFDA {
                    // スキャンデータ → EOI まで飛ばす
                    // ここではパースを終了
                    break;
                }
                if marker == 0xFFD9 {
                    break; // EOI
                }

                // セグメント長を読む（マーカーにより長さがある）
                let mut len_buf = [0u8; 2];
                file.read_exact(&mut len_buf).unwrap();
                file_position += 2;

                let seg_len = u16::from_be_bytes(len_buf) as u64;
                // 長さにはこの2バイト自身が含まれるので残りをスキップ
                file.seek(SeekFrom::Current((seg_len - 2) as i64)).unwrap();
                file_position += seg_len - 2;

                // C2PAなど
                if marker == 0xFFEB {
                    //ranges.push(HashRange::new(start, pos - start));
                    // println!("marker == 0xEB");
                } else {
                    ranges.push(HashRange::new(range_start, file_position - range_start));
                    // println!(
                    //     "marker == {:#X}, start: {}, len: {}",
                    //     marker,
                    //     range_start,
                    //     file_position - range_start
                    // );
                }
                range_start = file_position;
            }
        }

        // 残りをPush
        ranges.push(HashRange::new(range_start, file_position - range_start));

        ranges
    }

    pub fn get_hash(
        file_path: &str,
        algorithm: &str,
    ) -> Result<Vec<u8>> {
        let mut file = File::open(file_path)?;
        let hash_ranges = Self::get_hash_range(&mut file);
        let hash = hash_stream_by_alg(algorithm, &mut file, Some(hash_ranges), false).unwrap();
        Ok(hash)
    }
}



fn compute_hash(
    file_path: &str,
    algorithm: &str,
    hash_ranges: Option<Vec<HashRange>>,
    is_exclusion: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let hash = hash_stream_by_alg(algorithm, &mut file, hash_ranges, is_exclusion)?;
    Ok(hash)
}
