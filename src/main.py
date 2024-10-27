import os
import uuid
from flask import Flask, request, render_template
from flask.helpers import send_file
from flask_simplelogin import SimpleLogin, login_required
from generate import Generate
import shutil

application = Flask(__name__)
app = application
app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY')
app.config['ALLOWED_HOSTS'] = os.environ.get('ALLOWED_HOSTS')

def check_auth(user):
    return user.get('username') == 'admin' and user.get('password') == os.environ.get('PASSWORD')

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
