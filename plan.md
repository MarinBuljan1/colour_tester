Colour Differentiation Web App Plan

Goal
Create a simple web app that tests how well a user can differentiate between colours based on hue.

Layout
- Top 75%: three colour boxes in a row (left, center, right). The center is the target colour.
- Bottom 25%: a colour map line chart where x-axis is hue and y-axis is the user's distinguishability at that hue.

Colour Model
- Use HSL.
- Saturation: 100%.
- Lightness: 50%.
- Hue distance is the shortest distance around the hue wheel (e.g., 350 to 10 is 20).

Trial Setup
- Pick a center hue (0-359).
- Compute a delta (the smaller offset) using the formula below.
- One side hue is delta away; the other side is 2 * delta away.
- Assign the closer/farther hues randomly to left/right.
- Hue wrap uses shortest distance logic.

Interaction
- User drags the center box left or right.
- If released beyond the midpoint between the center box and a side box, that side is selected.
- Provide immediate but small feedback on correctness.

Adaptive Difficulty (Delta)
Delta is computed only from counts of correct (R) and wrong (W) answers in the local hue band.

=1 + MAX((1 - (R+1) / MAX(1, (R+1) + (W+1))) * 64 - (R+1) + (W+1), 0)

Interpretation:
- Delta is the smaller hue offset.
- Example: delta = 1 means side hues are 1 and 2 degrees away from the center hue.

Bottom Map
- The line shows average delta needed to be correct for each hue.
- For each hue x, use all results within +/- 10 degrees to compute the average.
- No fixed bins; use the rolling window as described.
- The x-axis shows the hue colour so the user can see the position.

Flow
- User chooses a side.
- Update stats for the relevant hue band, recompute delta.
- Update the bottom line.
- Select a new center hue and repeat.

Persistence
- Store stats in localStorage so results persist between reloads.

Center Hue Selection
- Pick new center hues weighted toward under-sampled hues.

Tech Stack
- Existing project uses Rust + Yew (CSR) + WebAssembly with a PWA shell.
