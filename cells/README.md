# README

## Running Artwork

`cargo run --release`

When ran, the program will save PNG frames to the `frames/` directory up to a maximum of 20_000 frames.

This is a lot of frames! Delete them if you are not using them.

## Converting frames to MP4

```bash
pushd frames
ffmpeg -r 60 -f image2 -s 1280x960 -i %05d.png -vcodec libx264 -crf 25  -pix_fmt yuv420p art.mp4
```