# Why Robot Arms Think in Lie Groups: YouTube package

What goes on the upload page. The description is cut from `docs/description.md` (every number
there is an oracle key or a row of `docs/description_numbers.tsv`, which
`tools/description_numbers.py` checks) and links back to it for the full notes; the chapter times
come from the final cut (`./x lie_robotics chapters`; `tools/youtube_package.py fill lie_robotics
--finals renders/final` writes them over the `?:??` and checks them).

- Title: Why Robot Arms Think in Lie Groups (exactly this: no voice or review suffix)
- Voice: michael. The upload text is `publish/michael/{title,description,tags}.txt`; the description below equals `publish/michael/description.txt` byte for byte.
- Visibility: Private, for the owner's review. Only the upload lane uploads it, and it is made public only on the owner's word.
- Category: Education. Audience: not made for kids.
- Altered or synthetic content: Yes (disclosed: the narration is a synthetic voice and imitates no one; the one real person the video quotes, Michael Collins, is quoted from NASA's printed transcript, not depicted). The description says so too.
- Thumbnail: `renders/lie_robotics_thumbnail.png` (`./x lie_robotics thumbnail`)
- Captions: English, `renders/final/lie_robotics.en.srt` (`./x lie_robotics captions`)
- Tags: Lie group, Lie algebra, robotics, robot arm, kinematics, inverse kinematics, screw theory, SE(3), SO(3), product of exponentials, Jacobian, gimbal lock, Rodrigues formula, matrix exponential, Newton's method, math explainer

## Description

```text
Tilt a book a quarter turn, then roll it a quarter turn, then do it in the other order: the book faces a different way. Slide it right then away, or away then right: it lands on the same spot. Positions add; orientations don't. This video follows what a seven-joint robot arm (a KUKA LBR iiwa 14) has to do about that: turns, the one move that makes them add (do the arithmetic on the flat piece that touches the curved space, then wrap the answer back on), screws, a product of seven exponentials, the Jacobian, the directions an arm can lose, its spare joint, and Newton's method on the group of poses in three steps. The last quiz asks what is left over when two small turns are made in two orders, and the answer is the Lie bracket. (Lie is said "Lee".)

Chapters
0:00 The book
1:03 One turn
2:05 Three angles
3:26 Halfway
4:33 Spin
5:52 The platform
6:52 Screws
7:55 Seven screws
9:07 By hand
10:08 Velocities add
11:15 Lost directions
12:33 The spare joint
13:22 Steps
14:27 The steady path
15:21 Everywhere

The exact versions
Euler's theorem: every turn of space fixes an axis, and a product of turns is a turn.
A spin is an angular velocity. Its exponential is e^{[ω]t} = I + sin t [ω] + (1 − cos t)[ω]² for a unit axis ω and an angle t in radians (Rodrigues' formula); the logarithm goes back, with a branch of its own near a half turn.
Chasles–Mozzi: every rigid displacement is a screw displacement.
The arm's pose is a product of exponentials, T(q) = e^{[S₁]q₁} ⋯ e^{[S₇]q₇} M (Lynch and Park, eq. 4.14), which Brockett presented in 1983.
Singularity is a property of the joint angles, not of the gripper's pose: the same gripper pose can be reached by a singular and by a non-singular set of angles.
Newton's method on poses: error = log(inverse(T(q)) · T_goal), then q = q + pinv(J_body(q)) · error, until the error is below 0.001 rad and 0.1 mm. From other starts the count of steps differs, and the method can fail.
The leftover of two small turns made in two orders follows a rule of its own on the flat piece, the Lie bracket; for turns it is the cross product.

Every number in the video was computed by code and checked by a second, independent program. The exact version of every simplification, beat by beat, the arm's numbers to test with, every source and what was not verified:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos/blob/main/explainer_corpus/lie_robotics/docs/description.md

Names to search: Lie group, Lie algebra, SE(3), SO(3), screw theory (twist, wrench), product of exponentials, Chasles–Mozzi theorem, the matrix exponential and logarithm, Rodrigues' formula, the Jacobian (space form, body form), singularities, redundant manipulators and the null space, Newton–Raphson inverse kinematics, gimbal lock.

Sources
K. M. Lynch and F. C. Park, Modern Robotics: Mechanics, Planning, and Control (Cambridge University Press, 2017)
R. M. Murray, Z. Li and S. S. Sastry, A Mathematical Introduction to Robotic Manipulation (CRC Press, 1994)
J. Solà, J. Deray and D. Atchuthan, "A micro Lie theory for state estimation in robotics", arXiv:1812.01537
O. Rodrigues, Journal de Mathématiques Pures et Appliquées 5 (1840), pp. 380–440
NASA, Apollo 11 Technical Air-to-Ground Voice Transcription; the Apollo flight software Comanche 055 (routine GLOCKMON)
KUKA, data sheet "LBR iiwa 14 R820" (2022)

Narration: synthetic speech (FrankenTTS). Animation: franken_manim, a Rust rewrite of manim.
```

## Chapter titles (in order; the times come from the final cut)

The book | One turn | Three angles | Halfway | Spin | The platform | Screws | Seven screws | By hand | Velocities add | Lost directions | The spare joint | Steps | The steady path | Everywhere
