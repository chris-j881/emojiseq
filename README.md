# emojiseq

A command-line tool that checks whether emoji codepoint sequences are
structurally well-formed.

## Why

Emoji are rarely a single codepoint. A "woman technologist" is a person, a
zero-width joiner, and a laptop, glued together. A country flag is two
regional indicator letters. A thumbs-up-with-skin-tone is a base character
immediately followed by one of five modifier codepoints. It's easy to get
these wrong when you're assembling a sequence list by hand (for a sticker
pack, a custom emoji picker, a chat client's autocomplete data) - a stray
extra joiner, a skin tone modifier on something that doesn't support one, or
half a flag pair will often still *look* fine in your editor and then render
as boxes or split glyphs somewhere else.

`emojiseq` reads a list of candidate sequences and tells you which ones are
structurally sound, so you can catch that before it ships.

## Input format

By default (`--format hex`), one sequence per line, written as
whitespace-separated hex codepoints (an optional `U+` prefix is accepted).
Anything after `;` or `#` is a comment. This mirrors how Unicode's own
`emoji-sequences.txt` / `emoji-zwj-sequences.txt` files list codepoints, so
you can paste directly from there.

```
# sequences.txt
1F44B                        ; waving hand
1F469 200D 1F4BB             ; woman technologist
1F1FA 1F1F8                  ; flag: united states
200D 1F600                   ; malformed: starts with a joiner
1F44D 1F3FB                  ; thumbs up, light skin tone
2764 1F3FB                   ; malformed: heart isn't a modifier base
```

With `--format text`, each line is raw UTF-8 emoji text instead of hex
codepoints - each Unicode scalar value on the line becomes one codepoint of
the sequence. This is for pasting actual emoji straight from a chat client
or a sticker pack's source file, rather than transcribing codepoints by
hand. In this mode only `;` starts a comment, not `#`, since `#` is itself a
valid keycap base character (`#️⃣`):

```
$ printf '%s\n' '👋' '👩‍💻' '#️⃣' | emojiseq --format text
ok   1F44B
ok   1F469 200D 1F4BB
ok   0023 FE0F 20E3
3 ok, 0 bad
```

## Usage

```
$ emojiseq sequences.txt
ok   1F44B
ok   1F469 200D 1F4BB
ok   1F1FA 1F1F8
bad  200D 1F600: sequence cannot start with U+200D; it must start with a base emoji
ok   1F44D 1F3FB
bad  2764 1F3FB: skin tone modifier at position 1 is applied to U+2764, which is not a modifier base
4 ok, 2 bad
```

`emojiseq` reads stdin if no file is given:

```
$ echo "1F469 200D 1F4BB" | emojiseq
ok   1F469 200D 1F4BB
1 ok, 0 bad
```

Exit status is 0 if every sequence passed, 1 if at least one failed, 2 for
usage errors (bad arguments, unreadable file).

## Strict vs. `--lenient`

By default `emojiseq` is strict: it enforces the well-formedness rules
Unicode's own emoji spec implies (no leading/trailing joiner, no doubled
joiner, skin tone modifiers only on characters registered as modifier
bases, flag sequences are exactly two regional indicators, tag sequences
end with the cancel tag, keycap sequences include the variation selector).

Pass `--lenient` to relax the "does this actually make sense" checks while
still rejecting input that has nothing to do with emoji at all (plain text,
unassigned codepoints). Use it when you want to accept sequences that some
fonts render acceptably even though they don't follow the letter of the
spec - for example a skin tone modifier on a base that isn't on the curated
modifier-base list yet, or a subdivision-flag-style run of more than two
regional indicators.

```
$ emojiseq --lenient sequences.txt
```

## Known limitations

The emoji-scalar and modifier-base tables in `src/sequence.rs` are
hand-curated subsets, not the full Unicode `emoji-data.txt`. Common emoji
are covered; obscure ones may be flagged as "not a recognized emoji-related
codepoint" even though they're valid. See the roadmap in the commit history
for where this is headed.

## Build

Standard library only, no external crates.

```
cargo build --release
```
