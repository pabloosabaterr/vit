```
      .__  __
___  _|__|/  |_
\  \/ /  \   __\
 \   /|  ||  |
  \_/ |__||__|
```
# Vit

Vit is an idea for a vector-based VCS.
Unlike traditional VCS like Git, where hashing tries to make each
object as different as possible from every other, Vit hashes trying
to cluster similar commits into similar positions. **What does this
mean?**
Each word in a commit message is hashed to a fixed point in 2D
space (same word, same point, always). The commit is placed at the
weighted center of all its word-points. Commits that share words
end up close together. Clusters emerge naturally without any manual
categorization.
Pre-processing
Before hashing, the message is cleaned:

* Converted to lowercase ("Fix" and "fix" are the same word)
* Punctuation is stripped ("fix:" becomes "fix")
* Stopwords are ignored ("the", "a", "an", "to", "in", "of", etc.)
* Synonyms are resolved via an optional user-defined dictionary
  ("restructure" becomes "refactor", "repair" becomes "fix", etc.)
This ensures "Fix: Button" and "fix button" land on the same
point, and "refactor auth" and "restructure auth" can too.

## Hashing algorithm

### Step 1: word to number
Each character is multiplied by its position so that letter order
matters (otherwise anagrams like "stop" and "pots" would collide):

```
"stop" = s×1 + t×2 + o×3 + p×4 = 1128
"pots" = p×1 + o×2 + t×3 + s×4 = 1142   (different)
```

### Step 2: number to polar point (angle + distance)
The number is split into an angle and a distance from the center.:
```
angle    = (hash mod 360) x (π / 180)    (rads)
distance = hash * scale
```
scale is a user-set variable that can be set in a `.vitrc` file.

Short words end up close to the center (few loops), long words
further out (more loops). Words spread across the space by both
direction and distance:
```
"fix"       = 685   :  5.67 rad, distance 685    (close to center)
"button"    = 2359  :  3.47 rad, distance 2359   (mid range)
"dashboard" = 5820  :  1.05 rad, distance 5820   (far from center)
```

### Step 3: polar to cartesian
To combine multiple word-points into a center, polar coordinates
are converted to cartesian:
```
X = distance × cos(angle)
Y = distance × sin(angle)
```

This conversion is necessary because you can't average angles
directly (averaging 350° and 10° gives 180°, which is wrong, it
should be ~0°). In cartesian space, averaging works correctly.

### Step 4: weighted center of all words
Each word has a weight that decays by position. The first word
dominates, later words refine:

```
weight(i) = 1 / (1 + k × i)
```

Where i is the 0-indexed word position and k is a
user-configurable decay factor:

```
k=0  :  all words equal        (1.00, 1.00, 1.00, 1.00)
k=1  :  smooth decay           (1.00, 0.50, 0.33, 0.25)  [default]
k=3  :  aggressive decay       (1.00, 0.25, 0.14, 0.10)
k=10 :  almost first-word only (1.00, 0.09, 0.05, 0.03)
```

The weight never reaches zero. Every word contributes.
Example with k=1:

```
commit: "fix button login"

word       point (X, Y)     weight
"fix"      (-0.82, -0.57)   1.00
"button"   (-5.51, -2.25)   0.50
"login"    (2.85, 4.40)     0.33

center X = (-0.82×1.00 + -5.51×0.50 + 2.85×0.33) / (1.00 + 0.50 + 0.33)
center Y = (-0.57×1.00 + -2.25×0.50 + 4.40×0.33) / (1.00 + 0.50 + 0.33)
```

Why does this create clusters?
The first word pulls the hardest. All commits starting with "fix"
are pulled toward the point of "fix". The second word creates
sub-clusters within that region. Later words fine-tune the
position.

```
"fix button login"   :  close to "fix", pulled toward "button"
"fix button color"   :  close to "fix", pulled toward "button"   (near each other)
"fix header broken"  :  close to "fix", pulled toward "header"   (same region)
"feat new dashboard" :  close to "feat", different region entirely
```

The third dimension: time
X and Y come from the message. Z comes from the commit date. The project
grows in depth over time.
Looking at the space from the front: you see clusters by topic.
Looking from the side: you see the timeline, when each topic was
worked on.
