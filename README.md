# Learn Korean

Leptos (WASM) karaoke for daily Korean-phrase shorts. Same shell as
[Euclid’s Elements](https://github.com/swimming-bookstore/euclids-elements):
read the lesson, or Play / `?record=1` for a 9:16 YouTube Short.

Phrase notes follow [퇴사할게여](https://github.com/whs-dot-hk/i-m-gonna-toesa):
the line, then **vocabulary**, then **grammar**. Series so far: `#/1` 퇴사할게여
(episodes 1–31), `#/2` 참교육 (00, then 01, 02, 03).

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
Hash `#/2` and `#/2/0` are 참교육 00. `#/2/1` is 참교육 01. `#/2/2` is 참교육 02. `#/2/3` is 참교육 03. `#/2/12` through `#/2/15` are 참교육 12–15.

YouTube: upload `docs/toesa-dayNN.mp4` or `docs/chamgyoyuk-dayNN.mp4` as a Short.
Title idea: `1 · 퇴사할게여 | Learn Korean`.

## GitHub Pages

`.github/workflows/pages.yml` builds with Trunk and deploys on push to `master`.

Repo → **Settings** → **Pages** → **Build and deployment** → Source: **GitHub Actions**.

First deploy URL is `https://<user>.github.io/<repo>/`. Hash routes (`#/1/12`, `#/2/1`) work as-is.
