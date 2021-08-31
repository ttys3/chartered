CREATE TABLE crates (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE crate_versions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    crate_id INTEGER NOT NULL,
    version VARCHAR(255) NOT NULL,
    filesystem_object VARCHAR(255) NOT NULL,
    yanked BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE (crate_id, version),
    FOREIGN KEY (crate_id) REFERENCES crates (id)
);

INSERT INTO crates VALUES (1, "cool-test-crate");
INSERT INTO crate_versions VALUES (1, 1, "1.0.0", "cool-object", FALSE);