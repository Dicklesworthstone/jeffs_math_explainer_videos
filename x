#!/bin/sh
# ./x: the corpus driver (plan §3.6; bead W1 .11 grows it into the full
# corpus-aware port of the template's driver). Commands so far:
#
#   ./x list                                  the corpus: slug, package and README title
#   ./x <slug> review storyboard|script|draft   persona review packet (plan §5.2)
#   ./x <slug> facts-check                      every video number against its oracle.py (G1)
#   ./x <slug> storyboard                       G2 checks; writes script.tsv from the storyboard
#   ./x <slug> build                            release build of the video's binary
#   ./x <slug> test                             its tests, clippy -D warnings and fmt --check
#   ./x <slug> lint                             narration script lint and chapter ownership
#   ./x <slug> probe [OUT_DIR]                  collect real scene glyphs and render probe pages
#   ./x <slug> draft [CHAPTER...]               960x540@30 scratch-voice renders with the layout
#                                               lint into explainer_corpus/<slug>/renders/draft/
#   ./x <slug> sheets [CHAPTER...]              contact sheets (4x3) of those drafts, to read
#   ./x <slug> narrate                          the narration into narration/{work,out}: new reads
#                                               only (ftts), every take transcribed (fw), reads that
#                                               finish their last word and an editor that listens to
#                                               what it cuts (tools/narration.py), then the gates
#   ./x <slug> narration-sheets [--takes] [LINE...]   spectrograms of the narration to read as images,
#                                               into narration/sheets/ (tools/narration_sheets.py)
#   ./x <slug> narrated [CHAPTER...]            960x540@30 with the real narration (NARRATION, default
#                                               narration/out) into renders/narrated/, then the cue
#                                               check and cue sheets (renders/narrated/cue_sheets/)
#   ./x <slug> score                            the music: every chapter's timeline at 60 fps from the
#                                               narration, then the score with QA into stems/
#   ./x <slug> scored [CHAPTER...]              960x540@60 with narration and stems, into renders/scored/
#   ./x <slug> final [CHAPTER...]               3840x2160@60, x264 CRF 16 (preset slow), narration and
#                                               stems, into renders/final/
#   ./x <slug> master                           the PCM master of renders/final/ (every chapter final)
#   ./x <slug> captions                         SRT and WebVTT from renders/final/ and the narration
#   ./x <slug> verify                           the delivery gates on the master (verify_video.sh)
#   ./x <slug> watch [scored|narrated|draft]    one file to watch: those renders in chapter order,
#                                               stream-copied to renders/<slug>_<kind>_watch.mp4
#   ./x <slug> thumbnail                        the video's thumbnail, 1280x720 PNG, into renders/
#   ./x <slug> chapters                         where each chapter starts in the final cut (M:SS key),
#                                               from renders/final/'s timelines, for the description
#   ./x remote <slug> COMMAND [ARGS]            sync this tree to REMOTE_HOST:REMOTE_DIR and run
#                                               `./x <slug> COMMAND` there as a systemd unit
#   ./x remote-wait [UNIT] [SECONDS]            wait (default 540 s) for UNIT (default the last
#                                               one started); its exit status, 3 = still running
#   ./x fetch PATH...                           copy files back from REMOTE_DIR, md5-checked; a
#                                               different local file is never overwritten
#
# Cargo runs through whatever `cargo` is on PATH: on a machine with the rch
# shim, builds go to its workers (nothing compiles on the owner's Mac).
# Settings (per machine, gitignored): project.env, see project.env.example.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
usage() { sed -n '2,/^# Cargo /p' "$0" | sed '$d; s/^# \{0,1\}//'; exit 2; }
die() { echo "x: $*" >&2; exit 2; }
[ -f "$here/project.env" ] && . "$here/project.env"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-nightly-2026-08-31}"
SKILLS="${SKILLS:-$HOME/.claude/skills}"
CARGO_FLAGS="-Zgit=shallow-deps -Zgitoxide=fetch"

need_remote() {
  [ -n "${REMOTE_HOST:-}" ] && [ -n "${REMOTE_DIR:-}" ] || die "set REMOTE_HOST and REMOTE_DIR in project.env"
}

