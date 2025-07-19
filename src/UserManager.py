from src.singleton import SingletonMeta
from werkzeug.security import check_password_hash
import os

class User():
    '''
    Represents a user of the system (username and hashed_password)
    '''
    def __init__(self, username: str, hashed_password: str):
        if not username:
            raise ValueError("username is required")

        if not hashed_password:
            raise ValueError("hashed_password is required")

        self.username = username
        self.hashed_password = hashed_password

    def __str__(self):
        return f"User {self.username}"

    def __repr__(self):
        return self.__str__()

    def check_password(self, password: str) -> bool:
        '''
        Check if the password is correct
        '''
        return check_password_hash(self.hashed_password, password)


class UserManager(metaclass=SingletonMeta):
    '''
    Singleton class that manages authorized users list
    '''
    def __init__(self):
        self.users : set[User] = set()

    def add_user(self, user: User):
        '''
        Add a user
        '''
        self.users.add(user)

    def remove_user(self, user: User):
        '''
        Remove a user
        '''
        self.users.remove(user)

    def get_user(self, username: str) -> User|None:
        '''
        Get a user by username
        '''
        for user in self.users:
            if user.username == username:
                return user

        return None

    def check_user(self, username: str, password: str) -> bool:
        '''
        Check if the password is correct
        '''
        print(f"Checking user {username}, {password}")
        user = self.get_user(username)
        if not user:
            return False

        return user.check_password(password)

    def load_from_file(self, filename: str):
        '''
        Load users from a file
        The file should have the following format:
            username:hashed_password
            other_username:other_hashed_password
        '''
        if not filename:
            raise ValueError("filename is required")

        if not os.path.exists(filename):
            raise FileNotFoundError(f"File {filename} not found")


        passes = open(filename).read().strip().split('\n')
        for p in passes:
            splitted = p.split(':')
            user = splitted[0]
            pwd = ':'.join(splitted[1:])
            self.add_user(User(user, pwd))
