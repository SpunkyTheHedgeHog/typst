// Test configuring font properties.

#[font "PT Sans", 10pt]

// Set same font size in three different ways.
#[font 20pt][A]
#[font 200%][A]
#[font 15pt + 50%][A]

// Do nothing.
#[font][Normal]

// Set style (is available).
#[font style: italic][Italic]

// Set weight (is available).
#[font weight: bold][Bold]

// Set stretch (not available, matching closest).
#[font stretch: ultra-condensed][Condensed]

// Error: 8-13 unexpected argument
#[font false]

// Error: 3:15-3:19 expected font style, found font weight
// Error: 2:29-2:35 expected font weight, found string
// Error: 1:44-1:45 expected font family or array of font families, found integer
#[font style: bold, weight: "thin", serif: 0]

// Warning: 16-20 should be between 100 and 900
#[font weight: 2700]

// Error: 8-28 unexpected argument
#[font something: "invalid"]

---
// Test font fallback and class definitions.

// Source Sans Pro + Segoe UI Emoji.
Emoji: 🏀

// CMU Serif + Noto Emoji.
#[font "CMU Serif", "Noto Emoji"][
    Emoji: 🏀
]

// Class definitions.
#[font serif: ("CMU Serif", "Latin Modern Math", "Noto Emoji")]
#[font serif][
    Math: ∫ α + β ➗ 3
]

// Class definition reused.
#[font sans-serif: "Noto Emoji"]
#[font sans-serif: ("Archivo", sans-serif)]
New sans-serif. 🚀
