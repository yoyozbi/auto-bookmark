import typst
import pdf2image
import sys
import os
import shutil
import uuid

from main import NB_IMAGES_PER_PAGE

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

TYPST_BASE = '''
#set page(margin: (
  top: 0cm,
  bottom: 0cm,
  left: 2cm,
  right: 2cm
))

'''

NB_IMAGES_PER_PAGE = 5 # Should not change unless the typst template changes
NB_TOP_IMAGES = 3 # Should not change unless the typst template changes

class Generate():
    def __init__(self, pdf_paths: list[str]) -> None
        self.pdf_paths = pdf_paths
        self.images = {
            "recto": [],
            "verso": []
        }
        self.temp_out_path = f"out/uuid.uuid4()"

    def generate(self) -> tuple[bool, str]:
        '''
        Generate the pdf that contains all the inputs pdfs
        Returns a tuple (success, message or pdfPath if successul)
        '''
        check_pdf_paths, message = self._check_pdf_paths()
        if not check_pdf_paths:
            return False, message

        os.mkdir(self.temp_out_path)

        for pdf in self.pdf_paths:
            recto_path, verso_path = self._convert_pdf_to_images(pdf)
            if not recto_path or not verso_path:
                print(f"Error converting {pdf}")
                return False, f"Error converting {pdf}"
            self.images["recto"].append(recto_path)
            self.images["verso"].append(verso_path)

        typst_content = self._generate_typst()

        with open(f"{self.temp_out_path}/main.typ", "w") as f:
            f.write(typst_content)

        out_path = f'out/{uuid.uuid4()}.pdf'
        try:
            typst.compile(f"{self.temp_out_path}/main.typ", out_path)
        except Exception as e:
            print(f"Error compiling the typst file: {e}")
            return False, f"Error compiling the typst file: {e}"
        finally:
            self._delete_out_temp_path()

        return True, out_path



    def _check_pdf_paths(self) -> tuple[bool, str]:
        '''
        Check if the PDF paths are valid
        '''
        if not all(p.endswith('.pdf') for p in self.pdf_paths):
            print("All files must be PDFs")
            return False, "All files must be PDFs"

        if not all(os.path.isfile(p) for p in self.pdf_paths):
            print("All files must exist")
            return False, "All files must exist"

        return True, ""

    def _delete_out_temp_path(self) -> None:
        '''
        Reset out path content
        '''
        if os.path.exists(self.temp_out_path):
            shutil.rmtree(self.temp_out_path)

    def _convert_pdf_to_images(self, pdfPath: str) -> tuple[str, str]:
        '''
        Convert the PDFs to images
        Return the paths of the images (recto, verso)
        '''
        pages = pdf2image.convert_from_path(pdfPath)

        if not pages:
            return "", ""
        if len(pages) < 2:
            return "", ""

        recto = pages[0]
        verso = pages[1]

        recto_path = f"{self.temp_out_path}/recto_{uuid.uuid4()}.png"
        verso_path = f"{self.temp_out_path}/verso_{uuid.uuid4()}.png"

        recto.save(recto_path)
        verso.save(verso_path)

        return recto_path, verso_path

    def _generate_typst(self) -> str:
        '''
        Generate the typst file from the images
        '''

        typst_content = TYPST_BASE
        typst_content += GRID
        for i in range(0,len(self.images["recto"]),NB_IMAGES_PER_PAGE ): # 5 images per pages
            recto_images = self.images["recto"][i:i+NB_IMAGES_PER_PAGE]
            verso_images = self.images["verso"][i:i+NB_IMAGES_PER_PAGE]

            if len(verso_images) >= NB_TOP_IMAGES:
                # Reverse the first 3 images
                verso_images = verso_images[:NB_TOP_IMAGES][::-1] + verso_images[NB_TOP_IMAGES:]

            if len(verso_images) == NB_TOP_IMAGES-1:
                # We need to add a blank image
                verso_images.append(False)
                verso_images = verso_images[::-1]

            typst_content += GRID
            for i, recto in enumerate(recto_images):
                if i < NB_TOP_IMAGES:
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

                if i < NB_TOP_IMAGES:
                    typst_content += NORMAL_IMAGE.replace("<PATH>", verso)
                else:
                    typst_content += ROTATE_270_IMAGE.replace("<PATH>", verso)

            typst_content += GRID_END

        return typst_content
