import os
import uuid
from flask import Flask, request, render_template
from flask.helpers import send_file
from flask_simplelogin import SimpleLogin, login_required
from werkzeug.security import check_password_hash
from generate import Generate
import shutil

application = Flask(__name__)
app = application

if 'SECRET_KEY_FILE' in os.environ:
    app.config['SECRET_KEY'] = open(os.environ.get('SECRET_KEY_FILE')).read().strip()
elif 'SECRET_KEY' in os.environ:
    app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY')
else:
    print("SECRET_KEY or SECRET_KEY_FILE not found")
    exit(1)

if 'ALLOWED_HOSTS' in os.environ:
    app.config['ALLOWED_HOSTS'] = os.environ.get('ALLOWED_HOSTS')
else:
    print("WARNING: ALLOWED_HOSTS not found defaulting to * (This is insecure)")
    app.config['ALLOWED_HOSTS'] = '*'

pwds = {}
if 'PWD_FILE' in os.environ:
    passes = open(os.environ.get('PWD_FILE')).read().strip().split('\n')
    for p in passes:
        splitted = p.split(':')
        user = splitted[0]
        pwd = splitted[1:]
        pwds[user] = pwd
elif 'ADMIN_PASSWORD' in os.environ:
    pwds['admin'] = os.environ.get('ADMIN_PASSWORD')
else:
        print("No PWD_FILE or ADMIN_PASSWORD found")
        exit(1)


def check_auth(user):
    return user.get('username') in pwds and check_password_hash(pwds[user.get('username')], user.get('password'))

SimpleLogin(app, login_checker=check_auth)

@app.route('/')
@login_required
def index():
    return render_template('index.html')

@app.route('/upload', methods=['POST'])
@login_required
def upload_file():
    if not os.path.exists('uploads'):
        os.mkdir('uploads')
    temp_upload_path = f'uploads/{uuid.uuid4()}'
    os.mkdir(temp_upload_path)
    file_paths = []
    for file in request.files.getlist('files'):
        path = f'{temp_upload_path}/{file.filename}'
        file.save(path)
        file_paths.append(path)

    g = Generate(file_paths)
    success, message = g.generate()
    if success:
        shutil.rmtree(temp_upload_path)
        return send_file(f'../{message}', as_attachment=True, mimetype='application/pdf')
    else:
        return message, 400
