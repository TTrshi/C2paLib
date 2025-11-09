// ------------------------------------------------------------------------
// mp4 = "0.14"
// openh264 = "0.9.0"
// openh264-sys2 = "0.9.0"
// #yuvutils-rs = "0.2"
// image = "0.25.8"
// thiserror = "1"

use image::{ImageBuffer, Rgb};
use mp4::Mp4Reader;
use openh264::decoder::{DecodedYUV, Decoder, DecoderConfig};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThumbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("MP4 parse error: {0}")]
    Mp4(#[from] mp4::Error),
    #[error("H264 decode error")]
    Decode,
    #[error("RGB conversion error")]
    RgbConversion,
    #[error("Image buffer error")]
    ImageBuffer,
}

/// MP4(H.264) から最初のフレームを JPEG にしてバイト列として返す
pub fn extract_thumbnail<P: AsRef<Path>>(mp4_path: P, jpeg_quality: u8) -> Result<(), ThumbError> {
    // 1) MP4 を開く
    let f = File::open(&mp4_path)?;
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);
    let mut mp4 = Mp4Reader::read_header(reader, size)?;
    let mut track_id = 0;

    println!("[TT_TEST] extract_thumbnail");
    for track in mp4.tracks() {
        //println!("[TT_TEST] track: {}", track.0);
        //println!("[TT_TEST] track_id: {}", track.1.track_id());
        //println!("[TT_TEST] track_type: {:?}", track.1.track_type());
        if let Ok(track_type) = track.1.track_type() {
            if track_type == mp4::TrackType::Video {
                println!("[TT_TEST] track_id: {}", track.1.track_id());

                track_id = track.1.track_id();
            }
        }
    }
    {
        if track_id > 0 {
            // 3) 最初のサンプルを取得
            let sample = mp4.read_sample(track_id, 1)?.ok_or(ThumbError::Decode)?;

            // 4) H.264 デコーダ初期化
            let mut decoder = Decoder::new().map_err(|_| ThumbError::Decode)?;

            println!("[TT_TEST] decoder");

            {
                match decoder.decode(&sample.bytes) {
                    Ok(data_opt) => match data_opt {
                        Some(decoded_yuv) => {
                            // 6) YUV -> RGB8
                            let (w, h) = decoded_yuv.dimensions();
                            let rgb_len = decoded_yuv.rgb8_len();
                            let mut rgb_buf = vec![0u8; rgb_len];
                            decoded_yuv.write_rgb8(&mut rgb_buf);
                        }
                        None => {
                            eprintln!("[TT_TEST] _opt: None");
                        }
                    },
                    Err(e) => {
                        eprintln!("[TT_TEST] Err decode: {}", e);
                    }
                }
            }

            // // 5) デコード（NAL 単位で……この例では一度に全サンプル bytes を使う）
            // let decoded_yuv: DecodedYUV<'_> = decoder
            //     .decode(&sample.bytes)
            //     .map_err(|_| ThumbError::Decode)?
            //     .ok_or(ThumbError::Decode)?;

            // // 6) YUV -> RGB8
            // let (w, h) = decoded_yuv.dimensions();
            // let rgb_len = decoded_yuv.rgb8_len();
            // let mut rgb_buf = vec![0u8; rgb_len];
            // decoded_yuv.write_rgb8(&mut rgb_buf);
        }
    }
    return Ok(());

    // 2) 映像トラックを探す
    let (track_id, track) = mp4
        .tracks()
        .iter()
        .find(|(_, t)| {
            // track_type() is method
            t.track_type().unwrap() == mp4::TrackType::Video
        })
        .ok_or(ThumbError::Decode)?;
    let track_id = *track_id;

    // 3) 最初のサンプルを取得
    let sample = mp4.read_sample(track_id, 1)?.ok_or(ThumbError::Decode)?;

    // 4) H.264 デコーダ初期化
    let mut decoder = Decoder::new().map_err(|_| ThumbError::Decode)?;

    // 5) デコード（NAL 単位で……この例では一度に全サンプル bytes を使う）
    let decoded_yuv: DecodedYUV<'_> = decoder
        .decode(&sample.bytes)
        .map_err(|_| ThumbError::Decode)?
        .ok_or(ThumbError::Decode)?;

    // 6) YUV -> RGB8
    let (w, h) = decoded_yuv.dimensions();
    let rgb_len = decoded_yuv.rgb8_len();
    let mut rgb_buf = vec![0u8; rgb_len];
    decoded_yuv.write_rgb8(&mut rgb_buf);

    // 7) ImageBuffer 化
    let img_buf: ImageBuffer<Rgb<u8>, _> =
        ImageBuffer::from_raw(w as u32, h as u32, rgb_buf).ok_or(ThumbError::ImageBuffer)?;

    // 8) JPEG へエンコード
    // let mut jpeg_bytes = Vec::new();
    // let dyn_img = image::DynamicImage::ImageRgb8(img_buf);
    // let fmt = image::ImageOutputFormat::Jpeg(jpeg_quality.into());
    // dyn_img.write_to(&mut jpeg_bytes, fmt)
    //     .map_err(|_| ThumbError::RgbConversion)?;

    //Ok(jpeg_bytes)
    Ok(())
}
