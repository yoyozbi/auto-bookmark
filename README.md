# Auto-bookmark
A simple python script to generate typst code from pdf bookmarks.
The bookmarks used takes 5cmX15cm approximately in the pdf page.
Use it at your own risk if you're bookmarks are not this size.

To be able to use this script a simple flask server is used to upload the pdf file and serve the resulting pdf.
It uses flask_simplelogin to protect the server with a password.

# Usage
Use the docker image to run the server. The server will be available at http://localhost:5000 (The default username is `admin`)
```bash
$ docker run -p 5000:5000 -e ADMIN_PASSWORD=<your-hashed-password> -e SECRET_KEY=<random-key> -e ALLOWED_HOSTS=* -d --name auto-bookmark auto-bookmark
```
Env variables:
- `ADMIN_PASSWORD`: The password to access the server (the username will be admin) you need to use `generate_password_hash` from `werkzeug.security` to generate the hash.
- `PWD_FILE`: The file to read the password from. If this is set, the `PASSWORD` variable will be ignored. The file should contain a `username:password` combo, one combo per line.
- `SECRET_KEY`: The secret key to use for the flask app. This is used to sign the session cookie.
- `SECRET_KEY_FILE`: The file to read the secret key from. If this is set, the `SECRET_KEY` variable will be ignored.
- `ALLOWED_HOSTS`: The allowed hosts for the server. This is used to set the `Access-Control-Allow-Origin` header. If this is not set, the header will default to `*` (This is insecure).
