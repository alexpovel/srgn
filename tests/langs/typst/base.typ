#import "template.typ": *
#show: dept-news.with(
  title: "Sample Document",
  edition: [
    March 18th, 2023 \
    Purview College
  ],
  hero-image: (
    path: "newsletter-cover.jpg",
    caption: [Award-wining science],
  ),
  publication-info: [
    The Dean of the Department of Chemistry. \
    Purview College, 17 Earlmeyer D, Exampleville, TN 59341. \
    #link("mailto:newsletter@chem.purview.edu")
  ],
)
Hello this is me doing this actually!!!!!!!!

#circle(radius: 4.0cm)[
  #align(right + top)[
  Automatically \
  sized to fit.]
  #align(left + top)[
  Automatically \
  sized to fit.]
]
]

#polygon(
  fill: blue.lighten(80%),
  stroke: blue,
  (20%, 0pt),
  (60%, 0pt),
  (80%, 2cm),
  (0%,  2cm),
)



= The Sixtus Award goes to Purview
It's our pleasure to announce that our department has recently been awarded the highly-coveted Sixtus Award for Excellence in Chemical Research. This is a massive achievement for our department, and we couldn't be prouder.

I just love how typst does all this in realtime!

#blockquote[Prof. Herzog][
  Our Lab has synthesized the most elements of them all.
]

The Sixtus Award is a prestigious recognition given to institutions that have demonstrated exceptional performance in chemical research. The award is named after the renowned chemist Sixtus Leung, who made significant contributions to the field of organic chemistry.

Hallo this, is live

This achievement is a testament to the hard work, dedication, and passion of our faculty, students, and staff. Our department has consistently produced groundbreaking research that has contributed to the advancement of the field of chemistry, and we're honored to receive this recognition.

The award will be presented to our department in a formal ceremony that will take place on May 15th, 2023. We encourage all members of our department to join us in celebrating this achievement.

#article[
  = Guest lecture from Dr. Elizabeth Lee
  Elizabeth Lee, a leading researcher in the field of biochemistry, spoke about her recent work on the development of new cancer treatments using small molecule inhibitors, and the lecture was very well-attended by both students and faculty.

  In case you missed it, there's a recording on #link("http://purview.edu/lts/2023-lee")[EDGARP].
]

#article[
  = Safety first
  Next Tuesday, there will be a Lab Safety Training.

  These trainings are crucial for ensuring that all members of the department are equipped with the necessary knowledge and skills to work safely in the laboratory. *Attendance is mandatory.*
]

#figure(
  rect(width: 100%, height: 80pt, fill: white, stroke: 1pt),
  caption: [Our new department rectangle],
)

#article[
  = Tigers win big
  #text(weight: "bold", font: "Syne", pad(x: 12pt, grid(
    columns: (1fr, auto, 1fr),
    row-gutter: 8pt,
    text(32pt, align(right)[12]),
    text(32pt)[---],
    text(32pt)[4],
    align(right)[Tigers],
    none,
    [Eagles]
  )))

  Another great game on the path to win the League. \
  Go tigers!
]
