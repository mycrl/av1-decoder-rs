use bytes::{Buf, BytesMut};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncSeekExt},
};
use toy_rav1d::{Buffer, ObuHeader};

#[tokio::main]
async fn main() {
    let mut file = OpenOptions::new()
        .read(true)
        .open("./big_buck_bunny_720p_h264.ivf")
        .await
        .unwrap();

    // ivf header
    file.seek(std::io::SeekFrom::Start(32)).await.unwrap();

    let mut buf = BytesMut::new();
    let mut frame_size = 0;
    loop {
        let size = file.read_buf(&mut buf).await.unwrap();
        if size == 0 {
            break;
        }

        loop {
            if buf.len() < 12 {
                break;
            }

            if frame_size > 0 {
                if frame_size <= buf.len() {
                    let frame_buf = buf.split_to(frame_size);
                    println!("{:#?}", frame_size);
                    frame_size = 0;

                    let mut buffer = Buffer::new(&frame_buf[..]);
                    let header = ObuHeader::decode(&mut buffer).unwrap();
                    println!("{:#?}", header);
                } else {
                    break;
                }
            } else {
                // frame size, pts
                frame_size = buf.get_u32_le() as usize;
                buf.advance(8);
            }
        }
    }
}
