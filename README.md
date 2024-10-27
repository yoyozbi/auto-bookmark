# Auto-bookmark
A simple python script to generate typst code from pdf bookmarks.
The bookmarks used takes 5cmX15cm approximately in the pdf page.
Use it at your own risk if you're bookmarks are not this size.

To be able to use this script a simple flask server is used to upload the pdf file and serve the resulting pdf.
It uses flask_simplelogin to protect the server with a password.
