CREATE TABLE users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uuid BINARY(128) NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL UNIQUE
);

INSERT INTO users (id, uuid, username) VALUES (1, X'936DA01F9ABD4D9D80C702AF85C822A8', "admin");

CREATE TABLE organisations (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uuid BINARY(128) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL UNIQUE
);

INSERT INTO organisations (id, uuid, name) VALUES (1, X'936DA01F9ABD4D9D80C702AF85C822A8', "core");

CREATE TABLE crates (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    organisation_id INTEGER NOT NULL,
    readme TEXT,
    description VARCHAR(255),
    repository VARCHAR(255),
    homepage VARCHAR(255),
    documentation VARCHAR(255),
    UNIQUE (name, organisation_id),
    FOREIGN KEY (organisation_id) REFERENCES organisations (id)
);

CREATE TABLE crate_versions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    crate_id INTEGER NOT NULL,
    version VARCHAR(255) NOT NULL,
    filesystem_object VARCHAR(255) NOT NULL,
    size INTEGER NOT NULL,
    yanked BOOLEAN NOT NULL DEFAULT FALSE,
    checksum VARCHAR(255) NOT NULL,
    dependencies BLOB NOT NULL,
    features BLOB NOT NULL,
    links VARCHAR(255),
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (crate_id, version),
    FOREIGN KEY (user_id) REFERENCES users (id)
    FOREIGN KEY (crate_id) REFERENCES crates (id)
);

CREATE TABLE user_organisation_permissions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    organisation_id INTEGER NOT NULL,
    permissions INTEGER NOT NULL,
    UNIQUE (user_id, organisation_id),
    FOREIGN KEY (user_id) REFERENCES users (id)
    FOREIGN KEY (organisation_id) REFERENCES organisations (id)
);

CREATE TABLE user_crate_permissions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    crate_id INTEGER NOT NULL,
    permissions INTEGER NOT NULL,
    UNIQUE (user_id, crate_id),
    FOREIGN KEY (user_id) REFERENCES users (id)
    FOREIGN KEY (crate_id) REFERENCES crates (id)
);

CREATE TABLE user_ssh_keys (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uuid BINARY(128) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    user_id INTEGER NOT NULL,
    ssh_key BLOB NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE user_sessions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    session_key VARCHAR(255) NOT NULL UNIQUE,
    user_ssh_key_id INTEGER,
    expires_at DATETIME,
    user_agent VARCHAR(255),
    ip VARCHAR(255),
    FOREIGN KEY (user_id) REFERENCES users (id)
    FOREIGN KEY (user_ssh_key_id) REFERENCES user_ssh_keys (id)
);
