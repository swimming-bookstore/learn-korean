# Learn Korean

Leptos (WASM) karaoke for daily Korean-phrase shorts. Same shell as
[Euclid’s Elements](https://github.com/swimming-bookstore/euclids-elements):
read the lesson, or Play / `?record=1` for a 9:16 YouTube Short.

Phrase notes follow [퇴사할게여](https://github.com/whs-dot-hk/i-m-gonna-toesa):
the line, then **vocabulary**, then **grammar**. Series so far: `#/1` 퇴사할게여
(episodes 1–31), `#/2` 참교육 (00, then 01, 02, 03), `#/3` 막다른 일자리
(Dead-End Job trailer, episodes 00–28), `#/4` 오징어 게임 (official trailer,
episodes 00–18).

```bash
trunk serve
```

http://127.0.0.1:8081/

Record a short (1080×1920):

```
http://127.0.0.1:8081/?record=1&autoplay=1#/1/1
```

```bash
trunk build --release
python3 scripts/record-short.py --all
```

## Karaoke

Two modes, like Euclid:

- **Read** — sentence, meaning, vocabulary, grammar.
- **Record** (`Play` or `?record=1`) — 9:16 short. Sentence stays on the
  plate; the vocab list stays on screen and gold walks one row, then grammar.

A new day is a `Lesson` in `src/content/`. Hash `#/1/12` is 퇴사할게여 episode 12.
Hash `#/2` and `#/2/0` are 참교육 00. `#/2/1` is 참교육 01. `#/2/2` is 참교육 02. `#/2/3` is 참교육 03. `#/2/6` is 참교육 06. `#/2/12` through `#/2/15` are 참교육 12–15. `#/2/18` is 참교육 18, the last episode.
Hash `#/3/2` is 막다른 일자리 02, 거미인력. Spider plus manpower. The company name is Spider Staffing.

YouTube: upload `docs/toesa-dayNN.mp4` or `docs/chamgyoyuk-dayNN.mp4` as a Short.
Title idea: `1 · 퇴사할게여 | Learn Korean`.

The song video is a 16:9 lesson, picture on the left and remarks on the right,
same shape as the Dead-End Job reaction. Source is the official MV. The cut is
the quiet after the line, the same rule as the trailer. Sung picture and sung
audio stop on that frame. The explanation plays only while that frame is held,
then both resume together. A pause inside the next line is the jump.
After the last note the picture and the song play out to the end of the MV.
The card stays blank for that stretch. Stopping at the note cuts the outro.

```bash
cargo run --release --manifest-path video/Cargo.toml
```

That writes `docs/toesa.mp4`. `--draft` renders the first two lines only.

YouTube thumbnail is `docs/youtube-toesa.png`, 1280×720, the same size as
the Euclid cards. It is the frame at 2:19: the pause on 그래 이팔청춘, the
line lit, not a black cut.

The Dead-End Job trailer is the same shape. On the pause the voice reads the
whole line, then vocabulary, then grammar, the same order as 퇴사할게여.
The notes are a Korean parse, the same split as 참교육: a content stem in
vocabulary, in dictionary form (a verb ends in 다), then that stem plus its
particle or ending in grammar. 의 is "of", not a slash and not "'s".
The cut is the quiet after the line. Trailer picture and trailer audio stop
there, and the explanation plays only while that frame is held.
Line 01 ends on 것 같지, which is quieter than 망한. The pause is after 같지,
not on 망한, or those two words are cut off.
A short pause still has to emit every frame. If those frames keep the source
clock, they are under one tick, the hold is dropped, and the notes are gone.
Speech is recognized in windows shorter than 20 seconds; one long pass
garbles everything after that.

```bash
cargo run --release --manifest-path video/Cargo.toml --bin dead-end-job
```

That writes `docs/dead-end-job.mp4`. `--draft` renders the first two lines only.

The Squid Game trailer is the same shape. Source is the official Netflix
trailer. The cut is the quiet after the line. A long recognition pass smears
the opening across the music and hears 빚을 치고; the lines come from windows
under 20 seconds, and the first pause is after 지고.

```bash
cargo run --release --manifest-path video/Cargo.toml --bin squid-game
```

That writes `docs/squid-game.mp4`. `--draft` renders the first two lines only.

## GitHub Pages

`.github/workflows/pages.yml` builds with Trunk and deploys on push to `master`.

Repo → **Settings** → **Pages** → **Build and deployment** → Source: **GitHub Actions**.

First deploy URL is `https://<user>.github.io/<repo>/`. Hash routes (`#/1/12`, `#/2/1`) work as-is.
