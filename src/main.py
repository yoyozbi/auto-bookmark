import os
import uuid
from flask import Blueprint, render_template, request
from flask.helpers import send_file
from flask_simplelogin import login_required
from src.generate import Generate
import shutil

from src.UploadManager import Upload, UploadStatus, UploadManager

bp = Blueprint('main', __name__, url_prefix='/')


@bp.route('/')
@login_required
def index():
    return render_template('index.html')

@bp.route('/upload', methods=['POST'])
@login_required
def upload_file():
    if not os.path.exists('uploads'):
        os.mkdir('uploads')
    temp_upload_path = os.path.abspath(f'uploads/{uuid.uuid4()}')
    os.mkdir(temp_upload_path)
    file_paths = []
    for file in request.files.getlist('files'):
        path = os.path.abspath(f'{temp_upload_path}/{file.filename}')
        file.save(path)
        file_paths.append(path)

    upload = Upload(uuid.uuid4(), file_paths)
    UploadManager().add_upload(upload)
    UploadManager().schedule_pdf_generation(upload.id)
    return str(upload.id)


@bp.route('/upload/<uuid:upload_id>/get', methods=['GET'])
@login_required
def get_upload_files(upload_id):
    upload = UploadManager().get_upload(upload_id)
    if not upload:
        return "Upload not found", 404
    if upload.status != UploadStatus.DONE:
        return "Upload not done", 400

    UploadManager().change_status(upload_id, UploadStatus.DOWNLOADED)

    if not upload.path or not os.path.exists(upload.path):
        return "File not found", 404
    return send_file(upload.path, as_attachment=True, mimetype='application/pdf')

@bp.route('/upload/<uuid:upload_id>/progress', methods=['GET'])
@login_required
def upload_progress(upload_id):
    upload = UploadManager().get_upload(upload_id)
    if not upload:
        return "Upload not found", 404

    return upload.to_dict()
