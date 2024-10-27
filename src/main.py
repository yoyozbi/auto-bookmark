import os
import uuid
from flask import Flask, request, render_template
from flask.helpers import send_file
from flask_simplelogin import SimpleLogin, login_required
from werkzeug.security import check_password_hash
from generate import Generate
import shutil
from enum import Enum
from apscheduler.schedulers.background import BackgroundScheduler

application = Flask(__name__)
app = application

scheduler = BackgroundScheduler()



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
        pwd = ':'.join(splitted[1:])
        pwds[user] = pwd
elif 'ADMIN_PASSWORD' in os.environ:
    pwds['admin'] = os.environ.get('ADMIN_PASSWORD')
else:
        print("No PWD_FILE or ADMIN_PASSWORD found")
        exit(1)


def check_auth(user):
    return user.get('username') in pwds and check_password_hash(pwds[user.get('username')], user.get('password'))

SimpleLogin(app, login_checker=check_auth)


UPLOADS = {}

def clear_uploads():
    print("Clearing uploads")
    for upload_id, data in UPLOADS.items():
        if data.get("status") == UploadStatus.DOWNLOADED:
            os.remove(data.get("path"))
            del UPLOADS[upload_id]
        elif data.get("status") == UploadStatus.ERROR:
            print(f"Error: {data.get('message')}")

scheduler.add_job(clear_uploads, trigger='interval', minutes=5)

scheduler.start()

class UploadStatus(Enum):
    IN_PROGRESS = 1
    ERROR = 2
    DONE = 3
    DOWNLOADED = 4

    def __str__(self):
        return self.name

def generate_file(g: Generate, upload_id: uuid.UUID):
    print(f"Generating {upload_id}")
    UPLOADS[upload_id] = {"status": UploadStatus.IN_PROGRESS}
    success, message = g.generate()
    if success:
        UPLOADS[upload_id] = {"status": UploadStatus.DONE, "path": message}
    else:
        UPLOADS[upload_id] = {"status": UploadStatus.ERROR, "message": message}

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
    scheduler.add_job(generate_file, args=[g, g.upload_id], id=str(g.upload_id))
    return str(g.upload_id)


@app.route('/upload/<uuid:upload_id>/get', methods=['GET'])
@login_required
def get_upload_files(upload_id):
    if upload_id not in UPLOADS:
        return "Upload not found", 404
    if UPLOADS[upload_id]["status"] != UploadStatus.DONE:
        return "Upload not done", 400

    UPLOADS[upload_id]["status"] = UploadStatus.DOWNLOADED
    return send_file(UPLOADS[upload_id]["path"], as_attachment=True, mimetype='application/pdf')

@app.route('/upload/<uuid:upload_id>/progress', methods=['GET'])
@login_required
def upload_progress(upload_id):
    if upload_id not in UPLOADS:
        return "Upload not found", 404

    transformed = UPLOADS[upload_id].copy()
    transformed["status"] = transformed["status"].value
    return transformed
