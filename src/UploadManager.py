import uuid
import time
import os
from enum import Enum
from apscheduler.schedulers.background import BackgroundScheduler
from src.generate import Generate
from src.singleton import SingletonMeta

class UploadStatus(Enum):
    IN_PROGRESS = 1
    ERROR = 2
    DONE = 3
    DOWNLOADED = 4

    def __str__(self):
        return self.name

class Upload():
    '''
    Represents an upload
    '''

    def __init__(self, id: uuid.UUID, file_paths: list[str]):

        if not id:
            raise ValueError("id is required")

        if not file_paths:
            raise ValueError("file_paths is required")

        self.id = id
        self.status = UploadStatus.IN_PROGRESS
        self.file_paths = file_paths
        self.path = None

        self.created_at = time.time()

    def __str__(self):
        return f"Upload {self.id} is {self.status}"

    def __repr__(self):
        return self.__str__()

    def to_dict(self) -> dict:
        return {"id": self.id, "status": self.status.value}


    @staticmethod
    def from_dict(data: dict):
        if not data:
            return None

        id = data.get("id")
        status = data.get("status")

        if not id or not status:
            raise ValueError("id and status are required in the dict")

        return Upload(id, status)

    def set_status(self, status: UploadStatus):
        self.status = status

    def set_path(self, path: str):
        self.path = path

    def get_status(self) -> UploadStatus:
        return self.status

    def get_id(self) -> uuid.UUID:
        return self.id

    def get_at(self) -> float:
        return self.created_at

    def get_path(self) -> str|None:
        return self.path

    def is_done(self) -> bool:
        return self.status == UploadStatus.DONE

    def is_in_progress(self) -> bool:
        return self.status == UploadStatus.IN_PROGRESS

    def is_error(self) -> bool:
        return self.status == UploadStatus.ERROR

    def is_downloaded(self) -> bool:
        return self.status == UploadStatus.DOWNLOADED

    def generate(self) -> tuple[bool, str]:
        '''
        Generate the resulting pdf file (this action takes time and should happen in the background)
        '''
        self.status = UploadStatus.IN_PROGRESS
        g = Generate(self.file_paths)
        success, message = g.generate()
        if success:
            self.status = UploadStatus.DONE
            self.path = message
        else:
            print(f"Error generating the pdf: {message}")
            self.status = UploadStatus.ERROR
            self.error_message = message

        return success, message



    def __eq__(self, other):
        return self.id == other.id

    def __hash__(self):
        return hash(self.id)

    def __ne__(self, other):
        return not self.__eq__(other)

    def __lt__(self, other):
        return self.id < other.id

    def __le__(self, other):
        return self.id <= other.id

    def __gt__(self, other):
        return self.id > other.id

    def __ge__(self, other):
        return self.id >= other.id

class UploadManager(metaclass=SingletonMeta):
    '''
    Manages Uploads in memory
    '''
    def __init__(self):
        self.uploads : set[Upload] = set()
        self.scheduler = None

    def set_scheduler(self, scheduler: BackgroundScheduler):
        '''
        Set the scheduler
        '''
        self.scheduler = scheduler

    def get_upload(self, id: uuid.UUID) -> Upload|None:
        '''
        Get the upload by id (or None if not found)
        '''
        for upload in self.uploads:
            if upload.get_id() == id:
                return upload

        return None

    def add_upload(self, upload: Upload):
        '''
        Add an upload
        '''
        self.uploads.add(upload)

    def remove_upload(self, id: uuid.UUID):
        '''
        Remove an upload
        '''
        if id in self.uploads:
            self.uploads.remove(id)

    def change_status(self, id: uuid.UUID, status: UploadStatus):
        '''
        Change the status of an upload
        '''
        upload = self.get_upload(id)
        if upload:
            upload.set_status(status)

    def schedule_pdf_generation(self, id: uuid.UUID):
        '''
        Start a background job to generate the pdf and update the status
        '''
        upload = self.get_upload(id)
        if not upload:
            return

        if self.scheduler is None:
            raise ValueError("Scheduler is not set")

        self.scheduler.add_job(upload.generate)


    def clear_uploads(self):
        '''
        Clear uploads that are older than 5 minutes
        '''
        print("Clearing uploads")
        for upload in self.uploads:
            if not upload.is_done() or not upload.is_error() or not upload.is_downloaded():
                continue

            if time.time() - upload.get_at() > 300:
                print("Clearing upload: ", upload.get_id())
                self.remove_upload(upload.get_id())
                path = upload.get_path()
                if path is None:
                    continue

                os.remove(path)

                file_paths = upload.file_paths
                for file_path in file_paths:
                    if os.path.exists(file_path):
                        os.remove(file_path)
