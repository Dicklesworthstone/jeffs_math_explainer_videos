# Why Robot Arms Think in Lie Groups

Video G explains why a robot arm's software multiplies where you would
expect it to add. Put a book on a table, tip it a quarter turn and roll it a
quarter turn; do the same two turns in the other order and it ends up facing
another way. Positions add in any order; orientations do not. The video
follows that one fact from a book to a seven-joint arm:

- **Turns form a group**, three angles always have a bad spot (the gimbal
  lock Apollo 11 flew around), and the average of two headings can point
  the wrong way.
- **The way out**: the space of orientations is curved, so do the arithmetic
  on the flat space of spins that touches it, and roll the answer back on
  with the exponential.
- **Screws**: a mobile platform on a floor turns about a point; in space every
  motion is one screw, a turn about an axis with a slide along it.
- **The arm** (a KUKA LBR iiwa 14 R820, with the lengths of its own
  description file) is seven screws, and its gripper's pose is a product of
  seven exponentials, worked by hand for two joints and then three.
- **Moving**: the Jacobian, the poses where the arm loses a direction and
  what they cost, and the spare seventh joint that lets the elbow swing
  while the gripper holds still.
- **Getting there**: inverse kinematics as Newton's method on the group
  (three steps for the pose shown), and the screw as the straight path
  between two poses.

Every chapter has a quiz with a visible countdown before its answer (15 in
all).

**Status: in production.** The brief, the facts and their oracle and the
storyboard are done and reviewed (`./x lie_robotics facts-check` passes;
review R1 is in `docs/reviews/lie_robotics/`); the script, scenes and drafts
follow. Nothing has been narrated, scored or
rendered at 4K yet.

```bash
./x lie_robotics test          # tests, clippy -D warnings, fmt --check
./x lie_robotics facts-check   # every number against oracle.py
./x lie_robotics storyboard    # gate G2; writes script.tsv from the storyboard
```

- `docs/brief.md`: the owner's request in his words, and every default chosen.
- `docs/fact-sheet.md`: every number, every source with its quoted passages,
  what was and was not verified, and the edge test behind each always/never
  sentence.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `src/model.rs`: the mathematics in closed forms (Rodrigues' formula, the
  screw exponential and logarithm, the product of exponentials, the
  Jacobian); `src/facts.rs`: every number the video shows or says;
  `oracle.py`: the same numbers by other routes (the URDF chain walked joint
  by joint, a generic series exponential and logarithm, finite differences).

The robot descriptions the numbers come from are cited by path in the fact
sheet and are not copied into this repository.
