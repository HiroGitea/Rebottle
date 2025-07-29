## Remux Dolby Vision Profile 5 in MKV to MP4 with QuickTime `dvh1` Support

Quicktime supports `dvh1` ONLY, which keeps DV metadata in the specific box separately in mp4 container. Apple has a complicated reason to use this extraordinary format for HLG compatibility.

**Tools needed:**

- **MKVToolNix** for extracting and manipulating `.mkv` files

- **`mp4muxer`** Official tool from Dolby for convert, mux and remux Dolby Vision and Dolby Digital Plus
- **MediaInfo** handy tool to analyze media files, useful when checking Dolby Vision Profile and Codec ID
- **FFmpeg** for general remuxing and stream extraction.(Not recommend for remuxing Dolby Vision)
- **GPAC (MP4Box)** for better Dolby Vision compatibility in MP4

**Remuxing**

Extract video stream from mkv with MKVToolNix, This ensures Dolby Vision metadata get transferred correctly.

```shell
mkvextract tracks /path/to/source.mkv 0:DV.hevc
```

Extract multichannel audio and subtitles with FFmpeg

```shell
ffmpeg -i /path/to/source.mkv -map 0:a:0 audio.ec3 -map 0:s:0 subs.srt
```

Remux video and audio and convert video stream with `--dvh1flag` to enable QuickTime compatibility.

```shell
mp4muxer -o dvh1.mp4 -i DV.hevc -i audio.ec3 --dv-profile 5 --dvh1flag 0
# Explicitly set the framerate if video stream is out of sync
mp4muxer_mac -o dvh1.mp4 -i DV.hevc --input-video-frame-rate 24000/1001 -i audio.ec3 --dv-profile 5 --dvh1flag 0
```

The output mp4 file should have Dolby Vision and Dolby Atmos support in QuickTime.



**Subtitles (Optional)**

Since `mp4` doesn't support SRT subtitles and MP4Box doesn't support convert SRT subtitles to MP4-compatible `mov_text` directly, A workaround is to use FFmpeg to convert SRT subtitles to  `mov_text` and remux into a carrier mp4 file, then mux the carrier mp4 file and the remuxed DV file together with `MP4Box`.

```shell
ffmpeg -i subs.srt -c:s mov_text subs.mp4 && \
MP4Box -add dvh1.mp4 -add subs.mp4 -new output.mp4
```



`mp4muxer` Supported frame rate:

| **Standard Name** | **Fraction** | **Decimal Equivalent**       |
| ----------------- | ------------ | ---------------------------- |
| Film (NTSC)       | 24000/1001   | 23.976                       |
| Film (PAL)        | 24           | 24.000                       |
| TV (NTSC)         | 30000/1001   | 29.970                       |
| TV (PAL)          | 25           | 25.000                       |
| High Frame        | 60           | 60.000                       |
| NTSC HFR          | 60000/1001   | 59.940                       |
| Custom            | any ratio    | e.g., 120000/1001 or 1000/41 |