# The tracked files and the new ones git does not ignore: never renders,
# narration takes or anything else .gitignore keeps out.
sync_to_remote() {
  need_remote
  (cd "$here" && git ls-files -co --exclude-standard) |
    rsync -a --files-from=- "$here/" "$REMOTE_HOST:$REMOTE_DIR/"
}

case "${1:-}" in
  list)
    [ $# -eq 1 ] || usage
    printf 'slug\tpackage\ttitle\n'
    for entry in "$here"/explainer_corpus/*; do
      [ -f "$entry/Cargo.toml" ] || continue
      package=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$entry/Cargo.toml" | head -n 1)
      title=""
      if [ -f "$entry/README.md" ]; then
        title=$(sed -n 's/^# //p' "$entry/README.md" | head -n 1)
      elif [ -f "$entry/docs/brief.md" ]; then
        title=$(sed -n 's/^# Brief — //p' "$entry/docs/brief.md" | head -n 1)
      fi
      printf '%s\t%s\t%s\n' "${entry##*/}" "$package" "$title"
    done
    exit 0 ;;
  remote)
    shift
    [ $# -ge 2 ] || usage
    sync_to_remote
    unit="x-$(printf '%s-%s' "$1" "$2" | tr -c 'A-Za-z0-9_-' '-' | cut -c1-40)-$(date +%H%M%S)"
    quoted=""
    for a in "$@"; do quoted="$quoted '$(printf '%s' "$a" | sed "s/'/'\\\\''/g")'"; done
    # shellcheck disable=SC2029  # expanded here on purpose: the unit's command line
    ssh "$REMOTE_HOST" "cd '$REMOTE_DIR' && mkdir -p logs && systemd-run --user --unit='$unit' --collect \
      --property=MemoryHigh=infinity --working-directory='$REMOTE_DIR' \
      --setenv=PATH=\$HOME/.rch/shims:\$HOME/.cargo/bin:\$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin \
      /bin/sh -c \"./x $quoted > logs/$unit.log 2>&1; echo \\\"== exit \\\$?\\\" >> logs/$unit.log\""
    mkdir -p "$here/.x"
    echo "$unit" > "$here/.x/remote.last"
    echo "x: started $unit on $REMOTE_HOST (./x remote-wait; log: $REMOTE_DIR/logs/$unit.log)"
    exit 0 ;;
  remote-wait)
    need_remote
    unit="${2:-$(cat "$here/.x/remote.last" 2>/dev/null || true)}"
    [ -n "$unit" ] || die "no unit started yet"
    limit="${3:-540}"
    waited=0
    while ssh "$REMOTE_HOST" "systemctl --user is-active --quiet '$unit'"; do
      [ "$waited" -ge "$limit" ] && { echo "x: $unit still running after ${limit}s"; exit 3; }
      sleep 15; waited=$((waited + 15))
    done
    ssh "$REMOTE_HOST" "tail -n 25 '$REMOTE_DIR/logs/$unit.log'"
    status=$(ssh "$REMOTE_HOST" "tail -n 1 '$REMOTE_DIR/logs/$unit.log'" | sed -n 's/^== exit \([0-9]*\)$/\1/p')
    exit "${status:-1}" ;;
  fetch)
    shift
    need_remote
    [ $# -ge 1 ] || usage
    for p in "$@"; do
      case "$p" in /*|*..*) die "fetch takes a repo-relative path: $p" ;; esac
      tmp="$here/.x/fetch.$$"
      mkdir -p "$here/.x"
      scp -q "$REMOTE_HOST:$REMOTE_DIR/$p" "$tmp" || die "cannot fetch $p"
      want=$(ssh "$REMOTE_HOST" "md5sum '$REMOTE_DIR/$p'" | cut -d' ' -f1)
      got=$( (md5sum "$tmp" 2>/dev/null || md5 -r "$tmp") | cut -d' ' -f1)
      [ "$want" = "$got" ] || { mv "$tmp" "$tmp.bad"; die "md5 mismatch for $p (kept as $tmp.bad)"; }
      if [ -e "$here/$p" ] && ! cmp -s "$tmp" "$here/$p"; then
        mv "$tmp" "$here/$p.fetched"
        echo "x: $p differs locally; the fetched copy is $p.fetched"
      else
        mkdir -p "$(dirname "$here/$p")"
        mv "$tmp" "$here/$p"
        echo "x: fetched $p ($got)"
      fi
    done
    exit 0 ;;
esac

[ $# -ge 2 ] || usage
slug=$1; cmd=$2; shift 2
case "$slug" in
  */*|.*|'') die "bad slug '$slug'" ;;
