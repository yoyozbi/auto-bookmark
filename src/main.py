import typst
import pdf2image
import sys
import os
import shutil

# Get the path of the 4 PDF files from args
if len(sys.argv) < 2:
    print("Usage: python main.py **.pdf")
    sys.exit(1)

pdfs = sys.argv[1:]

if not all(p.endswith('.pdf') for p in pdfs):
    print("All files must be PDFs")
    sys.exit(1)

if not all(os.path.isfile(p) for p in pdfs):
    print("All files must exist")
    sys.exit(1)

# Reset out path content
if os.path.exists("out"):
    shutil.rmtree("out")
os.mkdir("out")

# Convert the PDFs to images
images = {
    "recto": [],
    "verso": []
}

recto_i = 0
verso_i = 0

for pdf in pdfs:
    pages = pdf2image.convert_from_path(pdf)
    for i, page in enumerate(pages):
        if i % 2 == 0:
            page.save(f"out/recto_{recto_i}.png")
            images["recto"].append(f"recto_{recto_i}.png")
            recto_i += 1
        else:
            page.save(f"out/verso_{verso_i}.png")
            images["verso"].append(f"verso_{verso_i}.png")
            verso_i += 1

print(images)
# Typst
NORMAL_IMAGE = '''
image("<PATH>",width: 5.5cm),
'''
ROTATED_90_IMAGE = '''
grid.cell(rotate(90deg, image("<PATH>", width: 5.5cm), reflow: true), colspan: 3),
'''

ROTATE_270_IMAGE = '''
grid.cell(rotate(270deg, image("<PATH>", width: 5.5cm), reflow: true), colspan: 3),
'''

BLANK_IMAGE = '''
block(width: 5.5cm)
'''

GRID = '''
#grid(
  columns: (auto, auto, auto),
  rows: (auto, auto),
  column-gutter:  3cm,
  row-gutter: 0.7cm,
  align: center,
'''
GRID_END = ")"
PAGE_BREAK = "#pagebreak()"

typst_content = '''
#set page(margin: (
  top: 0cm,
  bottom: 0cm,
  left: 2cm,
  right: 2cm
))

'''

NB_IMAGES_PER_PAGE = 5

for i in range(0,len(images["recto"]),NB_IMAGES_PER_PAGE ): # 5 images per pages
    recto_images = images["recto"][i:i+NB_IMAGES_PER_PAGE]
    verso_images = images["verso"][i:i+NB_IMAGES_PER_PAGE]

    if len(verso_images) >= 3:
        # Reverse the first 3 images
        verso_images = verso_images[:3][::-1] + verso_images[3:]

    if len(verso_images) == 2:
        # We need to add a blank image
        verso_images.append(False)
        verso_images = verso_images[::-1]

    typst_content += GRID
    for i, recto in enumerate(recto_images):
        if i < 3:
            typst_content += NORMAL_IMAGE.replace("<PATH>", recto)
        else:
            typst_content += ROTATED_90_IMAGE.replace("<PATH>", recto)

    typst_content += GRID_END
    typst_content += PAGE_BREAK

    typst_content += GRID
    for i, verso in enumerate(verso_images):
        if not verso:
            typst_content += BLANK_IMAGE
            continue

        if i < 3:
            typst_content += NORMAL_IMAGE.replace("<PATH>", verso)
        else:
            typst_content += ROTATE_270_IMAGE.replace("<PATH>", verso)

    typst_content += GRID_END

with open("out/out.typ", "w") as f:
    f.write(typst_content)

typst.compile("out/out.typ", "out/out.pdf")
