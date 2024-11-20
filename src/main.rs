use clap::Parser;
use ffmpeg_next as ffmpeg;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};
use std::io::stdout;
use termion::{clear, cursor};

const ASCII_CHARS: &[u8] = b"@%#*+=-:. ";

fn frame_to_ascii(image: &DynamicImage, target_width: u32) -> String {
   
    let (original_width, original_height) = image.dimensions();
    let aspect_ratio = original_height as f32 / original_width as f32;
    let target_height = (target_width as f32 * aspect_ratio) as u32;
    let char_aspect_ratio = 0.6; // adjust to ye liking
    let adjusted_height = (target_height as f32 * char_aspect_ratio) as u32;
    
    let resized = image.resize_exact(
        target_width,
        adjusted_height,
        image::imageops::FilterType::Nearest,
    );

    let (width, height) = resized.dimensions();
    let mut ascii_frame = String::with_capacity((width * height + height) as usize);

    for y in 0..height {
        for x in 0..width {
            let pixel = resized.get_pixel(x, y);
            let brightness = (u32::from(pixel[0]) + u32::from(pixel[1]) + u32::from(pixel[2])) / 3;
            let char_idx = (brightness * (ASCII_CHARS.len() as u32 - 1)) / 255;
            ascii_frame.push(ASCII_CHARS[char_idx as usize] as char);
        }
        ascii_frame.push('\n');
    }

    ascii_frame
}


fn convert_video_to_ascii(
    video_path: &str,
    output_path: &str,
    target_width: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg::init()?;
    let mut ictx = ffmpeg::format::input(&video_path)?;

    let input = ictx
        
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)?;

    let video_stream_index = input.index();
    let context_decoder =

        ffmpeg::codec::context::Context::from_parameters(input.parameters())?;

    let mut decoder = context_decoder.decoder().video()?;
    let mut scaler = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )?;

    let mut frame = ffmpeg::util::frame::Video::empty();
    let mut rgb_frame = ffmpeg::util::frame::Video::empty();
    let mut output_file = File::create(output_path)?;

    for (stream, packet) in ictx.packets() {
        
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            while decoder.receive_frame(&mut frame).is_ok() {
                
                scaler.run(&frame, &mut rgb_frame)?;
                let dynamic_image = ffmpeg_frame_to_image(&rgb_frame)?;
                let ascii_frame = frame_to_ascii(&dynamic_image, target_width);
            
                writeln!(output_file, "FRAME_START")?;
                writeln!(output_file, "{}", ascii_frame)?;
            }
        }
    }

    decoder.send_eof()?;
    while decoder.receive_frame(&mut frame).is_ok() {
        
        scaler.run(&frame, &mut rgb_frame)?;
        let dynamic_image = ffmpeg_frame_to_image(&rgb_frame)?;
        let ascii_frame = frame_to_ascii(&dynamic_image, target_width);
        
        writeln!(output_file, "FRAME_START")?;
        writeln!(output_file, "{}", ascii_frame)?;
    }

    Ok(())
}

fn ffmpeg_frame_to_image(frame: &ffmpeg::util::frame::Video) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    
    let width = frame.width();
    let height = frame.height();
    let data = frame.data(0);
    let stride = frame.stride(0);
    let mut img_buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);

    for (y, row) in data
        .chunks(stride)
        .take(height as usize)
        .enumerate()
    {
        for x in 0..width as usize {
            let idx = x * 3;
            if idx + 2 < row.len() {
                let pixel = Rgb([row[idx], row[idx + 1], row[idx + 2]]);
                img_buffer.put_pixel(x as u32, y as u32, pixel);
            }
        }
    }

    Ok(DynamicImage::ImageRgb8(img_buffer))
}

fn play_ascii_video(file_path: &str, fps: u32, loop_playback: bool) -> Result<(), Box<dyn std::error::Error>> {
    
    let frame_duration = Duration::from_secs_f32(1.0 / fps as f32);
    let mut stdout = stdout();
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut frames: Vec<String> = Vec::new();
    let mut current_frame = String::new();
    
    for line in reader.lines() {
        let line = line?;
        if line == "FRAME_START" {
            if !current_frame.is_empty() {
                frames.push(current_frame.clone());
                current_frame.clear();
            }
        } else {
            current_frame.push_str(&line);
            current_frame.push('\n');
        }
    }
    if !current_frame.is_empty() {
        frames.push(current_frame);
    }

    loop {
        for frame in &frames {
            let start_time = Instant::now();
            write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;
            stdout.flush()?;
            print!("{}", frame);
            stdout.flush()?;
            let elapsed_time = start_time.elapsed();
            if elapsed_time < frame_duration {
                std::thread::sleep(frame_duration - elapsed_time);
            }
        }
        if !loop_playback {
            break;
        }
    }

    Ok(())
}


#[derive(Parser, Debug)]
#[command(
    name = "ASCIIRender",
    version = "1.0",
    author = "Steffe",
    about = "converts video to ASCII and plays it in the terminal"
)]



struct Args {
    #[arg(long)]
    convert: Option<String>,

    #[arg(long)]
    play: Option<String>,

    #[arg(long, default_value_t = 24)]
    fps: u32,

    #[arg(long, default_value_t = 80)]
    width: u32,

    #[arg(long, default_value = "output.txt")]
    output: String,

    #[arg(long)]
    loop_playback: bool,
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();


    if let Some(video_path) = args.convert {
        let output_path = &args.output;
        let target_width = args.width;

        convert_video_to_ascii(
            &video_path,
            output_path,
            target_width,
        )?;
        println!("Video converted and saved to {}", output_path);
    }


    if let Some(file_path) = args.play {

        let fps = args.fps;
        let loop_playback = args.loop_playback;
        
        play_ascii_video(
            &file_path, 
            fps, 
            loop_playback,
        )?;
    }

    Ok(())
} 