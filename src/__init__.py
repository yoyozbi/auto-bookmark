import os
from flask import Flask
from flask_simplelogin import SimpleLogin
from src import UploadManager, UserManager
from apscheduler.schedulers.background import BackgroundScheduler



# Constants
SECRET_KEY = 'SECRET_KEY'
SECRET_KEY_FILE = 'SECRET_KEY_FILE'
ALLOWED_HOSTS = 'ALLOWED_HOSTS'
PWD_FILE = 'PWD_FILE'
ADMIN_PASSWORD = 'ADMIN_PASSWORD'


def create_app(test_config=None) -> Flask:
   '''
   Create and configure an instance of the Flask application.
   '''
   scheduler = BackgroundScheduler()

   app = Flask(__name__, instance_relative_config=True)
   app.config.from_mapping(
        SECRET_KEY=get_secret_key(),
        ALLOWED_HOSTS=get_allowed_hosts(),
   );

   load_passwords()

   SimpleLogin(app, login_checker=check_auth)

   UploadManager.UploadManager().set_scheduler(scheduler)

   scheduler.add_job(UploadManager.UploadManager().clear_uploads, trigger='interval', minutes=5)
   scheduler.start()

   from . import main

   app.register_blueprint(main.bp)

   return app


def get_secret_key() -> str:
    '''
    Get the secret key from a secret key file or from environment variables
    '''
    if SECRET_KEY_FILE in os.environ:
        secret_key_file = os.environ.get(SECRET_KEY_FILE)
        if not secret_key_file or not os.path.exists(secret_key_file):
            print(f"Secret key file not found at {secret_key_file}")
            exit(1)
        return open(secret_key_file).read().strip()

    if SECRET_KEY in os.environ:
        secret_key = os.environ.get(SECRET_KEY)
        if not secret_key:
            print("SECRET_KEY is empty")
            exit(1)

        return secret_key

    print("SECRET_KEY or SECRET_KEY_FILE not found")
    exit(1)

def get_allowed_hosts() -> str:
    if ALLOWED_HOSTS in os.environ:
        allowed_hosts = os.environ.get(ALLOWED_HOSTS)
        if not allowed_hosts:
            print("ALLOWED_HOSTS is empty")
            exit(1)
        return allowed_hosts

    print("WARNING: ALLOWED_HOSTS not found defaulting to * (This is insecure)")
    return '*'

def load_passwords():
    if PWD_FILE in os.environ:
        pwd_file = os.environ.get(PWD_FILE)
        if not pwd_file or not os.path.exists(pwd_file):
            print(f"Password file not found at {pwd_file}")
            exit(1)
        UserManager.UserManager().load_from_file(pwd_file)
        return

    if ADMIN_PASSWORD in os.environ:
        admin_password = os.environ.get(ADMIN_PASSWORD)
        if not admin_password:
            print("ADMIN_PASSWORD is empty")
            exit(1)
        print("No PWD_FILE found, adding admin user")
        UserManager.UserManager().add_user(UserManager.User('admin', admin_password))
        return

    print("No PWD_FILE or ADMIN_PASSWORD found")
    exit(1)

def check_auth(user):
    '''
    Check if the user is authenticated
    '''
    print(UserManager.UserManager().users)
    return UserManager.UserManager().check_user(user.get('username'), user.get('password'))


application = create_app()
