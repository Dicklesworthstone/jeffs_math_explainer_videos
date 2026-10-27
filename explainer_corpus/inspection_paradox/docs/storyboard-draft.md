# Storyboard, full draft — Why Your Bus Wait Is Longer Than You Expect

The full draft of this video's words and pictures, all twelve chapters.
The production storyboard, [storyboard.md](storyboard.md), holds only the
chapters with native scene source (now 01), and its `script.tsv` and the
chapter registry follow it (root Mail 6790). Every section of storyboard.md
after its preamble is copied here verbatim, and a test in `src/main.rs`
holds the two together. A chapter moves into storyboard.md when root assigns
it and its scene source exists.

`tools/storyboard.py` reads only storyboard.md. This draft gets the same
checks (gate G2, the 85 s cap, prediction pauses), and its whole script
written to a scratch file, with the command in
[WORK_STATUS.md](WORK_STATUS.md). Every number below is a fact computed in
`src/facts.rs` and recomputed another way by `oracle.py` (second by second
over the bus times, never the closed formula).

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question. The beat ends with the
question, and the picture holds (the four-second countdown ring and "pause
for more time") before the next beat answers it. Nothing that answers a
question is on screen before its answer beat starts.

Status: second draft, 2026-10-24, GoldGate. It revises the first draft (kept
unchanged in `previous/draft01/`) under the owner's direction of
2026-10-24, root Mail 6776:
- more setting, motivation and intuition before the machinery;
- no first-minute derivation deadline;
- bus-sampled against time-sampled on the same timetable before any
  algebra;
- a commuter kept as the anchor through the variance explanation;
- timetable-aware arrival tested as the model's limit;
- a closing transfer question.

Words, pictures and pause intentions exist for every chapter. Chapter 01's
scene source is `src/chapters/the_stop.rs`, uncompiled. Chapters 02 to 12
have no scene source and are not registered. Nothing has been rendered,
narrated or listened to. A storyboard promise isn't a working visual. The
twelve-chapter storyboard as it stood before this split is kept unchanged
in `previous/draft02-registry12/`.

Repair, 2026-10-24 (WildBass's review, root Mail 6819). The film's title
is now root's "Why Your Bus Wait Is Longer Than You Expect". 01_4's bars
are labelled as sped-up waits on two different days. 04_8 starts from
half of six. The 07_5 readout rounds explicitly. The 08_8 card counts
choices with repeats. K07 says "spread evenly in time". The 09_7 card
drops an unread book section. Chapter 11's planners keep their own ready
moments, and 12_5 pays off the train. The draft as it stood before this
repair is root's intake snapshot,
`narration_lab/third-wave-20261024/root/intake-0550/inspection_paradox/`.

## The arc

One example runs through the whole video:
- Two bus routes serve one stop. Each runs every 12 minutes, exactly on
  time, the red bus 2 minutes after the blue. The gaps go 2, 10, 2, 10 all
  hour.
- Ten buses an hour, a bus every 6 minutes on average, so the sign is true.
- **You** take a bus from this stop to catch a train. You never check the
  timetable. Your wait averages 4 min 20 s, not 3.
- The **passengers** are dots, one in the middle of every minute. They are
  the picture of "a moment chosen at random", and they never change identity:
  - the sixty of the hour (chapter 02);
  - the twelve of one stretch (chapter 04);
  - the bars and triangles (chapter 05);
  - the people who tell their gap (chapter 06);
  - the planners, each standing a minute before the first bus they can
    make (chapter 11).

The acts:
- **Act I, the puzzle and the two ways of counting (01 to 03).** The
  commuter's reason to care. The fair-looking guess of three minutes, the
  prediction, and the 4:20 surprise with nothing late. Then the gaps:
  - the bus company's count, one tag per bus, averages 6;
  - the passengers' count, one dot per minute, puts five in six of them in
    a long gap.

  Then one cause varied at a time. From even gaps, slide the red bus: the
  buses' share stays half while the passengers' share follows the gap's
  length, up to all of them when the buses come together.
- **Act II, the waits (04 to 06).** Twelve passengers wait 52 minutes
  between them, 4 min 20 s each. The waits drawn as triangles, the waiting as
  their area, 25 times as much in the long gap. The two averages with
  numbers (6 against 8 min 40 s), the wait as half the passengers' gap, and
  the name.
- **Act III, what controls it (07, 08).** No bus late, nothing bunched. Even
  gaps give 3 min, and the slider runs the wait from 3 to 6. The triangles
  become the formula: half the average gap plus the spread over twice the
  average gap. That is your minute and twenty seconds a morning, and never
  less than half the average gap.
- **Act IV, beyond the stop and back (09 to 12):**
  - completely random buses wait the whole average gap;
  - the same sum in class sizes, a profiler and a 1922 essay;
  - back to the stop: the timetable reader and the planners, the model's
    limit;
  - the planner who evens the gaps;
  - a last timetable for the viewer to solve from the picture;
  - your train: allow the longest wait, and expect the average.

| Chapter | Spine | Status |
|---|---|---|
| `01_the_stop` | you, the train, the stop, two routes, the sign; your lucky and unlucky mornings; the guess of 3 (a prediction); 4 min 20 s, nothing late; title; what we'll do | in storyboard.md and the registry; scene source written (`the_stop.rs`), uncompiled |
| `02_two_ways_to_count` | the gaps 2, 10; the bus company's tags, half 2, half 10, averaging 6; your moment sits in one gap; a random moment (a prediction); sixty passenger dots, fifty in long gaps; 1/2 of gaps, 5/6 of time | words and pictures only |
| `03_stretch_one_gap` | even gaps, both counts agree; slide the red bus a minute at a time; the buses' half never moves, the passengers' share 7, 8, 9, 10 of 12; together (a prediction), all of them; the sign still 6 | words and pictures only |
| `04_one_a_minute` | one twelve-minute stretch; twelve passengers; waits 1 and 5 on average by gap; 52 minutes shared by 12 (a prediction); why the guess of 3 fails | words and pictures only |
| `05_triangles` | the wait for every moment; each gap a triangle as wide and as tall as the gap; the passengers' bars hold exactly the triangles' area; 2 and 50; 25 times the waiting (a prediction); waiting grows with the square | words and pictures only |
| `06_two_averages` | the tags' average, 6 min; the passengers' answers, 8 min 40 s (a prediction); a random moment waits half its gap; the name; looking back as well as forward | words and pictures only |
| `07_no_bus_late` | no lateness, no bunching; even gaps (a prediction), 3 min; slide the red bus, now watching the wait; 4:20 at 2, 3:00 at 6, 6:00 together | words and pictures only |
| `08_the_formula` | half the squared gap per gap; the average of squares over twice the average gap; 52 = 36 + 16; 3:00 + 1:20, your minute and twenty seconds; never below half (a prediction) | words and pictures only |
| `09_at_random` | no timetable, buses with no memory (a prediction); the whole six minutes; the formula agrees; your gap averages twelve; more spread, past six | words and pictures only |
| `10_same_sum` | two classes of 2 and 10 (a prediction), eight and two thirds; Feld and Grofman; for programmers, the profiler; Terada's trams | words and pictures only |
| `11_back_to_the_stop` | the opening picture with the triangles; 52 over 12; you check the timetable (a prediction); every passenger plans, still five in six in long gaps, every wait a minute, the triangles no longer fit | words and pictures only |
| `12_one_more_timetable` | the planner evens the gaps; gaps 4 and 8 (a prediction), triangles 8 and 32, 3 min 20 s; your train, allow ten minutes, 4:20 is the average cost; your bus isn't late | words and pictures only |

## Vocabulary (one word per idea)

- **you**: the commuter, the anchor of the film. A dot with a ring that
  walks to the stop in 01 and comes back whenever "you" is said.
- **passenger**: one of the dots, one in the middle of every minute,
  arriving without a look at the timetable. Never "people" or "person" for
  the dots. "People" is used only for real people (Feld and Grofman,
  Terada).
- **bus**, **stop**, **the blue bus** and **the red bus** (the two routes;
  "route" only in 01_2), **timetable**.
- **the sign**: what it promises, ten buses an hour and a bus every six
  minutes on average.
- **gap**: the time from one bus to the next at this stop, of either
  route. "Headway" appears once, on a card, never in the voice.
- **short gap**, **long gap**. In 03 also **the gap that's growing**, the
  one from each red bus to the next blue bus.
- **tag**: the gap before a bus, written on that bus. There is one per bus,
  so one per gap.
- **turn up** is what you and the passengers do. Buses **come**.
- **at random**: any moment as likely as any other, with no look at the
  timetable. Defined in 02_4 and assumed whenever the video says "at
  random".
- **wait**: from turning up until the next bus comes.
- **waiting**: minutes waited, added over passengers. It is an area.
- **triangle**: the wait drawn over one gap.
- **averaged over buses**, **averaged over passengers** (or **moments**):
  the two averages.
- **the passengers' gap**: the gap a passenger turned up in.
- **spread**: the average squared distance of the gaps from the average
  gap. "Variance" appears once, on a card, never in the voice.
- **even gaps**.
- **completely at random**: buses with no memory, at every moment equally
  likely to come, however long since the last bus. "Poisson" appears once,
  on a card.

## Color roles

| Idea | Color |
|---|---|
| the blue bus, its markers, its row of times | blue `#58C4DD` |
| the red bus, its markers, its row of times | red `#FC6255` |
| a short gap: its bracket, its triangle, the passengers in it | teal `#5CD0B3` |
| a long gap (and in 03 the gap that's growing): its bracket, its triangle, the passengers in it | gold `#F0AC5F` |
| the true average wait, a verdict | green `#83C167` |
| the spread, the extra piece of the formula | violet `#C792EA` |
| the think ring and "?", the ring around **you** | `hero` `#FFD866` |
| the naive guess (half the average gap), axes, ticks, clock labels, bus tags | `dim`, `faint` |
| passengers before their gap is shown, numbers, signs | `ink` |

- A passenger is a small dot on the hour line. Once its gap is shown, the
  dot takes the gap's color.
- **You** are an ink dot one size larger with a hero ring, always the same
  object.
- Buses are short rounded markers standing on the line, never dots.
- The bus tags sit above the markers and the passengers below the line, so
  the two counts never share a row.

## Sources and scope

Sources read, and what was not read or watched, are in
[brief.md](brief.md). The video claims the time-average result for
passengers who turn up independently of the buses, uniformly in time, with
gaps that have a long-run average and mean square. It makes no claim about:
- why real buses bunch (catalog entry `bus_bunching`);
- how real passengers time their arrivals;
- any city's service.

Chapter 11 shows the model's limit with the same objects: the passengers
who plan.

## 01_the_stop: A bus every six minutes (~84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Every weekday you take a bus to the station to catch a train. You never check the timetable. You walk to the stop when you're ready and take the first bus that comes. | Cold open on a plain street diagram, left to right: a house, the bus stop (a post with a sign), the station with a train at the platform. **You** (ink dot with a hero ring) leave the house and walk to the stop. A small clock face on the station, no numbers on it yet | moves, shapes | |
| 01_2 | Two bus routes stop here on the way to the station. Each runs every twelve minutes, exactly on time, and the red bus comes two minutes after the blue one. | The street slides up and shrinks to a strip at the top, still showing you at the stop. Below, one hour as a line with a tick and a dim label every twelve minutes, :00 :12 :24 :36 :48 :60 (from `model::route`). On "Two bus routes" five blue markers stand up at :00 :12 :24 :36 :48, lagged. On "the red bus", five red markers at :02 :14 :26 :38 :50 | lagged reveal, number line from the model | |
| 01_3 | So the sign tells the truth. Ten buses an hour, a bus every six minutes on average. | The stop's sign, enlarged at left, gains two lines: "10 buses an hour" and "every 6 minutes on average" (F-T1.buses_per_hour, F-T1.mean_gap). On "Ten buses" all ten markers pulse once, left to right | lagged flash | |
| 01_4 | Your train won't wait for you, so you'd like to know how long you'll stand here. Some mornings a bus pulls in as you arrive. Some mornings you've just missed one. | Two of your mornings, on two different days, one after the other, as ghosts of you on the hour line (`model::lucky_morning`, `unlucky_morning`). Each bar is the wait as you arrive, then that wait running out, sped up, each morning by its own amount, so the two bars share no clock. First you land half a minute before the :12 blue bus. A gold bar reaches from the bus back to you, "wait 0:30 as you arrive" under it and a dim "one morning, sped up" to the right of the bus, and the bar shrinks steadily to the bus. That bar, its label and its note go, and the ghost dims. Then half a minute after the :02 red bus, a gold bar reaches back almost the whole gap, "wait 9:30 as you arrive" and "another morning, sped up", and it shrinks steadily to the bus (F-M1). Both mornings sit in the same long gap, so both bars are gold. The bar and its words go, and the two ghosts stay, dimmed, on the line | value tracker, live bar | |
| 01_5 | A fair guess is half of six, three minutes. Over many mornings, will your average wait be three minutes, more, or less? ⟨predict⟩ | Right panel, dim: "guess: half of 6 = 3 min". Three choice chips under it, "less", "3 min", "more". Countdown, "pause for more time". Nothing marks an answer | think countdown | |
| 01_6 | More. Four minutes and twenty seconds, and every bus is exactly on time. That's an extra minute and twenty seconds every morning, on average. | On "More" the "more" chip lights. On "Four minutes" a green line under the chips, "average wait: 4 min 20 s", and a "precisely" card (K01). On "exactly on time" a small green dot over each of the ten markers, lagged, and "every bus on time" above them. On "an extra minute" a hero "+ 1 min 20 s" under the green line | panel from facts, lower-third | |
| 01_7 | Nothing here is late, broken or crowded. The extra time comes from the timetable itself, and from how you pick the moment you turn up. By the end, you'll work it out for any timetable, and know when the answer stops applying to you. | The sign and the panel leave, and the two ghosts of you glow. On "By the end" three small dim icons in a row: a tag and a dot, a gap with arrows on its ends, a triangle. Then everything clears and the title card stands, "Why Your Bus Wait Is Longer Than You Expect" | icons, title card | hold |

### Notes for 01

- No derivation in chapter 01. The puzzle comes early, at 01_5 and 01_6,
  about 40 s in by the storyboard estimate. The first explanation starts in
  02.
- Prediction (01_5): the chips are on screen from "will your average wait",
  and nothing lights until 01_6.
- Exact version K01 (01_6): "average over moments of turning up spread
  evenly over the hour, with no look at the timetable; each wait runs to the
  next bus of either route; every bus exactly on time".
- 01_4's two mornings are the first and last passengers of the long gap in
  chapter 04 (waits 9½ and ½). The same ghosts return there.
- 01_4's bars are illustrations, not a clock. Each starts as the wait when
  you arrive, to scale on the hour line, and runs out sped up, the lucky
  bar over 1 s and the unlucky one over 2.6 s, so their rates differ. The
  words "sped up" say so, and the first morning's bar and words leave
  before the second morning begins.
- The train of 01_1 and 01_4 is paid off in 12_5 of the draft. A deadline
  needs the longest wait, ten minutes, just after a red bus leaves
  (F-W1.allowance); 4:20 is what an average morning costs.
- Misconception defused: "the bus is late" (01_2, 01_3 and the green ticks
  in 01_6).
- Pause intentions (for listening, not rules; `docs/pause-overrides.json`):
  - after "half of six, three minutes." a thought pause, 0.4 to 0.8 s,
    before the question;
  - after "More." 0.45 to 0.9 s, so the chip lights before the number;
  - after "exactly on time." 0.35 to 0.7 s;
  - after "the moment you turn up." 0.4 to 0.8 s before the preview;
  - the title hold.
- Pronunciation to probe: "minutes" and "minute" (the noun, never the
  adjective "my-NEWT"), "route" (the owner's choice of "root" or "rowt"),
  "weekday".

## 02_two_ways_to_count: The bus's count and yours (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Look at the gaps between buses. Two minutes, ten minutes, two, ten, all hour long. Five short gaps and five long ones. | Under the line, each gap gets a bracket and its length, lagged left to right: short ones teal "2", long ones gold "10". Two counters beside the line, "5 short gaps" in teal and "5 long gaps" in gold | brackets from the model, lagged reveal | |
| 02_2 | The bus company sees the timetable one bus at a time. Tag each bus with the gap before it. Half the tags say two minutes, half say ten, and they average six, the number on the sign. | Above each marker a small dim tag, the gap before it: 10, 2, 10, 2, … (`facts::gap_before_each_bus`; the bus at :00 follows the :50 bus of the hour before). The tags drop into a tally, "5 tags say 2, 5 tags say 10, average 6". The sign's "every 6 minutes" flashes | tags from the model, lagged reveal, flash | |
| 02_3 | That's not how you see it. You don't arrive with a bus. You arrive at some moment, and that moment sits inside a gap. | You step onto the line at a moment in a long gap. That gap brightens, and the others dim | highlight | |
| 02_4 | Say you turn up at a random moment, any minute of the hour as likely as any other. What's the chance you land in a long gap? ⟨predict⟩ | You slide slowly from end to end of the line with no gap shaded. From "What's the chance", the question card "chance of landing in a long gap?", the countdown and "pause for more time". No dots and no fraction | think countdown | |
| 02_5 | Five in six. Picture the other passengers turning up steadily, one in the middle of every minute. Sixty in the hour, and fifty of them land in a long gap. | Sixty ink dots drop below the line, one per minute, lagged left to right. Each takes its gap's color as it lands, 50 gold and 10 teal. A live counter, "in long gaps: 50 of 60" (F-S1.passengers_in_long) | lagged reveal, dot swarm, live counter | |
| 02_6 | The long gaps fill fifty of the sixty minutes. Half the gaps, but five sixths of the time. | The gold dots stay. Two bars to scale beside the line: "share of tags: 1/2" in dim (from the tags above) and "share of passengers: 5/6" in gold (from the dots below); "50/60 = 5/6" | bars to scale from facts | |
| 02_7 | So the dots measure time. Each gap gets one tag, from the bus that ends it, but a dot for every minute it lasts. Counting buses and counting moments give different answers, on the very same timetable. | One long gap brightens, with its single tag above and its ten dots below; then one short gap, with its single tag and two dots. It stands, then the chapter fades | highlight, comparison | hold |

### Notes for 02

- The bus-sampled and time-sampled counts share one timetable and one
  picture, tags above the line and dots below. No average of the
  passengers' answers yet (that waits for 06), and no variance.
- Prediction (02_4): no dots until 02_5.
- The tags' average (6) equals the sign's. That is the bus-weighted average,
  said in words, with no formula.
- The hold at 02_7 is for looking: one tag against ten dots, then one
  against two.
- Pause intentions:
  - "Five in six." 0.5 to 1.0 s before the dots start;
  - after "That's not how you see it." 0.35 to 0.7 s;
  - after "five sixths of the time." the cue transition.

## 03_stretch_one_gap: Stretch one gap (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | To see what controls this, start from a timetable with no surprise. Move the red bus to six minutes after the blue. Now every gap is six minutes. | The red markers slide right from +2 to +6 min. Every bracket becomes "6", drawn dim (neither short nor long). Every dot turns back to ink, six under each gap. The tags above all say 6 | moves, brackets from the model, recoloring | |
| 03_2 | Watch one kind of gap, from each red bus to the next blue bus. Half the buses end one of these gaps, and half the passengers stand in one. The two counts agree. | The red-to-blue gaps get a gold outline, the gap we'll stretch. Two readouts at the right, from the model: "tags on buses that end it: 1/2" (above) and "passengers in it: 6 of every 12" (below) | readouts from the model, highlight | |
| 03_3 | Now slide the red bus back toward the blue, a minute at a time. The same ten buses, still a bus every six minutes on average. Watch the passengers. | The red markers step left, 6 → 5 → 4 → 3 → 2 min after the blue, with a short hold at each step. The gold gaps widen. As a red marker passes a dot, the dot changes from ink to gold. The sign's two lines stay lit | value tracker, stepped moves, live recoloring | |
| 03_4 | The buses' count never moves. Half of them still end the gap that's growing. The passengers' count follows its length. Seven of every twelve, then eight, nine, ten. | The upper readout stays "1/2" through every step (dim, unchanging). The lower readout updates with each step, "7 of every 12", "8", "9", "10", spoken as it changes. A small plot beside it adds one point per step: passengers' share against the gap's length, on a straight line | live readout, data window | listen 3 s |
| 03_5 | Keep sliding until the red bus comes with the blue one. What share of the passengers is in the long gap then? ⟨predict⟩ | The red markers pause at +2. The question card "passengers in the long gap, buses together?"; countdown. Nothing moves | think countdown | |
| 03_6 | All of them. The short gap shrinks to nothing, so every passenger waits in a twelve-minute gap. And the sign still says a bus every six minutes. | The red markers slide to +0, standing beside the blue. Every dot is gold. The lower readout says "12 of every 12". The upper readout is replaced by "average gap over buses: 6 min (0, 12, 0, 12, …)" | moves, readouts | |
| 03_7 | So the timetable counts each gap once, however long it lasts. Your mornings land in a gap in proportion to its length. Stretch a gap, and more of your mornings fall inside it. | The red markers slide back to +2, our timetable. The dots return to 50 gold and 10 teal, and you reappear in a long gap. It stands, then the chapter fades | moves, recoloring | |

### Notes for 03

- One cause at a time: only the red bus moves. The number of buses, the
  sign's average gap and the passengers stay fixed. Every readout comes
  from `model` at the current offset (F-G1). The passengers' share is
  (12 − d)/12 (oracle self-test "stretch"). The buses' share is 1/2 at
  every offset from 1 to 6.
- Negative control: with even gaps the attractive idea "the sign's average
  is your experience" is true (03_1, 03_2). It fails only when the gaps are
  uneven.
- The `listen 3 s` in 03_4 is a visual hold: the last step to 10 of 12
  lands with no voice, so the viewer can compare the frozen upper readout
  with the moving lower one.
- Prediction (03_5): the red markers stop at +2 and nothing previews the
  answer.
- Edge (03_6): with the buses together, the blue and red bus share each
  moment, so the video gives no bus share there, only the sign's average
  gap (F-G1.mean_gap_every_offset includes the zero gaps).
- Pause intentions:
  - after "The two counts agree." the cue transition, kept clear before the
    slide;
  - "All of them." 0.5 to 1.0 s while the last red marker lands;
  - after "the gap that's growing." 0.3 to 0.6 s.

## 04_one_a_minute: Twelve passengers (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Back to our timetable. The hour is the same twelve minutes, five times over, so zoom in on one stretch. A blue bus, a red bus two minutes later, and ten minutes after that, the next blue bus. | The hour line's first twelve minutes widen to fill the frame, with the rest fading. Blue marker at 0, red at 2, blue at 12, labels 0, 2, 12 min. The brackets 2 (teal) and 10 (gold) stay under it, and "× 5 in an hour" in dim | scale the group, brackets | |
| 04_2 | It holds twelve of our passengers, one in the middle of each minute, none of them looking at the timetable. Now ask how long each one waits. | The twelve dots of this stretch widen with the line (the same objects), at 0.5, 1.5, …, 11.5 min, in their gap colors. Your two ghosts from 01_4 are on the first and last gold dots | identity-keeping scale | |
| 04_3 | The two passengers in the short gap wait a minute and a half, and half a minute. One minute each, on average. | From each teal dot a teal bar rises to its wait, 1½ and ½ (heights from `model::midpoint_waits`), labelled. Then "average 1 min" in teal beside the short gap | bars from the model | |
| 04_4 | The ten passengers in the long gap wait anywhere from nine and a half minutes down to half a minute. Five minutes each, on average. Your unlucky morning and your lucky one are the first and the last of them. | Gold bars rise from the ten gold dots, 9½ down to ½, lagged left to right (a staircase), and "average 5 min" in gold. On "Your unlucky morning" the first bar's ghost of you glows, "9:30", then the last, "0:30" (F-M1) | bars from the model, lagged reveal, callback | |
| 04_5 | So what's the average wait over all twelve passengers? ⟨predict⟩ | The question card "average wait, all 12 passengers?"; countdown. No total on screen | think countdown | |
| 04_6 | Not three. The two passengers in the short gap waited two minutes between them. The ten in the long gap waited fifty. | Panel: "1½ + ½ = 2" in teal, then "9½ + 8½ + … + ½ = 50" in gold, each as it is said; the terms follow the bars' left-to-right order | text from facts | |
| 04_7 | That's fifty-two minutes of waiting, shared by twelve passengers. Four and a third minutes each, or four minutes and twenty seconds. | "2 + 50 = 52 minutes of waiting", then "52 ÷ 12 = 4⅓ min = 4 min 20 s" in green, boxed. A "precisely" card (K02) | TeX derivation, verdict box, lower-third | |
| 04_8 | Half of six is the average of one and five, as if both gaps had the same number of passengers. They don't. The long gap has five times as many. | The guess from 01_5 returns, dim, "half of 6 = 3", and on "the average of one and five" it gains "= (1 + 5) ÷ 2", the 1 teal and the 5 gold; then a slash through the whole line. Under it "2 passengers" teal and "10 passengers" gold, "× 5" between. It stands, then the chapter fades | equation card, highlight | |

### Notes for 04

- Prediction (04_5): no sum on screen until 04_6.
- Exact version K02 (04_7): "passengers at the middle of each minute give
  exactly the average for a moment chosen at random, because the wait falls
  in a straight line inside each gap". This is fact F-X2.midpoint_exact,
  checked for every whole gap from 1 to 12 minutes. The oracle self-test
  also shows that passengers at the *start* of each minute do not give
  13/3.
- Misconception defused: "average the gaps' average waits" (04_8). The
  line starts from the viewer's own guess, half of six, and shows it is the
  same number as the average of the gaps' average waits: half of 6 is 3,
  and (1 + 5) ÷ 2 is 3 (F-T1.half_mean_gap, F-W1.naive_mean_of_waits). With
  one bus at each end of every gap, half the average gap is always the
  plain average of the gaps' half-lengths.
- Never read the twelve waits aloud. The bars and their labels carry them.
- Pause intentions:
  - "Not three." 0.4 to 0.8 s before the sum;
  - the cue break after "waited fifty." stays short (the default cue
    transition), so "That's fifty-two" follows as the same thought;
  - the comma in "a minute and a half, and half a minute" is connected
    (under 0.28 s);
  - after "Five minutes each, on average." 0.35 to 0.7 s before the
    callback.

## 05_triangles: The waiting is an area (~79 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Now draw the wait for every moment in those twelve minutes. Across is the moment you turn up. Up is how long you'll wait. | The twelve-minute line becomes the bottom axis of a plot (`Frame2`): across "moment you turn up (min)", 0 to 12; up "wait (min)", 0 to 10. The twelve bars stay, now standing in the plot | axes, data window | |
| 05_2 | Right after a bus leaves, you'd wait the whole gap. The wait shrinks steadily until the next bus comes, then jumps back up. | You (the ringed dot) ride along the bottom axis from 0 to 12. Above you a point traces the wait: from 2 down to 0 at the red bus, a jump to 10, down to 0 at 12. A live readout "wait: m:ss" follows | value tracker, live number, curve | |
| 05_3 | So each gap draws a triangle, as wide as the gap and just as tall. | The traced edges close into two filled triangles, teal 2 by 2 and gold 10 by 10, with width and height braces "2", "2", "10", "10" | shapes from the model, braces | |
| 05_4 | Give each of our twelve passengers a bar one minute wide, as tall as their wait. The bars hold exactly as much as the triangles. So the total waiting is measured by this area, because we have one passenger per minute. | Each bar widens to one minute. The little corners the bars stick out above the slanted edge appear together with the equal corners they leave empty. Bars and corners fade, then triangle fill pulses without moving off the axes | morph, fill emphasis | |
| 05_5 | The short triangle is half of two times two, which is two. The long one is half of ten times ten, which is fifty. The axes are both minutes, so these areas are in square minutes. At one passenger per minute, they represent the fifty-two minutes of waiting again. | In the right panel, "½ × 2 × 2 = 2 min²" teal and "½ × 10 × 10 = 50 min²" gold, then "2 + 50 = 52 min²". The narration explicitly multiplies area by one passenger per minute to recover the passengers' total wait | computed area labels, unit bridge | |
| 05_6 | The long gap is five times as wide as the short one. How many times as much waiting does it hold? ⟨predict⟩ | A width brace "× 5" from the small triangle to the large. The question card "how many times the waiting?"; countdown | think countdown | |
| 05_7 | Twenty-five times. It's five times as wide and five times as tall. Five times as many passengers, each waiting five times as long on average. | Copies of the small teal triangle fly into the large one and tile it, lagged: 15 upright and 10 upside down, 25 in all (counted by `model::tiling`). "× 25" | lagged group reveal, tiling | |
| 05_8 | So the waiting grows with the square of the gap. Double a gap and it holds four times the waiting. | The tiles fade. Clear the old ×5 arrow and ×25 answer. Mark a 5-minute half-width bracket in green and show "5 × 2 = 10 min". A 5-minute triangle (dim outline) fills the long one four times: "gap × 2 → waiting × 4". It stands, then the chapter fades | equation card, outline | |

### Notes for 05

- The plot's two scales are equal (a minute across equals a minute up), so
  each triangle really is as wide as it is tall. The scene refuses unequal
  scales.
- Each area is in square minutes. At the example's density of one passenger
  per minute, multiplying by that density yields passenger-minutes, the total
  waiting summed across passengers. Equal numerical values do not mean equal
  physical units. The later average divides the area by elapsed time.
- 05_2 rides you along the axis: the commuter becomes the moving point that
  draws the triangles.
- Prediction (05_6): no tiles until 05_7.
- The tiling is exact. A right isosceles triangle scaled by 5 is tiled by 25
  copies: `model::tiling(5)` gives 15 upright + 10 inverted, and the oracle
  counts them on a lattice.
- Pause intentions:
  - "Twenty-five times." 0.5 to 1.0 s while the first tiles land;
  - after "as tall as their wait." 0.6 to 1.2 s, so the bars finish
    widening before "The bars hold".

## 06_two_averages: Ask the buses, ask the passengers (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now put numbers on the two ways of counting. Averaged over buses, the tags say six minutes, the number on the sign. | The full hour line returns with its ten markers, the tags above and the sixty dots below, the same objects as in 02. "averaged over buses: (5 × 2 + 5 × 10) ÷ 10 = 6 min" | TeX from facts, callback | |
| 06_2 | Now ask each passenger how long the gap they landed in is. One in six says two minutes. Five in six say ten. | Each dot gets a tiny tag of its gap. The tally "10 say 2, 50 say 10" | dot swarm, tags | |
| 06_3 | So what's the average gap, as the passengers see it? ⟨predict⟩ | Question card "the passengers' average gap?"; countdown | think countdown | |
| 06_4 | Eight minutes and forty seconds. A sixth of the passengers say two, five sixths say ten. Same buses, same gaps, two honest averages. | "averaged over passengers: ⅙ × 2 + ⅚ × 10 = 8⅔ min = 8 min 40 s". Two boxes side by side, "over buses: 6 min" dim and "over passengers: 8 min 40 s" gold | equation card, comparison boxes | |
| 06_5 | And a passenger who turns up at a random moment in a gap waits half that gap, on average. Half of eight minutes forty is four minutes twenty. The same answer, a second way. | One gap highlighted, you sliding through it, your wait shown as the part of the gap ahead: "average: half the gap". Then "½ × 8 min 40 s = 4 min 20 s" in green, and a "precisely" card (K03) | highlight, equation card, lower-third | |
| 06_6 | Averaging over buses and averaging over moments give different answers, because long gaps last longer and get picked more. In probability, this is called the inspection paradox. | The two boxes again. Lower-third "names to search: inspection paradox · length-biased sampling" | lower-third | |
| 06_7 | Look back as well as forward. On average the last bus left four minutes twenty before you came. Behind you four twenty, ahead of you four twenty. Together, the eight forty of your gap. | A separate diagram shows the averages to scale, with you between two bus markers; it does not relabel a particular ten-minute gap. An arrow back, "since the last bus: 4 min 20 s on average", and one forward, "until the next: 4 min 20 s on average". The two join into "your gap: 8 min 40 s on average". It stands, then the chapter fades | arrows, equation card | |

### Notes for 06

- Prediction (06_3): the passengers' tally is on screen (10 say 2, 50 say
  10) but not their average until 06_4.
- Exact version K03 (06_5): "a moment chosen uniformly within a gap of
  length g waits g/2 on average; averaged over passengers the gap is 26/3
  min, so the average wait is 13/3 min" (F-S1.wait_share_of_gap,
  F-S1.person_gap).
- "Since the last bus" averages the same 4 min 20 s by symmetry. Inside each
  gap, the time behind you and the time ahead are mirror images (F-S1.age).
- Misconception defused: "one of the two averages is wrong" (06_4: both
  are honest answers to different questions).
- Pause intentions:
  - "Eight minutes and forty seconds." 0.5 to 1.0 s before the reasons;
  - "Same buses, same gaps," connected;
  - after "two honest averages." a thought pause, 0.5 to 1.0 s.

## 07_no_bus_late: No bus was late (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Notice what we didn't need. No bus broke down, no driver ran late, and nothing bunched up in traffic. Every gap was written into the timetable. | The hour line with its markers and the green "on time" ticks from 01_6. The two rows of timetable times above the line (blue :00 :12 …, red :02 :14 …) | ticks, table from the model | |
| 07_2 | Real buses do bunch. A late bus picks up extra passengers and falls further behind, while the bus behind it catches up. That's a different story, about why gaps grow. Here the gaps are simply uneven, and that alone stretches the wait. | A small dim inset at the top right, "bus bunching: delays feed on themselves (a different story)", with two markers drifting together on a short loop. The main picture is unchanged. A "precisely" card (K04) | inset card, lower-third | |
| 07_3 | Go back to the even timetable, with the red bus six minutes after the blue. Every gap is six minutes. What's the average wait there? ⟨predict⟩ | The red markers slide right from +2 to +6 min, as in 03_1. Every bracket becomes "6", dim. The question card "average wait now?"; countdown | moves, brackets from the model, think countdown | |
| 07_4 | Three minutes. Half of six, just what the sign suggests. With even gaps, the buses and the passengers agree. | Ten equal triangles 6 by 6 above the line; "over buses: 6 min", "over passengers: 6 min", "average wait: 3 min" in green | shapes from the model, comparison boxes | |
| 07_5 | Now slide the red bus again, this time watching the wait. The same ten buses an hour, the same average gap of six minutes. Only the wait changes. | A slider "red bus after blue: d min", swept from 0 to 12 and back to 2. The triangles above the line resize live. At the right, a plot of average wait against d traces a U from the model (`model::offset_wait`), with a live readout "average wait: m:ss" | value tracker, live recomputation, curve, plot | listen 5 s |
| 07_6 | Two minutes apart, it's four minutes twenty. Six apart, it's three. And if the two buses come together, you're really waiting for a bus every twelve minutes, and the average is six. | Dots on the U at d = 2 (4:20), d = 6 (3:00) and d = 0 (6:00), each marked as it is said, with the slider jumping there | plot highlights | |
| 07_7 | Same buses, same sign, and the average wait runs from three minutes to six. Only the spacing changed. | The U with its lowest and highest points labelled "3:00" and "6:00", and the sign "every 6 minutes on average" beside it. It stands, then the chapter fades | plot | |

### Notes for 07

- Prediction (07_3): the triangles are not drawn until 07_4.
- 07_5 is the same slide as 03_3, now with the wait. The passengers' shares
  from chapter 03 are still on the dots, so the U can be read as the cost of
  those shares.
- Exact version K04 (07_2): "the effect needs only uneven gaps, from any
  cause; bus bunching is one cause, a feedback between delays and boarding,
  outside this video".
- The slider moves continuously, so most of its offsets are not whole
  minutes, and most of their waits are not whole seconds: of the 121
  offsets 0, 0.1, …, 12 minutes, only 13 are (F-O1.tenth_offsets,
  F-O1.whole_second_tenths). The readout therefore takes the exact wait at
  the slider's offset (`model::offset_wait_rational`) and rounds it to the
  nearest second, halves up (`model::clock_rounded`). At 2.5 minutes the
  wait is 193/48 min, 4 min 1.25 s, shown "4:01" (F-O1.between_wait,
  F-O1.between_readout). That readout is the video's one rounding, and
  nothing spoken or boxed uses it. `model::clock` still refuses anything
  but whole seconds. Every whole-minute offset gives a whole number of
  seconds (F-O1.wait_clocks), and the readout shows those exactly. Only
  three of them are spoken.
- Edge (07_6): two buses at once still count as two buses (ten an hour, and
  the sign's six minutes is still true over buses, zero gaps included), but
  the wait is 6:00 (oracle self-test "together").
- Pause intentions:
  - "Three minutes." 0.4 to 0.9 s;
  - the listen window in 07_5 is a visual hold (the sweep), five seconds of
    picture with no voice.

## 08_the_formula: The formula, from the triangles (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Now turn the triangles into a formula for any timetable. The gap of two holds two square minutes, and the gap of ten holds fifty. Any gap holds half its length squared. | The two triangles from 05 retain their minute axes. Measured copies carry the bracket numbers into "2 min → ½ × (2 min)² = 2 min²", the corresponding ten-minute row, then "g → ½ × g² (g: a gap's length, min)". Faint mirror halves complete each square. Keep the one-passenger-per-minute area-to-wait bridge visible | computed geometry, measured text copies, separate outlines | |
| 08_2 | Add up the triangles and divide by the time they cover. That's the average wait. | Carry areas and gap lengths into "average wait = (2 + 50) min² ÷ (2 + 10) min", then "= 52 min² ÷ 12 min = 4⅓ min". Carry the same pieces into letters below | measured copies, unit cancellation, separate halo | Hold the sum in letters before cleanup |
| 08_3 | Divide the top and the bottom by the number of gaps. The top becomes half the average of the squared gaps. The bottom becomes the average gap. | Explicit top/bottom division by the number of gaps precedes "average wait = average of g² ÷ (2 × average gap)". Carry the pieces into their measured slots, with "52 ÷ (2 × 6)" underneath | measured text copies, sequential row changes | Hold the formula before the square argument |
| 08_4 | For our gaps, two squared is four and ten squared is a hundred, so the average square is fifty-two. Six squared is only thirty-six. The extra sixteen measures how uneven the gaps are. | The triangles and mirrors form 2×2 and 10×10 squares; compare average square52, mean square36 and violet corner16, all in min². Mark both distances from6. Then reveal the two signed-strip decompositions, cancellation, the definition of spread, and the general identity in that order | computed squares and strips, conserved areas, sequential reading | After speech, preserve each argument row's reading deadline; roughly18 seconds of authored inspection |
| 08_5 | So the average wait is half the average gap, three minutes, plus sixteen over twice six, which is a minute and twenty seconds. There's the extra time. | "average wait = ½ × average gap + spread ÷ (2 × average gap)". Under it "3 min + 16 ÷ 12 min = 3:00 + 1:20 = 4:20", the 1:20 in violet, boxed in green | TeX derivation, verdict box | |
| 08_6 | For you at the stop, that minute and twenty seconds is the cost of uneven gaps. The sign can't show it, because it counts buses, not mornings. | The street strip from 01 returns at the top, with you at the stop. The violet "1:20" flies from the formula to you; the sign's "every 6 minutes" dims beside it | callback, move | |
| 08_7 | Can the average wait ever be less than half the average gap? ⟨predict⟩ | Question card "ever less than half the average gap?"; countdown | think countdown | |
| 08_8 | No. The extra piece is never negative, so for passengers who turn up at random, the average wait is at least half the average gap. It's exactly half only when every gap is the same. | The violet term flashes: "spread ≥ 0, and spread = 0 only for even gaps". A small dim check card: "every choice of up to 4 gaps (repeats allowed, order ignored), each 0 to 12 min: 2,375 choices, 0 below half; equal only when all the gaps are the same". That is F-X1, a finite check; the square argument is the proof. A "precisely" card (K05). It stands, then the chapter fades | flash, card, lower-third | |

### Notes for 08

- Numbers before letters. Every step is shown with 2 and 10 before g.
- "spread" is defined by its picture (the violet 16) before it is named.
  "Variance" appears once, in K05.
- The commuter anchor (08_6) restates the variance term as your 1:20 a
  morning (F-W1.extra_clock equals F-F1.penalty_clock).
- Exact version K05 (08_8): "for passengers turning up uniformly in time,
  independently of the buses: average wait = E[G²] ÷ (2 E[G]) = E[G]/2 +
  Var(G) ÷ (2 E[G]); equal to E[G]/2 exactly when Var(G) = 0. Larson &
  Odoni, Urban Operations Research, §2.13, 'random incidence'".
- The always-sentence (08_8) is tested at its edges:
  - even gaps give equality (oracle "even gaps" and F-X1.equal_iff_even);
  - zero gaps are included in the sweep;
  - a strictly uneven pair exceeds half.
- Pause intentions:
  - "No." 0.4 to 0.8 s;
  - after "There's the extra time." a thought pause, 0.6 to 1.1 s;
  - after "not mornings." the cue transition before the prediction.

## 09_at_random: No timetable at all (~77 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | What if there's no timetable at all? Suppose buses come completely at random. At every moment a bus is just as likely to come, however long it's been since the last bus. | A fresh hour line. Markers appear at irregular moments (a seeded illustration, labelled "illustration", with no numbers on it). Over each new marker a dim clock face that does not reset | moves, seeded picture | |
| 09_2 | Keep the same average, a bus every six minutes. Now some gaps are tiny and some are long, and the long ones hold huge triangles. | The wait triangles rise above the irregular line, a few tiny and a few huge; "average gap 6 min (model)" | shapes, label | |
| 09_3 | How long is the average wait now? ⟨predict⟩ | Question card "average wait now?"; countdown | think countdown | |
| 09_4 | Six minutes. The whole average gap, not half of it. | "average wait: 6 min" in green beside "average gap: 6 min (model)", and a "precisely" card (K06) | equation card, lower-third | |
| 09_5 | The buses have no memory. So from the moment you turn up, the wait ahead of you is just like a brand new gap, and a gap averages six minutes. | You land on the line. Everything behind you dims, and the wait ahead is drawn as a fresh gap labelled "a new gap: 6 min on average" | highlight | |
| 09_6 | The formula agrees. For completely random gaps the spread is the average gap squared, thirty-six. Three minutes, plus thirty-six over twelve. Six. | "spread = 6² = 36 (model)"; "3 + 36 ÷ 12 = 6" with the 3 dim and the 36 ÷ 12 violet, boxed in green | TeX derivation | |
| 09_7 | Look back too, and the last bus left six minutes before you, on average. So the gap you land in averages twelve minutes, twice the average gap. | Arrows back and forward from you, "6 min" each, joined into "your gap: 12 min on average". Lower-third "names to search: waiting-time paradox, bus paradox" | arrows, lower-third | |
| 09_8 | Even gaps wait three minutes, random buses wait six, and our timetable sits at four twenty. More spread means a longer wait, even past six. The formula needs only the average gap and the spread. | A dim scale of average wait for a bus every 6 minutes on average: "even gaps: 3 min", "our timetable: 4 min 20 s", "completely random: 6 min", the scale running on past 6 with an arrow "more spread". A "precisely" card (K07). It stands, then the chapter fades | number line, lower-third | |

### Notes for 09

- The irregular markers are a seeded picture, not data. Nothing measured is
  shown, and every number on screen is a model number labelled "model".
- Exact version K06 (09_4): "model: buses as a Poisson process, one every 6
  minutes on average; gaps exponential, mean 6, variance 36; passengers
  turn up independently of the buses; long-run averages". The oracle uses
  exact moments k! μ^k. Its self-test adds quadrature and a seeded
  simulation of 200,000 arrivals (5.969 min); neither is shown.
- Exact version K07 (09_8): "the formula holds for passengers who turn up
  independently of the buses and spread evenly in time, when the gaps have
  a long-run average and spread (finite mean and finite mean square)".
  Both conditions on turning up are needed. Someone who turns up at :02:30
  every day is independent of a fixed timetable, yet waits 9:30 every day,
  not 4:20.
- The 09_7 search names come from the one source read on them: Masuda and
  Hiraoka's abstract says Terada "described the waiting-time paradox, also
  called the bus paradox". No book section is cited on the card, because
  none was read (brief, "Located, not read"). The line makes no claim
  about where real services sit on the scale. "Even past six" is checked:
  gaps 0, 0, 18 keep the average gap at 6 and wait 9 (F-R2). The scale's
  right end is never labelled a worst case.
- Pause intentions:
  - "Six minutes." 0.6 to 1.2 s, the chapter's surprise;
  - after "The buses have no memory." 0.45 to 0.9 s.

## 10_same_sum: The same sum elsewhere (~98 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | The same sum turns up far from bus stops. Whenever you pick a person or a moment at random instead of a group, the big groups get picked more often. | Chapter 04's stretch returns as it was: its twelve passengers on the line, two teal in the short gap and ten gold in the long one, under the two gaps' brackets. At "a person or a moment" the dots flash one by one, at "instead of a group" the brackets flash, and at "the big groups" the ten gold dots and the long bracket flash. The picture shrinks into the top-left corner, every dot with it, and two empty rooms appear | morph, callback | |
| 10_2 | Now seat those twelve passengers in a school with two classes, one of two students and one of ten. The school says the average class has six students. | The same twelve dots fly out of the corner one by one and grow into students: the two teal into the small room, the ten gold into the big one, five by two. The rooms flash at "two classes", the small class at "one of two students", the big one at "and one of ten". At "The school says" a teal and a gold bar, each as tall as its class, are born in the rooms and settle side by side in the upper row, captioned "one bar per class" and "classes of 2 and 10". At "six students" the row levels off at 6 and "school's average: (2 + 10) ÷ 2 = 6" appears at the top | dots, bars from facts, equation card | |
| 10_3 | Ask every student in turn how big their class is. | The students are asked one at a time, the first three slowly. A tiny tag under each says its class's size, 2 or 10, and a bar as tall as the answer is born at the tag and joins the lower row, captioned "one bar per student" and "2 say 2, 10 say 10". The asking runs on past the line, one student at a time, until all twelve have answered | individual answers, visual hold | hold |
| 10_4 | What's the average answer? ⟨predict⟩ | The question card "students' average?" and the countdown, "pause for more time". The twelve bars are up; their level and their sum are not | think countdown | |
| 10_5 | Eight and two thirds. It's the bus sum again, with students in place of minutes. Ten of the twelve students sit in the big class. | The lower row levels off at "8⅔" in gold, and "students' average: (2 × 2 + 10 × 10) ÷ 12 = 104 ÷ 12 = 8⅔" appears under the school's sum. At "It's the bus sum again" 06_4's "over passengers: 8 min 40 s" appears under the corner picture while its brackets flash. At "students in place of minutes" the students flash; at "Ten of the twelve students" the ten gold students and their bars flash, and the 8⅔ is ringed | equation card, callback | |
| 10_6 | Scott Feld and Bernard Grofman wrote about this in nineteen seventy-seven and called it the class size paradox. | Lower-third "source:" with "Feld & Grofman, 'Variation in class size, the class size paradox, and some consequences for students', Research in Higher Education 6 (1977) 215–222", held until it can be read. The students fly back into the corner as passengers while the school fades | lower-third | |
| 10_7 | For programmers. A sampling profiler checks what your program is doing many times a second, so it measures where the time goes, not how often each function is called. On average, one ten millisecond call shows up as often as five two millisecond calls put together: half the samples, but only a sixth of the calls. | A framed panel slides in from the right under the corner picture, headed "for programmers": three loops of an example program along a line, each one long gold call and five short teal calls, named "10 ms" and "2 ms" under the first loop. At "checks what your program is doing" evenly spaced checks drop onto the calls in their calls' colors. At "one ten millisecond call" the tally "10 ms call: ½ of the time, ⅙ of the calls" in gold while the long calls flash; at "five two millisecond calls" "2 ms calls: ½ of the time, ⅚ of the calls" in teal while the short calls flash; at "half the samples" "checks landing in the 10 ms calls: ½" while the gold checks flash. At "a sixth of the calls" the gold tally is ringed | code panel, tallies from facts | |
| 10_8 | That needs checks that aren't in step with the program. Checks that tick exactly with its loop land in the same function every time, and the shares come out completely wrong. | At "aren't in step" the even checks and their line go. At "Checks that tick exactly with its loop" a bracket marks one loop, "the loop", and one check a loop drops into the middle of each long call. At "land in the same function every time" the line "checks in step with the loop: all in the 10 ms calls". At "the shares come out completely wrong" the same checks slide a long call later and turn teal, and the line becomes "started later: none in the 10 ms calls". A "precisely" card (K08), held until it can be read | aliasing control, lower-third | |
| 10_9 | And people noticed it over a century ago. In nineteen twenty-two the physicist Torahiko Terada wrote an essay about waiting for Tokyo's trams, with his own observations of the trams. | The panel goes, and the corner picture grows back to its place, the twelve passengers on their line again. At "In nineteen twenty-two" lower-third "source:" with "Torahiko Terada, physicist, essay of 1922; English translation and discussion: N. Masuda & T. Hiraoka, arXiv:2004.14081 (2020)". It stands until it can be read, then the chapter fades | callback, lower-third | hold |

### Notes for 10

- Prediction (10_4): the twelve bars are up before the question, each from
  its own student's answer; their level, the 8⅔ and the 104 ÷ 12 are not on
  screen until 10_5. The question is a line of its own because the twelve
  answers take longer than "Ask every student in turn how big their class
  is." The asking runs on in silence and the question waits for it.
- The students are chapter 04's twelve passengers, the same dots. The class
  sizes are the stretch's gaps' passenger counts (F-W2.people,
  F-W2.long_people), so each student's answer is the gap its passenger
  stood in, and the students' average is 06's passengers' gap
  (F-S1.person_gap, 8:40). The dots go back to the corner as passengers in
  10_6 and home to their line in 10_9.
- The tags on the students repeat the tags on the passengers in 06_2:
  same shape, same place below the dot.
- Feld and Grofman: names, title, journal, volume, pages and year are as
  printed on the article's first page (scan hosted by UC Irvine, pages 215
  and 216 read 2026-10-24; brief.md). Page 216: "We call this the class
  size paradox." The voice claims only that they wrote about it in 1977 and
  named it.
- Terada: the voice claims only what the translators' abstract says: "a
  Japanese physicist, Torahiko Terada", an essay "in 1922", who "observed
  and analyzed data on Tokyo City trams". The essay itself was not read.
- The profiler is an example program, not a measurement: a loop of one
  10 ms call and five 2 ms calls (F-P1), three loops drawn. Each tally keeps
  the time share and the call share apart, "of the time" beside "of the
  calls".
- The even checks, every 3 ms from 1.5 ms, share no factor with the 20 ms
  loop and run over whole loops, so exactly half land in the long call.
  Over the first loop alone they do not, and checks at random moments land
  there half the time only on average; hence "On average" in 10_7. Checks
  in step with the loop, one a loop, land all in the long call or, started
  10 ms later, none, and a check every 4 ms gets three fifths or two fifths
  by where it starts. No sampling clock is promised the time share exactly.
- Exact version K08 (10_7, 10_8): "on average and over a long run, samples
  taken often and not in step with the program fall in each function in
  proportion to its running time".
- Pronunciation to probe: Feld, Grofman ("GROFF-man"), Torahiko Terada
  ("toh-RAH-hee-koh teh-RAH-dah", to be checked against a Japanese source
  and the owner), "millisecond".
- Pause intentions:
  - "Eight and two thirds." 0.4 to 0.9 s;
  - "For programmers." 0.35 to 0.7 s, a label, not a sentence the voice
    leans on;
  - the end of 10_3 is a visual hold: the twelve answers run on with no
    voice until the last bar is in, about three seconds of picture, and
    the question waits for it. 10_3 and 10_4 are never one sounding unit;
  - the source card after 10_6, K08 after 10_8 and the Terada card after
    10_9 are reading holds, picture with no voice until each card can be
    read.

## 11_back_to_the_stop: Back to the stop (~81 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Back to the stop. Ten buses an hour, all on time, a bus every six minutes on average. All of that is true. | The opening picture rebuilt as 01_3 left it: chapter 01's street strip with you at the stop, halved at the top; the hour line with the blue and red markers; the sign. At "Ten buses an hour" the markers pulse in time order; at "all on time" a green dot over every marker and "every bus on time"; at "a bus every six minutes on average" the sign's last line is ringed; at "All of that is true" the sign's two lines with numbers are ringed in green | rebuild of 01 | |
| 11_2 | But you don't sample buses when you turn up. You sample a moment. Five times in six, that moment falls in a long gap, where the waits are long. | "every bus on time" and its dots go. At "you don't sample buses" the tags appear over the markers, as in 02; at "You sample a moment" the sixty dots appear under the line, one in the middle of every minute. At "Five times in six" the long gaps shade gold and every dot takes its gap's color. At "where the waits are long" the tags and the shading go, and the wait triangles rise over the whole hour, a minute up as long as a minute across, five small teal and five large gold, see-through, behind the markers | shapes from the model, callback | |
| 11_3 | The area of the triangles, divided by the time they cover. Fifty-two square minutes over twelve minutes. Four minutes and twenty seconds. | At "The area of the triangles" the first twelve minutes brighten and the other four stretches dim, labelled "the same × 5"; the short triangle is labelled "2 min²" left of the line and the long one "50 min²" inside it. At "divided by the time they cover" a bracket under the stretch, "12 min". At "Fifty-two square minutes" the green box "average wait: 52 min² ÷ 12 min = 4 min 20 s"; at "Four minutes and twenty seconds" it is ringed, and it stands until it can be read | areas from facts, verdict box | |
| 11_4 | Now suppose you do check the timetable, and turn up one minute before your bus. How long do you wait? ⟨predict⟩ | The labels go and the triangles return to one brightness. At "turn up one minute before your bus" you leave the strip and grow back to full size under the passengers' row at :25, one minute before the red marker at :26, with a thin line up to your moment on the hour line, and that marker flashes. At "How long do you wait" the question "your wait?" beside you; countdown, "pause for more time" | think countdown | |
| 11_5 | One minute. You're no longer turning up at random, so the triangles' average doesn't describe you. | A green segment grows up from your moment to the triangle's slant, the triangle's own height there, and "wait: 1 min" stands beside you. At "the triangles' average doesn't describe you" the triangles and the box dim | highlight | |
| 11_6 | And if every passenger planned? Each would aim for the first bus they could make with a minute to spare, and stand at the stop one minute before it. Ten would wait for each blue bus, and two for each red. | You, your line, the box, the strip and the sign go. At the top, a panel: "share of passengers in long gaps", a bar filled five sixths in gold, "⅚"; "average wait", a bar that grows to 4 min 20 s, "4 min 20 s". At "with a minute to spare" the dots that are ready less than a minute before a bus flash, two in every twelve minutes. At "and stand at the stop" the sixty dots slide forward along the line, never back, each to one minute before the first bus it can make with a minute to spare (`model::planned_arrivals`); the dot at :59½ runs off the end at :60 and comes in again at :00, to stand at :01. Each dot takes the color of the gap it moves through and stacks down from the passengers' row, the dot with least far to slide on top: ten before each blue marker, gold, and two before each red, teal (F-K2.per_blue, F-K2.per_red), every stack in the last minute of its gap (F-K2.stand_at). At "Ten would wait for each blue bus" the blue stacks and markers flash, at "two for each red" the red ones | moves, stacking from the model, bars from facts | |
| 11_7 | Five in six would still stand in a long gap, but every wait would be one minute. They'd all stand in the last minute of a gap, so the triangles' average wouldn't describe them. Change how passengers arrive, and you change the model. | The "⅚" is ringed: the share stays (F-K2.in_long_share, equal to F-T1.long_time_share). At "every wait would be one minute" "4 min 20 s" goes, and while the wait bar falls steadily to a minute, "1 min" comes in beside it (F-K2.mean_wait_clock) and "wait 1 min" under every stack (F-K2.waits). At "They'd all stand in the last minute of a gap" the last minute of every gap is drawn over the line in its gap's color; at "the triangles' average wouldn't describe them" the gold triangles all but vanish. At "Change how passengers arrive" a "precisely" card (K09). It stands until it can be read, then the chapter fades | bars and labels from facts, lower-third | hold |

### Notes for 11

- 11_1 rebuilds chapter 01's picture from chapter 01's own street (the
  scene's copy is checked against `the_stop.rs`), with the hour, the
  markers and the sign as 01_3 left them.
- 11_3's box divides the area by the time in their units, "52 min² ÷ 12
  min". The letter formula is 08's and is not repeated here.
- Prediction (11_4): a negative control for the assumption. Arriving by the
  timetable is not arriving at random (oracle "timetable reader",
  F-K1.wait). Your wait is the triangle's own height at your moment, and
  "wait: 1 min" is not on screen until 11_5.
- 11_6 and 11_7 test the model's limit with the same dots. Each dot is
  the moment its passenger is ready. A planner stands at the stop one
  minute before the first bus at or after that moment plus one minute, so
  every dot moves only forward and nobody turns up before they're ready
  (F-K2.never_early). Nobody is shared out over the buses: the stacks of
  ten and two come from the dots and the timetable alone.
- Planning doesn't move passengers out of long gaps. Five in six still
  stand in one, exactly the time share (F-K2.in_long_share against
  F-T1.long_time_share). What changes is where in the gap they stand.
  Random arrivals are spread evenly through their gap, so on average half
  of it is still ahead of them (F-S1.wait_share_of_gap, 1/2). Every planner
  stands in the last minute of a gap, so on average only a sixth of it is
  ahead (F-K2.wait_share_of_gap, 1/6). The triangles assume the even
  spread, so they give 4:20 where the planners wait 1:00
  (F-K2.mean_wait_clock). The share of long gaps alone never gave the
  wait. Oracle negative control "planners".
- Not used: spreading the planners evenly, six to a bus. That would put
  half of them in a short gap only by making some pass up a bus they could
  make, which the film's planners don't do. The scene's tests keep it as a
  control: it also waits a minute, with only half in long gaps.
- Exact version K09 (11_7): "the average wait E[G²]/(2E[G]) is for arrival
  moments independent of the timetable and spread evenly in time;
  passengers who plan, or who arrive with other passengers' timing, need a
  model of how they arrive". The video makes no claim about how many real
  passengers plan.
- Pause intentions:
  - "One minute." 0.4 to 0.8 s;
  - after "And if every passenger planned?" 0.35 to 0.7 s before the dots
    move;
  - after "every wait would be one minute." a thought pause, 0.5 to 1.0 s,
    while the bar falls;
  - the verdict after 11_3 and K09 after 11_7 are reading holds, picture
    with no voice until each can be read.

## 12_one_more_timetable: One more timetable (~106 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Now forget the planners. Go back to passengers who never look at the timetable, one a minute, spread evenly over the hour. | Chapter 11's end rebuilt without its panel, its labels, its triangles or K09: the hour line, the blue and red markers of our timetable, and the sixty dots in their stacks a minute before the buses, each in its gap's color. At "Now forget the planners" the dots slide back along the line, never forward, each to the moment it was ready, the middle of its minute, and leave their stacks; the dot at :01 runs off the start and comes in again at :59½. Each takes the color of the gap it moves through. At "one a minute" the dots flash in time order, and at "spread evenly over the hour" the dim caption "at random: no look at the timetable, spread evenly over the hour" stands under them | rebuild of 11, moves, colors from the model | |
| 12_2 | Here's a last timetable to solve with the picture. Move the red bus to four minutes after the blue, so the gaps are four and eight. What's the average wait? ⟨predict⟩ | At "Move the red bus" the red markers slide from two minutes after each blue marker to four, and every dot after a red marker and before the next blue turns gold, the rest teal. Then brackets under every gap, "4" in teal and "8" in gold. At "What's the average wait" the question "average wait?" under the caption; countdown, "pause for more time". No triangle and no wait on screen until the countdown ends | moves, brackets, think countdown | |
| 12_3 | Raise the triangles again, a minute up for every minute across. The gap of four holds half of four squared, eight square minutes. The gap of eight holds half of eight squared, thirty-two square minutes. | The question and the ring go, and so do the brackets after the first twelve minutes. The wait triangles rise over the whole hour, a minute up as long as a minute across: five 4 by 4 in teal and five 8 by 8 in gold, see-through, behind the markers. At "The gap of four" the first short triangle brightens, the triangles after the first twelve minutes dim, and "4 min → ½ × (4 min)² = 8 min²" comes in at the top in teal; at "The gap of eight" the first long triangle brightens and "8 min → ½ × (8 min)² = 32 min²" comes in under it in gold | shapes from the model, rows from facts | |
| 12_4 | Together they make forty square minutes, over the twelve minutes they cover. Dividing square minutes by minutes leaves minutes. Forty over twelve is three and a third minutes. Three minutes and twenty seconds. | At "Together they make forty square minutes" "8 min² + 32 min² = 40 min²" under the two rows; at "over the twelve minutes they cover" the two brackets give way to one under the whole stretch, "12 min". At "Dividing square minutes by minutes" "average wait = 40 min² ÷ 12 min" lower left, and "min² ÷ min = min" dim under it; at "Forty over twelve" "= 3⅓ min"; at "Three minutes and twenty seconds" "= 3 min 20 s" in green, boxed and ringed. The whole step stands with no voice until it can be read | unit division, verdict box | hold |
| 12_5 | That's a point on the curve from before, between our timetable and the even timetable. The more even the gaps, the closer the wait comes to three minutes. | Before the voice, chapter 07's plot returns lower right at its place and scale: the offset axis "red bus after blue (min)" with its ticks, the wait axis, the whole U in green and its three points as 07_6 left them, d = 2 (4:20), d = 6 (3:00) and d = 0 (6:00). At "That's a point on the curve" the knob stands at four minutes on the offset axis with a guide up from it, and the U gains a green point at d = 4 (3:20). At "between our timetable and the even timetable" the "4:20" and "3:00" are ringed; at "The more even the gaps" the "3:00" flashes | plot callback from 07 | |
| 12_6 | And your train? Back on our timetable, four minutes twenty is what the stop costs you on an average morning. | The rows, the steps, the plot and the stretch bracket go, the dots fade and the triangles return to one brightness. Chapter 01's street strip comes back at the top, you at the stop and the train at the station. At "Back on our timetable" the red markers slide back to two minutes after the blue and the triangles reshape with them, five 2 by 2 in teal and five 10 by 10 in gold. At "four minutes twenty" "average cost: 4 min 20 s" in green, lower right | callback to 01, shapes from the model | |
| 12_7 | But a deadline needs your worst morning, not an average. Turn up just as a red bus leaves, and you wait the whole long gap, so allow the full ten minutes. | At "your worst morning" the gold triangle after the red bus at :02 brightens and the other triangles dim. At "Turn up just as a red bus leaves" you leave the strip and grow back to full size just under the line at :02, and a gold bar, chapter 01's wait as you arrive, grows from the blue bus at :12 back to you: the whole long gap, the triangle's tallest edge. At "allow the full ten minutes" "allow 10 min" under the bar in your color | bar from the model, callback to 01 | |
| 12_8 | Ten minutes is the worst case only because every bus runs exactly on time. It's a bound in this model, not a guarantee against real delays, or for the whole journey. | At "only because every bus runs exactly on time" a green dot over every marker and "every bus on time", as in 01 and 07. At "It's a bound in this model" a "precisely" card (K10). It stands until it can be read | lower-third | hold |
| 12_9 | Your bus isn't late. You're just more likely to turn up during a long gap. | Before the voice, K10, the bar and its label go, and you go back to the stop in the strip. At "Your bus isn't late" "every bus on time" is ringed in green. At "You're just more likely" the sixty dots return in their gaps' colors, and the five long gaps shade gold. When the line ends, the lower third "names to search:" with "inspection paradox · random incidence · length-biased sampling · renewal–reward theorem · class size paradox", held until it can be read. Then the picture fades and the receipts card comes up alone, "Every number in this video comes from the code", "and is checked against an independent oracle", "github.com/Dicklesworthstone/jeffs_math_explainer_videos" and "explainer_corpus/inspection_paradox". It stands until it can be read, then the film fades | lower-third, receipts card | hold |

### Notes for 12

- The chapter keeps the film's objects: the hour line and markers of 01,
  the sixty passengers of 02, the triangles of 05 and 11, the U of 07, the
  street strip and wait bar of 01. One tracker, the red bus's offset,
  moves the red markers and repaints the dots in 12_2 and 12_6, and the
  triangles follow it in 12_6.
- 12_1 undoes 11's planners on screen. Each dot slides back the way it
  came, to the moment it was ready. The film is back to arrival moments
  independent of the timetable and spread evenly in time, K01's condition,
  for the puzzle and the train (F-W2.per_minute).
- The transfer question (12_2) is new to the viewer but uses only the
  picture. Nothing about the answer is on screen until the countdown ends:
  no triangle, no area, no wait. The triangles come in 12_3, the wait at
  the end of 12_4. F-Q1 computes the puzzle second by second in the
  oracle, which checks it against the offset sweep at d = 4.
- 12_3 and 12_4 count area, as 08 and 11 do. Both axes are minutes, so a
  triangle's area is in square minutes; the two make forty square minutes
  (F-Q1.total_area), and dividing by the twelve minutes they cover leaves
  minutes, 3⅓ min (F-Q1.mean_wait), 3:20 (F-Q1.mean_wait_clock). The words
  never call the area minutes of waiting. 05_5's bridge, one passenger a
  minute turning square minutes into minutes of someone's waiting, is a
  different counting measure and is not used here.
- New facts F-Q1.short (4) and F-Q1.long (8), the puzzle's two gaps, the
  shortest and longest of `model::hour_gaps(PUZZLE_OFFSET)`, so the spoken
  "four" and "eight" of the gaps are bound to the gaps, not to the offset
  or an area.
- 12_5's point is the U's at four minutes (`no_bus_late::wait_of`); the
  scene refuses a U that is not nearer three wherever the gaps are more
  even, whole minute by whole minute.
- 12_6 to 12_8 separate the two numbers a traveller needs. The average
  wait, 4:20 (F-W1.mean_wait_clock), is the cost of an average morning. A
  deadline needs the longest wait. Turning up just as a red bus leaves you
  wait the whole long gap, ten minutes (F-W1.allowance,
  `model::longest_wait`; the oracle takes the worst of the hour's 3,600
  seconds and rounds up). That bound holds only with every bus exactly on
  time, and covers the wait at the stop alone.
- Exact version K10 (12_8): "with every bus exactly on time, the longest
  wait is the longest gap, 10 min, whenever you turn up; it says nothing
  about late buses or the rest of the journey".
- The last line restates the title's claim, "longer than you expect", in
  the honest direction. The wait is longer than the guess from the sign
  because of where random moments fall, not because a bus is late.
- The names and the receipts each get their own reading time: the names
  over the last picture, the receipts on a card alone after the picture
  fades.
- Departures from the earlier draft: its 12_1 (even gaps, three minutes)
  is 07_3 and 07_4 again and is dropped for the return to random arrivals;
  its 12_4 called the forty square minutes "forty minutes of waiting"; its
  12_5 is split into the cost (12_6), the margin (12_7) and the margin's
  condition (12_8).
- Pause intentions:
  - after "What's the average wait?" the countdown, a prediction hold;
  - after "three and a third minutes." a thought pause, 0.5 to 1.0 s, before
    the clock reading;
  - the step after 12_4 is an inspection hold, picture with no voice until
    it can be read, and the U then returns before 12_5 speaks;
  - after "And your train?" 0.35 to 0.7 s, the question the film opened
    with;
  - after "not an average." 0.4 to 0.8 s, the turn from the cost to the
    margin;
  - after "exactly on time." 0.5 to 1.0 s, while the green marks land;
  - K10 after 12_8, then the names and the receipts after 12_9, are
    reading holds, and the picture clears K10 before 12_9 speaks.
