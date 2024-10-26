#set page(margin: (
  top: 0cm,
  bottom: 0cm,
  left: 2cm,
  right: 2cm
))

#grid(
  columns: (auto, auto, auto),
  rows: (auto, auto),
  column-gutter:  3cm,
  row-gutter: 0.7cm,
  align: center,

image("out/recto_0.jpg", width: 5.5cm),
image("out/recto_1.jpg", width: 5.5cm),
image("out/recto_2.jpg", width: 5.5cm),
grid.cell(rotate(90deg, image("out/recto_3.jpg", width: 5.5cm), reflow: true), colspan: 3),
grid.cell(rotate(90deg, image("out/recto_4.jpg", width: 5.5cm), reflow: true), colspan: 3),
)

#pagebreak()
#grid(
  columns: (auto, auto, auto),
  rows: (auto, auto),
  column-gutter:  3cm,
  row-gutter: 1cm,
  align: center,

image("out/verso_0.jpg", width: 5.5cm),
image("out/uglylove.png", width: 5.5cm),
image("out/devils4.png", width: 5.5cm),
grid.cell(rotate(270deg, image("out/dragon.png", width: 5.5cm), reflow: true), colspan: 3),
grid.cell(rotate(90deg, image("out/escape.png", width: 5.5cm), reflow: true), colspan: 3),
)