esac
crate="$here/explainer_corpus/$slug"
[ -d "$crate" ] || die "no explainer at explainer_corpus/$slug"
pkg=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$crate/Cargo.toml" | head -n 1)
bin=$(awk '/^\[\[bin\]\]/ { f = 1 } f && /^name/ { gsub(/.*= *"|"/, ""); print; exit }' "$crate/Cargo.toml")
binary="${CARGO_TARGET_DIR:-$here/target}/release/$bin"
drafts="$crate/renders/draft"
narration="${NARRATION:-$crate/narration/out}"
finals="$crate/renders/final"

# The release binary, built first.
build_binary() {
  cd "$here"
  # shellcheck disable=SC2086  # CARGO_FLAGS is two flags
  cargo build --release -p "$pkg" $CARGO_FLAGS
  [ -x "$binary" ] || die "no binary at $binary after the build"
}
need_narration() {
  [ -d "$narration" ] || die "no narration at $narration (set NARRATION, or run the narration pipeline)"
}
# The chapter keys as one comma-separated list: the arguments, or all of them.
chapter_list() {
  if [ $# -gt 0 ]; then printf '%s' "$*" | tr ' ' ','; else "$binary" chapters | paste -sd, -; fi
}

case "$cmd" in
  lint)
    [ $# -eq 0 ] || usage
    lint="$SKILLS/math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/lint_script.py"
    [ -f "$lint" ] || die "no lint_script.py under SKILLS (set SKILLS in project.env)"
    build_binary
    exec python3 -I "$lint" "$crate/script.tsv" --bin "$binary" ;;
  probe)
    [ $# -le 1 ] || usage
    build_binary
    out="${1:-$crate/renders/glyphs_probe}"
    "$binary" glyphs "$out" --res 1920x1080
    echo "x: READ every PNG under $out/pages; sources: $out/registry.tsv" ;;
  review)
    [ $# -eq 1 ] || usage
    exec python3 -I "$here/tools/review.py" "$slug" "$1" ;;
  facts-check)
    [ $# -eq 0 ] || usage
    exec python3 -I "$here/tools/facts_check.py" "$slug" ;;
  storyboard)
    [ $# -eq 0 ] || usage
    exec python3 -I "$here/tools/storyboard.py" "$slug" ;;
  build)
    [ $# -eq 0 ] || usage
    cd "$here"
    # shellcheck disable=SC2086  # CARGO_FLAGS is two flags
    exec cargo build --release -p "$pkg" $CARGO_FLAGS ;;
  test)
    [ $# -eq 0 ] || usage
    cd "$here"
    # shellcheck disable=SC2086
    cargo test --release -p "$pkg" $CARGO_FLAGS
    # shellcheck disable=SC2086
    cargo clippy --release -p "$pkg" --all-targets $CARGO_FLAGS -- -D warnings
    exec cargo fmt -p "$pkg" --check ;;
  draft)
    build_binary
    mkdir -p "$drafts"
    if [ $# -eq 0 ]; then set -- all; fi
    exec "$binary" render "$@" --res 960x540 --fps 30 --scratch-voice --lint --out "$drafts" ;;
  narrate)
    [ $# -eq 0 ] || usage
    # The voice is the owner's choice per video (AGENTS.md). The headroom is the voice's own: the
    # limiter's ceiling over speech loudness, the lowest at which it turns down more than 1 dB of
    # about 1% of samples (the narration skill's criterion, which the chooser enforces). matt and
    # michael peak higher over their loudness than leo and robert do.
    case "$slug" in
      capture_recapture) voice=matt headroom=17 ;;
      raptorq_gf256) voice=michael headroom=18 ;;
      bakery_algorithm) voice=michael headroom=18 ;;
      hoeffding_d) voice=robert headroom=15 ;;
      *) die "no voice chosen for $slug (add it to ./x narrate)" ;;
    esac
    N="$SKILLS/natural-sounding-narration-with-frankentts/scripts"
    # One ftts build for every read. A take is keyed by its text and voice, not by the engine that
    # made it, so another build first on someone's PATH mixes its reads in unseen (262 of the fish
    # video's takes once came from an old one). FTTS in project.env names the binary to use.
    FTTS=$(command -v "${FTTS:-ftts}") || die "ftts not found: set FTTS in project.env or put it on PATH"
    # The batch consumes FTTS as a filename after we cd into the crate.
    FTTS="$(cd "$(dirname "$FTTS")" && pwd)/$(basename "$FTTS")"
    ftts_is=$("$FTTS" --version 2>&1 | head -n 1) || true
    [ "$ftts_is" = "ftts ${FTTS_VERSION:-0.1.11}" ] ||
      die "$FTTS is '$ftts_is', not ftts ${FTTS_VERSION:-0.1.11}: set FTTS in project.env to that build"
    export FTTS
    work=narration/work
    out=narration/out
    py() { uv run --quiet --no-project --with numpy python3 -I "$@"; }
    cd "$crate"
    mkdir -p "$work"
    py "$here/tools/narration.py" "$N" plan script.tsv "$work" --voice "$voice" --model "${FTTS_MODEL:-}"
    python3 -I "$N/tts_batch.py" "$work/takes.tsv" "$work/takes" --voice "$voice" --workers "${WORKERS:-8}"
    python3 -I "$N/asr_batch.py" "$work/takes" "$work/asr.json"
    py "$here/tools/narration.py" "$N" choose script.tsv "$work" "$out" --limit-headroom "$headroom"
    python3 -I "$N/asr_batch.py" "$out" "$out/lines_asr.json"
    # every gate runs, so one build shows everything that is wrong
    status=0
    py "$N/qa_lines.py" "$out/spoken.tsv" "$out/lines_asr.json" || status=1
    py "$N/validate_narration.py" "$out" || status=1
    py "$here/tools/narration.py" "$N" edges script.tsv "$out" || status=1
    exit $status ;;
  narration-sheets)
    cd "$crate"
    exec uv run --quiet --no-project --with numpy --with matplotlib python3 -I "$here/tools/narration_sheets.py" \
      "$SKILLS/natural-sounding-narration-with-frankentts/scripts" narration narration/sheets "$@" ;;
  narrated)
    need_narration
    build_binary
    out="$crate/renders/narrated"
    mkdir -p "$out"
    keys=$(chapter_list "$@")
    if [ $# -eq 0 ]; then set -- all; fi
    "$binary" render "$@" --res 960x540 --fps 30 --narration "$narration" --lint --out "$out"
    exec python3 -I "$SKILLS/math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/cue_sheets.py" \
      "$out" --script "$crate/script.tsv" --chapters "$keys" --sheets "$out/cue_sheets" ;;
  score)
    [ $# -eq 0 ] || usage
    need_narration
    build_binary
    # pass 1: every chapter's timeline at the finals' 60 fps (64x36, no picture worth keeping)
    "$binary" render all --timeline-only --fps 60 --narration "$narration" --out "$crate/renders/timeline"
    # pass 2: the whole video's music from those timelines, sliced per chapter, with QA
    exec "$binary" score --timelines "$crate/renders/timeline" --out "$crate" ;;
  scored)
    need_narration
    [ -d "$crate/stems" ] || die "no stems in $crate/stems (run ./x $slug score)"
    build_binary
    out="$crate/renders/scored"
    mkdir -p "$out"
    if [ $# -eq 0 ]; then set -- all; fi
    exec "$binary" render "$@" --res 960x540 --fps 60 --narration "$narration" --score "$crate/stems" --out "$out" ;;
  final)
    need_narration
    [ -d "$crate/stems" ] || die "no stems in $crate/stems (run ./x $slug score)"
    build_binary
    mkdir -p "$finals"
    if [ $# -eq 0 ]; then set -- all; fi
    exec "$binary" render "$@" --res 3840x2160 --fps 60 --narration "$narration" --score "$crate/stems" \
      --crf 16 --preset slow --out "$finals" ;;
  master)
    [ $# -eq 0 ] || usage
    build_binary
    exec "$binary" master --out "$finals" ;;
  captions)
    [ $# -eq 0 ] || usage
    need_narration
    build_binary
    exec "$binary" captions --narration "$narration" --out "$finals" ;;
  verify)
    [ $# -eq 0 ] || usage
    check="$SKILLS/franken-manim-mega-skill/scripts/verify_video.sh"
    [ -x "$check" ] || die "no verify_video.sh under $SKILLS (set SKILLS in project.env)"
    master_mp4="$finals/${bin}_master.mp4"
    [ -f "$master_mp4" ] || master_mp4=$(ls "$finals"/*_master.mp4 2>/dev/null | head -n 1)
    [ -f "$master_mp4" ] || die "no master in $finals (run ./x $slug master)"
    export EXPECT_RES=3840x2160 EXPECT_FPS=60 EXPECT_BT709=1 EXPECT_LUFS=-16 EXPECT_TP_MAX=-1.0
    exec "$check" "$master_mp4" ;;
  thumbnail)
    [ $# -eq 0 ] || usage
    build_binary
    exec "$binary" thumbnail --res 1280x720 --out "$crate/renders" ;;
  chapters)
    # A start is rounded down to the second: a link must never land after the chapter began.
    [ $# -eq 0 ] || usage
    ls "$finals"/*.timeline.tsv >/dev/null 2>&1 || die "no timelines in $finals (run ./x $slug final)"
    for f in "$finals"/*.timeline.tsv; do
      printf '%s\t%s\n' "$(basename "$f" .timeline.tsv)" "$(awk -F'\t' '$1 == "fps" { fps = $2 } $1 == "frames" { n = $2 } END { print n / fps }' "$f")"
    done | awk -F'\t' '{ s = int(t); printf "%d:%02d %s\n", s / 60, s % 60, $1; t += $2 }'
    exit 0 ;;
  watch)
    # every chapter's draft in chapter order, stream-copied into one file to watch
    [ $# -le 1 ] || usage
    kind="${1:-scored}"
    case "$kind" in narrated|scored|draft) ;; *) usage ;; esac
    src="$crate/renders/$kind"
    build_binary
    list="$src/watch.concat.txt"
    : > "$list"
    for k in $("$binary" chapters); do
      [ -f "$src/$k.mp4" ] || die "no $k.mp4 in $src (render it: ./x $slug $kind $k)"
      printf "file '%s'\n" "$k.mp4" >> "$list"
    done
    out="$crate/renders/${slug}_${kind}_watch.mp4"
    if [ -e "$out" ]; then
      mkdir -p "$crate/renders/previous"
      mv "$out" "$crate/renders/previous/${slug}_${kind}_watch.$(date +%Y%m%d-%H%M%S).mp4"
    fi
    ffmpeg -hide_banner -loglevel error -f concat -safe 0 -i "$list" -c copy -movflags +faststart "$out"
    echo "x: $out ($(ffprobe -v error -show_entries format=duration -of csv=p=0 "$out" | cut -d. -f1) s)" ;;
  sheets)
    sheet="$SKILLS/franken-manim-mega-skill/scripts/contact_sheet.sh"
    [ -x "$sheet" ] || die "no contact_sheet.sh under $SKILLS (set SKILLS in project.env)"
    if [ $# -eq 0 ]; then
      set -- $(cd "$drafts" && ls ./*.mp4 2>/dev/null | sed 's|^\./||; s|\.mp4$||')
    fi
    for c in "$@"; do
      [ -f "$drafts/$c.mp4" ] || die "no draft $c.mp4 in $drafts"
      out="$drafts/$c.sheet-$(date +%H%M%S).png"
      "$sheet" "$drafts/$c.mp4" "$out" 4x3 && echo "x: $out"
    done ;;
  *) usage ;;
esac
