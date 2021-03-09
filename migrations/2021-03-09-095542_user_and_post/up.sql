-- Your SQL goes here
PRAGMA FOREIGN_KEYS = ON;
CREATE TABLE user (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    register_date DATETIME NOT NULL
);

CREATE TABLE message (
    id INTEGER NOT NULL PRIMARY KEY,
    user INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    pub_date DATETIME NOT NULL,
    FOREIGN KEY(user) REFERENCES user(user)
)