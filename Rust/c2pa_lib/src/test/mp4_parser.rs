use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use anyhow::{Context, Result, anyhow};
use mp4::{BoxHeader, HEADER_SIZE};
use c2pa::{hash_stream_by_alg, HashRange};


pub struct Mp4Parser;

impl Mp4Parser {
    fn parse_boxes(file: &mut File, read_position: u64, read_size: u64) -> Vec<HashRange> {
        let mut ranges = Vec::new();

        let mut file_position = read_position;

        while file_position < read_position + read_size {
            let header_position = file.stream_position().unwrap();
            let header = match BoxHeader::read(file) {
                Ok(h) => h,
                Err(_) => break,    // EOF
            };

            let box_size = header.size;
            let box_type = header.name;

            if box_size < HEADER_SIZE {
                // 読み取りサイズが小さい場合はbreak
                break;
            }

            let content_start = file_position + HEADER_SIZE;
            let content_size = box_size - HEADER_SIZE;

            // moov/trak/mdia/minf/stbl のようなコンテナは再帰的に解析
            if box_type == mp4::BoxType::MoovBox
                // || header_type == mp4::BoxType::TrakBox
                // || header_type == mp4::BoxType::MdiaBox
                // || header_type == mp4::BoxType::MinfBox
                // || header_type == mp4::BoxType::StblBox
            {
                let result = Self::parse_boxes(file, content_start, content_size);
                ranges.extend(result);

                file.seek(SeekFrom::Start(header_position)).unwrap();
                file.seek(SeekFrom::Current(box_size as i64)).unwrap();
            }
            else {

                // ハッシュ対象
                if box_type == mp4::BoxType::FtypBox
                    || box_type == mp4::BoxType::MdatBox
                    // || header_type == mp4::BoxType::TrakBox
                    // || header_type == mp4::BoxType::MdiaBox
                    // || header_type == mp4::BoxType::MinfBox
                    // || header_type == mp4::BoxType::StcoBox
                {
                    // Push
                    ranges.push(HashRange::new(content_start, content_size));
                }
                file.seek(SeekFrom::Current(content_size as i64)).unwrap();
            }

            file_position += box_size;
        }

        return ranges;
    }

    fn get_hash_range(file: &mut File) -> Vec<HashRange> {
        let file_size = file.metadata().unwrap().len();
        return Self::parse_boxes(file, 0, file_size);
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






                // let payload = size - header_len;

                // let sample_len = 8192; /*8KB*/
                // //let mut hasher = Sha256::new();
                // let points = [
                //     0,                                               // 先頭
                //     (payload.saturating_sub(sample_len as u64)) / 2, // 中央
                //     payload.saturating_sub(sample_len as u64),       // 末尾
                // ];
                // let mut buf = vec![0u8; sample_len];
                // for &p in &points {
                //     //println!("p: {}, p+: {}", p , p + 8192);
                //     if p < payload {
                //         f.seek(SeekFrom::Start(box_start + p)).unwrap();
                //         let read_len = sample_len.min((payload - p) as usize);
                //         f.read_exact(&mut buf[..read_len]).unwrap();
                //         hasher.update(&buf[..read_len]);

                //         let cur_pos = f.stream_position().unwrap();
                //         //println!("Current position: {}", cur_pos);
                //     }
                // }


    // fn main() -> Result<()> {
    // let r_0 = get_hash_range_mp4("data/mov_hts-samp009.mp4");
    // println!("range: {:#?}", r_0);
    // let r_1 = get_hash_range_mp4("out/output.mp4");
    // println!("range: {:#?}", r_1);
    //     {


    //         let hash_ranges_0 = Some(vec![
    //             //HashRange::new(29373, 10),
    //             //r_0.clone()[0].clone(),
    //             r_0.clone()[1].clone(),
    //             //r_0.clone()[2].clone()

    //         ]);
    //         let hash_ranges_1 = Some(vec![
    //             //HashRange::new(29373, 10),
    //             //r_1.clone()[0].clone(),
    //             r_1.clone()[1].clone(),
    //             //r_1.clone()[2].clone(),
    //         ]);

    //         // Before size: 61720
    //         // After size: 128826
    //         let f = File::open("data/mov_hts-samp009.mp4").unwrap();
    //         let file_size = f.metadata()?.len();
    //         println!("Before size: {:?}", file_size);
    //         let f = File::open("out/output.mp4").unwrap();
    //         let file_size = f.metadata()?.len();
    //         println!("After size: {:?}", file_size);

    //         let before_hash = compute_hash("data/mov_hts-samp009.mp4", 
    //         "sha256", 
    //         Some(r_0), 
    //         // hash_ranges_0.clone(),  
    //         false).unwrap();
    //         let after_hash = compute_hash("out/output.mp4", 
    //         "sha256", 
    //         Some(r_1), 
    //         // hash_ranges_1.clone(), 
    //         false).unwrap();

    //         println!("Before vec: {:?}", before_hash);
    //         println!("After  vec: {:?}", after_hash);
    //         println!("Before hash: {}", encode(&before_hash));
    //         println!("After  hash: {}", encode(&after_hash));

    //         if before_hash == after_hash {
    //             println!("The files are identical.");
    //         } else {
    //             println!("The files differ.");
    //         }
    //     }