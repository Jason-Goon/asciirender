# ascii video converter

a minimal tool to convert videos into ascii art and play them in the terminal.

convert a video to ascii and save frames to a file:
```bash
cargo run --release -- --convert <video_path> --output <output_file> --width <target_width>
# example 
cargo run --release -- --convert video.mp4 --output ascii_video.txt --width 1300
```
play an ascii video from a file:
```bash
cargo run --release -- --play <output_file> --fps <frames_per_second> --loop-playback
#example 
cargo run --release --play ascii_video.txt --fps 30 --loop-playback
```
### arguments

- `--convert <video_path>`: specify the path to the input video to convert.
- `--play <output_file>`: specify the file containing ascii frames to play.
- `--output <output_file>`: path to save the converted ascii frames (default: `output.txt`).
- `--fps <frames_per_second>`: playback speed in fps (default: `24`).
- `--width <target_width>`: target width (default: `80`).
- `--loop-playback`: (default: `false`).

## dependencies

this project uses:
- `clap`
- `ffmpeg-next`
- `image`
- `termion`

make sure `ffmpeg` is installed on your system for video decoding.

## example

convert a video:

cargo run -- --convert example.mp4 --output output.txt --width 100


play the converted ascii video:

cargo run -- --play output.txt --fps 30 --loop-playback
